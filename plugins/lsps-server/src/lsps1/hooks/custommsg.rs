use anyhow::Result;
use uuid::Uuid;

use lsp_primitives::methods;

use lsp_primitives::json_rpc::ErrorData;
use lsp_primitives::lsps0::common_schemas::{IsoDatetime, NetworkCheckable, Outpoint};
use lsp_primitives::lsps0::parameter_validation::ParamValidationError;
use lsp_primitives::lsps1::builders::Lsps1CreateOrderResponseBuilder;
use lsp_primitives::lsps1::schema::{
    Channel, Lsps1CreateOrderResponse, Lsps1GetInfoResponse, OrderState, Payment
};

use crate::custom_msg::context::CustomMsgContext;
use crate::db::schema::Lsps1Order;
use crate::db::sqlite::queries::{
    GetChannelQuery, GetOrderQuery, GetPaymentDetailsQuery, Lsps1CreateOrderQuery,
};
use crate::lsps1::fee_calc::StandardFeeCalculator;
use crate::lsps1::msg::{BuildLsps1Order, BuildUsingDbPayment};
use crate::lsps1::payment_calc::PaymentCalc;
use crate::{options, PluginState};

pub(crate) async fn check_lsps1_enabled(
    context: &mut CustomMsgContext<PluginState>,
) -> Result<(), ErrorData> {
    let lsps1_enabled = context.plugin.option(&options::lsps1_enable()).unwrap();

    if lsps1_enabled {
        Ok(())
    } else {
        log::debug!("Ignored call because lsps1 is disabled");
        Err(ErrorData::method_not_found(&context.request.method))
    }
}

pub(crate) async fn do_lsps1_get_info(
    method: methods::Lsps1GetInfo,
    context: &mut CustomMsgContext<PluginState>,
) -> Result<Lsps1GetInfoResponse, ErrorData> {
    log::debug!("lsps1_get_info");

    check_lsps1_enabled(context).await?;
    method.into_typed_request(context.request.clone())?;
    let state = context.plugin.state();
    let info_response = state.lsps1_info.as_ref().clone();
    info_response.ok_or_else(|| ErrorData::method_not_found(method.name()))
}

