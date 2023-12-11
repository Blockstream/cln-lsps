use cln_plugin::Plugin;

use lsp_primitives::lsps1::schema::PaymentState;

use crate::cln_objects::Payment;
use crate::db::schema::Lsps1PaymentDetails;
use crate::error::InvoicePaymentError;
use crate::state::PluginState;

pub(crate) async fn invoice_payment(
    plugin: Plugin<PluginState>,
    payment: &Payment,
) -> Result<(), InvoicePaymentError> {
    let db = &plugin.state().database;

    log::info!("Looking for payment with label in database");
    let details: Lsps1PaymentDetails = db
        .get_payment_details_by_label(&payment.label)
        .await
        .map_err(|x| InvoicePaymentError::Log(x.to_string()))? // Error quering in database
        .ok_or_else(|| InvoicePaymentError::NotMine)?; // No value was found

    db.update_payment_state(&payment.label, PaymentState::Paid, details.generation)
        .await
        .map_err(|x| InvoicePaymentError::Log(x.to_string()))?;

    Ok(())
}
