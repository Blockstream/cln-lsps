use anyhow::Result;

use cln_rpc::model::requests::InvoiceRequest;
use cln_rpc::primitives::AmountOrAny;

use lsp_primitives::lsps0::common_schemas::SatAmount;

use crate::custom_msg::context::CustomMsgContext;
use crate::lsps1::fee_calc::FeeCalculator;
use crate::PluginState;

use crate::db::schema::{Lsps1Order, Lsps1PaymentDetails, Lsps1PaymentDetailsBuilder};

pub struct PaymentCalc<T: FeeCalculator> {
    pub(crate) fee_calc: T,
}

impl<T: FeeCalculator> PaymentCalc<T> {
    pub async fn compute_payment_details(
        &mut self,
        context: &mut CustomMsgContext<PluginState>,
        order: &Lsps1Order,
    ) -> Result<Lsps1PaymentDetails> {
        // Compute the fee-rate and the bolt11-invoice
        let fee = self.fee_calc.calculate_fee(context, order.clone())?;
        let bolt_11_invoice_label = format!("lsps1_{}", order.uuid);
        let bolt11_invoice = self
            .construct_bolt11_invoice(context, order, fee.order_total_sat, &bolt_11_invoice_label)
            .await?;

        // We do not support onchain payments.
        // This allows us to be lazy here
        Lsps1PaymentDetailsBuilder::new()
            .fee_total_sat(fee.fee_total_sat)
            .order_total_sat(fee.order_total_sat)
            .bolt11_invoice(bolt11_invoice)
            .bolt11_invoice_label(bolt_11_invoice_label)
            .build()
    }

    async fn construct_bolt11_invoice(
        &mut self,
        context: &mut CustomMsgContext<PluginState>,
        order: &Lsps1Order,
        amount: SatAmount,
        label: &str,
    ) -> Result<String> {
        // cln_rpc
        let cln_rpc = &mut context.cln_rpc;

        // Construct the description
        let channel_capacity = SatAmount::new(
            order.lsp_balance_sat.sat_value() + order.client_balance_sat.sat_value(),
        );
        let channel_expiry_blocks = order.channel_expiry_blocks;
        let description = format!(
            "LSPS1: Request channel with capacity {} for {} blocks",
            channel_capacity, channel_expiry_blocks
        );

        let cln_amount = cln_rpc::primitives::Amount::from_sat(amount.sat_value());
        let invoice_request = InvoiceRequest {
            amount_msat: AmountOrAny::Amount(cln_amount),
            label: label.to_string(),
            description,
            expiry: Some(order.expires_at.unix_timestamp() as u64),
            cltv: None,
            deschashonly: None,
            fallbacks: None,
            preimage: None,
        };

        let invoice_response = cln_rpc.call_typed(invoice_request).await?;

        return Ok(invoice_response.bolt11);
    }
}