pub(crate) async fn do_lsps1_create_order(
    method: methods::Lsps1CreateOrder,
    context: &mut CustomMsgContext<PluginState>,
) -> Result<Lsps1CreateOrderResponse, ErrorData> {
    log::debug!(
        "Handling lsps1.create_order from peer={:?}",
        context.peer_id
    );

    check_lsps1_enabled(context).await?;
    let typed_request = method.into_typed_request(context.request.clone())?;

    let state = context.plugin.state();

    // Define the relevant timestamps
    let now = IsoDatetime::now();
    let created_at = now.clone();
    let expires_at = now.clone();

    let order = typed_request.params;
    order
        .refund_onchain_address
        .require_network(&context.network)
        .map_err(|e| {
            ParamValidationError::invalid_params(
                "order.refund_onchain_address".to_string(),
                e.to_string(),
            )
        })?;

    // TODO: find a nicer way to get the options
    let info_response = state
        .lsps1_info
        .as_ref()
        .clone()
        .ok_or_else(|| ErrorData::method_not_found(method.name()))?;

    // Return an error if the order is invalid
    order.validate_options(&info_response.options)?;

    // Construct the database order object
    let lsps1_order = Lsps1Order {
        uuid: Uuid::new_v4(),
        client_node_id: context.peer_id,
        announce_channel: order.announce_channel,
        created_at,
        expires_at,
        lsp_balance_sat: order.lsp_balance_sat,
        client_balance_sat: order.client_balance_sat,
        funding_confirms_within_blocks: order.funding_confirms_within_blocks,
        required_channel_confirmations: order.required_channel_confirmations,
        channel_expiry_blocks: order.channel_expiry_blocks,
        token: order.token.clone(),
        refund_onchain_address: order.refund_onchain_address.as_ref().map(|x| x.to_string()),
        order_state: OrderState::Created,
        generation: 0,
    };

    // Compute the fee
    // TODO: Provide a way to configure the fee calculation
    let opt = options::lsps1_fee_computation_base_fee_sat();
    let base_fee = context.plugin.option(&opt).unwrap();
    let weight_units = context
        .plugin
        .option(&options::lsps1_fee_computation_onchain_ppm())
        .unwrap();
    let sat_per_billion_sat_block = context
        .plugin
        .option(&options::lsps1_fee_computation_liquidity_ppb())
        .unwrap();

    let fee_calc = StandardFeeCalculator {
        fixed_msat: base_fee as u64,
        weight_units: weight_units as u64,
        sat_per_billion_sat_block: sat_per_billion_sat_block as u64,
    };
    let mut payment_calc = PaymentCalc { fee_calc };
    let payment = payment_calc
        .compute_payment_details(context, &lsps1_order)
        .await
        .map_err(ErrorData::internalize)?;

    // Write everything to the database
    let db = context.plugin.state().database.clone();
    let query = Lsps1CreateOrderQuery {
        order: lsps1_order,
        payment,
    };

    let mut tx = db.begin().await.map_err(ErrorData::internalize)?;
    let _ = query
        .execute(&mut tx)
        .await
        .map_err(ErrorData::internalize)?;

    tx.commit().await.map_err(ErrorData::internalize)?;

    // Construct the response that we will send to the user
    let payment = Payment {
        min_fee_for_0conf: None,
        min_onchain_payment_confirmations: None,
        onchain_address: None,
        onchain_payment: None,
        fee_total_sat: query.payment.fee_total_sat,
        order_total_sat: query.payment.order_total_sat,
        bolt11_invoice: query.payment.bolt11_invoice,
        state: query.payment.state
    };

    let response = Lsps1CreateOrderResponse {
        order_id: query.order.uuid,
        lsp_balance_sat: order.lsp_balance_sat,
        client_balance_sat: order.client_balance_sat,
        funding_confirms_within_blocks: order.funding_confirms_within_blocks,
        required_channel_confirmations: order.required_channel_confirmations,
        channel_expiry_blocks: order.channel_expiry_blocks,
        token: order.token.unwrap_or(String::from("")),
        order_state: query.order.order_state,
        announce_channel: order.announce_channel,
        created_at,
        expires_at,
        payment,
        channel: None
    };
    Ok(response)
}

pub(crate) async fn do_lsps1_get_order(
    method: methods::Lsps1GetOrder,
    context: &mut CustomMsgContext<PluginState>,
) -> Result<Lsps1CreateOrderResponse, ErrorData> {
    check_lsps1_enabled(context).await?;
    let typed_request = method.into_typed_request(context.request.clone())?;

    let uuid_value =
        Uuid::parse_str(&typed_request.params.order_id).map_err(ErrorData::internalize)?;

    let db = context.plugin.state().database.clone();

    let mut tx = db.begin().await.map_err(ErrorData::internalize)?;

    let get_order_query = GetOrderQuery {
        order_id: uuid_value,
    };

    log::debug!("Retriever order details from database");
    let order = get_order_query
        .execute(&mut tx)
        .await
        .map_err(ErrorData::internalize)?
        .ok_or_else(ErrorData::not_found)?;

    log::debug!("Retreive payment details from database");
    let payment_details = GetPaymentDetailsQuery::by_uuid(uuid_value)
        .execute(&mut tx)
        .await
        .map_err(ErrorData::internalize)?
        .ok_or_else(|| ErrorData::internalize("Failed to find payment corresponding to order"))?;

    let payment = Payment::from_db_payment(payment_details);

    log::debug!("Retrieve channel info from database");
    let channel_details = GetChannelQuery::by_order_id(uuid_value)
        .execute(&mut tx)
        .await
        .map_err(ErrorData::internalize)?;

    let channel_details = match channel_details {
        Some(channel_details) => {
            // Convert from database type to Lsps1 spec type
            let channel = Channel {
                funded_at: channel_details.funded_at,
                funding_outpoint: Outpoint {
                    txid: channel_details.funding_txid,
                    outnum: channel_details.outnum,
                },
                expires_at: IsoDatetime::now(),
            };

            Some(channel)
        }
        None => None,
    };

    tx.commit().await.map_err(ErrorData::internalize)?;

    Lsps1CreateOrderResponseBuilder::new()
        .db_order(order)
        .payment(payment)
        .channel(channel_details)
        .build()
        .map_err(ErrorData::internalize)
}
