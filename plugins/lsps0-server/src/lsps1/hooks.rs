use anyhow::Result;

use lsp_primitives::methods::server as server_methods;

use lsp_primitives::lsps0::common_schemas::{
    IsoDatetime, NetworkCheckable, NetworkChecked, SatAmount,
};
use lsp_primitives::lsps1::builders::{Lsps1CreateOrderResponseBuilder, PaymentBuilder};
use lsp_primitives::lsps1::schema::{
    Lsps1CreateOrderRequest, Lsps1CreateOrderResponse, Lsps1InfoResponse, OrderState,
};

use crate::custom_msg::context::CustomMsgContext;
use crate::error::CustomMsgError;
use crate::lsps1::fee_calc::FixedFeeCalculator;
use crate::lsps1::payment_calc::PaymentCalc;
use crate::lsps1_utils;
use crate::PluginState;

pub(crate) async fn do_lsps1_get_info(
    _method: server_methods::Lsps1Info,
    context: &mut CustomMsgContext<PluginState>,
) -> Result<Lsps1InfoResponse, CustomMsgError> {
    log::debug!("lsps1_get_info");

    let info_response = lsps1_utils::info_response(context.plugin.options());

    match info_response {
        Ok(response) => Ok(response),
        Err(err) => {
            log::warn!("Error in handling lsps1.list_protocols call");
            log::warn!("{:?}", err);
            Err(CustomMsgError::InternalError(
                "Error in handling lsps1.list_protocols call".into(),
            ))
        }
    }
}

pub(crate) async fn do_lsps1_create_order(
    method: server_methods::Lsps1CreateOrder,
    context: &mut CustomMsgContext<PluginState>,
) -> Result<Lsps1CreateOrderResponse<NetworkChecked>, CustomMsgError> {
    log::debug!("lsps1_create_order");

    // Define the relevant timestamps
    let now = IsoDatetime::now();
    let created_at = now.clone();
    let expires_at = now.clone();

    // Parse the request and return an invalid params if it fails
    // TODO: The current error is not LSPS1-spec compliant
    let typed_request = method
        .into_typed_request(context.request.clone())
        .map_err(|x| CustomMsgError::InvalidParams(x.to_string().into()))?;

    let order: Lsps1CreateOrderRequest<NetworkChecked> = typed_request
        .params
        .require_network(context.network)
        .map_err(|x| CustomMsgError::InvalidParams(x.to_string().into()))?;

    // TODO: find a nicer way to get the options
    let info_response = lsps1_utils::info_response(context.plugin.options())
        .map_err(|e| CustomMsgError::InternalError(e.to_string().into()))?;
    let options = info_response.options;

    // TODO: Return the approriate error information
    // consult LSPS1 cause it is rather picky about the response and error-code
    order.validate_options(&options)
        .map_err(|e| CustomMsgError::Lsps1OptionMismatch(e))?;

    // Construct the database order object
    let lsps1_order = crate::db::schema::Lsps1OrderBuilder::new()
        .order_request(&order)
        .client_node_id(context.peer_id)
        .created_at(created_at)
        .expires_at(expires_at)
        .order_state(OrderState::Created)
        .build()
        .map_err(|e| CustomMsgError::InternalError(e.to_string().into()))?;

    // Compute the fee and construct all order details
    // TODO: Implement an actual mechanism to compute the fee
    // Ideally, we would have alinear system built-in and
    // provide hooks to the operator to allow them to implement their
    // own fees.
    let fee_calc = FixedFeeCalculator::new(SatAmount::new(10_000));
    let mut payment_calc = PaymentCalc { fee_calc };
    let payment = payment_calc
        .compute_payment_details(context, &lsps1_order)
        .await
        .map_err(|e| CustomMsgError::InternalError(e.to_string().into()))?;

    // Write everything to the database
    let db = context.plugin.state().database.clone();
    db.create_order(&lsps1_order, &payment)
        .await
        .map_err(|e| CustomMsgError::InternalError(e.to_string().into()))?;

    // Construct the response that we will send to the user
    let payment = PaymentBuilder::new()
        .fee_total_sat(payment.fee_total_sat)
        .order_total_sat(payment.order_total_sat)
        .bolt11_invoice(payment.bolt11_invoice)
        .build()
        .map_err(|e| CustomMsgError::InternalError(e.to_string().into()))?;

    let response = Lsps1CreateOrderResponseBuilder::from_request(order)
        .uuid(lsps1_order.uuid)
        .created_at(created_at)
        .expires_at(expires_at)
        .order_state(OrderState::Created)
        .payment(payment)
        .build()
        .map_err(|e| CustomMsgError::InternalError(e.to_string().into()))?;

    Ok(response)
}