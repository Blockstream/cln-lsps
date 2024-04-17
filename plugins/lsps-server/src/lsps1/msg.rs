use crate::db::schema::{Lsps1Order, Lsps1PaymentDetails};
use lsp_primitives::lsps1::builders::{Lsps1CreateOrderResponseBuilder, PaymentBuilder};

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
    fn db_payment_details(self, lsps1_payment: Lsps1PaymentDetails) -> Self;
}

impl BuildUsingDbPayment for PaymentBuilder {
    fn db_payment_details(self, payment: Lsps1PaymentDetails) -> Self {
        self.fee_total_sat(payment.fee_total_sat)
            .order_total_sat(payment.order_total_sat)
            .bolt11_invoice(payment.bolt11_invoice)
            .onchain_address(None)
            .minimum_fee_for_0conf(payment.minimum_fee_for_0conf)
            .state(payment.state)
    }
}
