use anyhow::Result;

use lsp_primitives::json_rpc::{JsonRpcResponse, DefaultError, ErrorData};
use lsp_primitives::methods::server as server_methods;

use lsp_primitives::lsps0::common_schemas::{NetworkChecked, NetworkCheckable, SatAmount, IsoDatetime};
use lsp_primitives::lsps1::schema::{Lsps1CreateOrderRequest, Lsps1CreateOrderResponse, OrderState};
use lsp_primitives::lsps1::builders::{Lsps1CreateOrderResponseBuilder, PaymentBuilder};

use crate::lsps1_utils;
use crate::lsps1::fee_calc::FixedFeeCalculator;
use crate::lsps1::payment_calc::PaymentCalc;
use crate::custom_msg::context::CustomMsgContext;
use crate::{PluginState, do_continue, send_response};

pub(crate) async fn do_lsps1_get_info(
    _method: server_methods::Lsps1Info,
    mut context: CustomMsgContext<PluginState>,
) -> Result<serde_json::Value> {
    log::debug!("lsps1_get_info");

    let info_response = lsps1_utils::info_response(context.plugin.options());

    let response = match info_response {
        Ok(ok) => JsonRpcResponse::success(context.request.id, ok),
        Err(err) => {
            log::warn!("Error in handling lsps1.list_protocols call");
            log::warn!("Error {:?}", err);
            let error = ErrorData::<DefaultError>::internal_error("Internal server error".into());
            JsonRpcResponse::error(context.request.id, error)
        }
    };

    send_response(&mut context.cln_rpc, context.peer_id, response).await?;
    do_continue()
}

pub(crate) async fn do_lsps1_create_order(
    method : server_methods::Lsps1CreateOrder,
    mut context : CustomMsgContext<PluginState>,
) -> Result<serde_json::Value> {
    log::debug!("lsps1_create_order");

    let now = IsoDatetime::now();
    let created_at = now.clone();
    let expires_at = now.clone();

    // TODO: Respond with error
    let typed_request = method.into_typed_request(context.request.clone()).unwrap();

    // TODO: Send a nice error message here
    let order : Lsps1CreateOrderRequest<NetworkChecked> = typed_request
        .params
        .require_network(context.network)
        .unwrap();

    // TODO: find a nicer way to get the options
    // TODO: Remove the unwrap and return the approriate error information
    let info_response = lsps1_utils::info_response(context.plugin.options()).unwrap();
    let options = info_response.options;
    
   // TODO: Return the approriate error information
   // consult LSPS1 cause it is rather picky about the response and error-code
   order.validate_options(&options).unwrap();

   // Construct the database order object
   // TODO: Handle the error approriately
   let lsps1_order = crate::db::schema::Lsps1OrderBuilder::new()
       .order_request(&order)
       .client_node_id(context.peer_id)
       .created_at(created_at)
       .expires_at(expires_at)
       .order_state(OrderState::Created)
       .build()
       .unwrap();

   // Compute the fee and construct all order details
   // TODO: Implement an actual mechanism to compute the fee
   // Ideally, we would have alinear system built-in and 
   // provide hooks to the operator to allow them to implement their 
   // own fees.
   let fee_calc = FixedFeeCalculator::new(SatAmount::new(10_000));
   let mut payment_calc = PaymentCalc { fee_calc : Box::new(fee_calc) };
   let payment = payment_calc.compute_payment_details(&mut context, &lsps1_order).await.unwrap();

    // Write everything to the database
    let db = context.plugin.state().database.clone();
    db.create_order(&lsps1_order, &payment).await.unwrap();

    // Construct the response that we will send to the user
    let payment = PaymentBuilder::new()
        .fee_total_sat(payment.fee_total_sat)
        .order_total_sat(payment.order_total_sat)
        .bolt11_invoice(payment.bolt11_invoice)
        .build().unwrap();

    let response = Lsps1CreateOrderResponseBuilder::from_request(order)
        .uuid(lsps1_order.uuid)
        .created_at(created_at)
        .expires_at(expires_at)
        .order_state(OrderState::Created)
        .payment(payment)
        .build()
        .unwrap();


    let json_rpc_response = JsonRpcResponse::<_,DefaultError>::success(context.request.id, response);
    send_response(&mut context.cln_rpc, context.peer_id, json_rpc_response).await?;
    return do_continue();
}
