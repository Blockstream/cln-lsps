use crate::db::schema::{Lsps1Order, Lsps1PaymentDetails};
use lsp_primitives::lsps1::builders::{Lsps1CreateOrderResponseBuilder};
use lsp_primitives::lsps1::schema::Payment;

pub trait BuildLsps1Order {
    fn db_order(self, lsps1_order: Lsps1Order) -> Self;
}

impl BuildLsps1Order for Lsps1CreateOrderResponseBuilder {
    fn db_order(self, order: Lsps1Order) -> Self {
        self.uuid(order.uuid)
            .lsp_balance_sat(order.lsp_balance_sat)
            .client_balance_sat(order.client_balance_sat)
            .funding_confirms_within_blocks(order.funding_confirms_within_blocks)
            .required_channel_confirmations(order.required_channel_confirmations)
            .channel_expiry_blocks(order.channel_expiry_blocks)
            .token(order.token.unwrap_or("".to_string()))
            .announce_channel(order.announce_channel)
            .created_at(order.created_at)
            .expires_at(order.expires_at)
            .order_state(order.order_state)
    }
}



pub trait BuildUsingDbPayment {
    fn from_db_payment(lsps1_payment: Lsps1PaymentDetails) -> Self;
}

impl BuildUsingDbPayment for Payment {
    fn from_db_payment(payment: Lsps1PaymentDetails) -> Self {
        Self {
            fee_total_sat: payment.fee_total_sat,
            order_total_sat: payment.order_total_sat,
            bolt11_invoice: payment.bolt11_invoice,
            min_fee_for_0conf: None,
            min_onchain_payment_confirmations: None,
            onchain_address: None,
            onchain_payment: None,
            state: payment.state,
        }
    }
}
