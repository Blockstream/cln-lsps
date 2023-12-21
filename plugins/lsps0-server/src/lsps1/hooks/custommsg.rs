use anyhow::Result;
use lsp_primitives::lsps0::parameter_validation::InvalidParam;
use uuid::Uuid;

use lsp_primitives::methods;

use lsp_primitives::json_rpc::ErrorData;
use lsp_primitives::lsps0::parameter_validation::ParamValidationError;
use lsp_primitives::lsps0::common_schemas::{IsoDatetime, NetworkCheckable, SatAmount};
use lsp_primitives::lsps1::builders::{Lsps1CreateOrderResponseBuilder, PaymentBuilder};
use lsp_primitives::lsps1::schema::{Lsps1CreateOrderResponse, Lsps1InfoResponse, OrderState};

use crate::custom_msg::context::CustomMsgContext;
use crate::db::sqlite::queries::{GetOrderQuery, GetPaymentDetailsQuery, Lsps1CreateOrderQuery};
use crate::lsps1::fee_calc::FixedFeeCalculator;
use crate::lsps1::msg::{BuildLsps1Order, BuildUsingDbPayment};
use crate::lsps1::payment_calc::PaymentCalc;
use crate::lsps1_utils;
use crate::PluginState;

pub(crate) async fn do_lsps1_get_info(
    _method: methods::Lsps1Info,
    context: &mut CustomMsgContext<PluginState>,
) -> Result<Lsps1InfoResponse, ErrorData> {
    log::debug!("lsps1_get_info");
    lsps1_utils::info_response(context.plugin.options())
        .map_err(ErrorData::internalize)
}

pub(crate) async fn do_lsps1_create_order(
    method: methods::Lsps1CreateOrder,
    context: &mut CustomMsgContext<PluginState>,
) -> Result<Lsps1CreateOrderResponse, ErrorData> {
    log::debug!("lsps1_create_order");

    // Define the relevant timestamps
    let now = IsoDatetime::now();
    let created_at = now.clone();
    let expires_at = now.clone();

    // Parse the request and return an invalid params if it fails
    // TODO: The current error is not LSPS1-spec compliant
    let typed_request = method
        .into_typed_request(context.request.clone())?;

    let order = typed_request.params;
    order
        .refund_onchain_address
        .require_network(&context.network)
        .map_err(|e| ParamValidationError::invalid_params(
            "order.refund_onchain_address".to_string(),
             e.to_string()))?;

    // TODO: find a nicer way to get the options
    let info_response = lsps1_utils::info_response(context.plugin.options())
        .map_err(ErrorData::internalize)?;
    let options = info_response.options;

    // Return an error if the order is invalid
    order
        .validate_options(&options)?;

    // Construct the database order object
    let lsps1_order = crate::db::schema::Lsps1OrderBuilder::new()
        .order_request(&order)
        .client_node_id(context.peer_id)
        .created_at(created_at)
        .expires_at(expires_at)
        .order_state(OrderState::Created)
        .build()
        .map_err(ErrorData::internalize)?;

    // Compute the fee and cgeneratioionstruct all order details
    // TODO: Implement an actual mechanism to compute the fee
    // Ideally, we would have alinear system built-in and
    // provide hooks to the operator to allow them to implement their
    // own fees.
    let fee_calc = FixedFeeCalculator::new(SatAmount::new(10_000));
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

    let mut tx = db.begin().await
        .map_err(ErrorData::internalize)?;
    let _ = query.execute(&mut tx)
        .await
        .map_err(ErrorData::internalize)?;

    tx.commit()
        .await
        .map_err(ErrorData::internalize)?;

    // Construct the response that we will send to the user
    let payment = PaymentBuilder::new()
        .fee_total_sat(query.payment.fee_total_sat)
        .order_total_sat(query.payment.order_total_sat)
        .bolt11_invoice(query.payment.bolt11_invoice)
        .state(query.payment.state)
        .build()
        .map_err(ErrorData::internalize)?;

    let response = Lsps1CreateOrderResponseBuilder::from_request(order)
        .uuid(query.order.uuid)
        .created_at(created_at)
        .expires_at(expires_at)
        .order_state(OrderState::Created)
        .payment(payment)
        .build()
        .map_err(ErrorData::internalize)?;

    Ok(response)
}

pub(crate) async fn do_lsps1_get_order(
    method: methods::Lsps1GetOrder,
    context: &mut CustomMsgContext<PluginState>,
) -> Result<Lsps1CreateOrderResponse, ErrorData> {

    let typed_request = method
        .into_typed_request(context.request.clone())?;

    let uuid_value = Uuid::parse_str(&typed_request.params.order_id)
        .map_err(ErrorData::internalize)?;

    let db = context.plugin.state().database.clone();

    let mut tx = db.begin().await
        .map_err(ErrorData::internalize)?;

    // TODO: Risk of PhantomData
    // Read both queries in a single transaction
    let get_order_query = GetOrderQuery {
        order_id: uuid_value,
    };
    let order = get_order_query
        .execute(&mut tx)
        .await
        .map_err(ErrorData::internalize)?
        .ok_or_else(ErrorData::not_found )?;

    log::info!("Storing payment details in db");

    let payment_details = GetPaymentDetailsQuery::by_uuid(uuid_value)
        .execute(&mut tx)
        .await
        .map_err(ErrorData::internalize)?
        .ok_or_else(|| ErrorData::internalize("Failed to find payment corresponding to order"))?;

    log::info!("Creating payment");
    let payment = PaymentBuilder::new()
        .db_payment_details(payment_details)
        .build()
        .map_err(ErrorData::internalize)?;

    Lsps1CreateOrderResponseBuilder::new()
        .db_order(order)
        .payment(payment)
        .order_state(OrderState::Created)
        .build()
        .map_err(ErrorData::internalize)
}
