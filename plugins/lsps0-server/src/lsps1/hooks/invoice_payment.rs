use anyhow::{Context, Result};
use cln_plugin::Plugin;

use lsp_primitives::lsps1::schema::PaymentState;

use crate::cln::hooks::invoice_payment::InvoicePaymentHookResponse;
use crate::cln::hooks::invoice_payment::Payment;
use crate::db::sqlite::queries::{GetPaymentDetailsQuery, UpdatePaymentStateQuery};
use crate::state::PluginState;

pub(crate) async fn invoice_payment(
    plugin: Plugin<PluginState>,
    payment: &Payment,
) -> Result<InvoicePaymentHookResponse> {
    let db = &plugin.state().database;
    let mut tx = db.begin().await?;

    log::info!("Looking for payment with label in database");
    // Check if we should handle the invoice_payment hook
    // We'll only handle the hook if we are sure the payment
    // is lsps1-related
    let payment_details = GetPaymentDetailsQuery::ByLabel(String::from(payment.label.clone()))
        .execute(&mut tx)
        .await
        .with_context(|| "Failed to execute 'get_payment_details_by_label'-query on database")?;

    if payment_details.is_none() {
        // The lsps1-plugin can ignore this payment
        // This payment is unrelated
        return Ok(InvoicePaymentHookResponse::Continue);
    }
    let payment_details = payment_details.unwrap();

    // Set the payment-state to hold in the database
    // The hook is called so we have received the HTLC
    let update_payment = UpdatePaymentStateQuery {
        state: PaymentState::Hold,
        generation: payment_details.generation,
        label: payment.label.to_string(),
    }
    .execute(&mut tx)
    .await?;

    tx.commit().await?;

    // Here we attempt to open the channel
    // This might take a while because we need to reach out
    // to our peer and might want to wait for channel confirmation
    //
    // This op should be cancellable
    let result = try_open_channel().await;

    // Set the state to paid if succeeded
    let new_state = match result {
        Ok(_) => PaymentState::Paid,
        Err(_) => PaymentState::Refunded,
    };

    let mut tx = db.begin().await?;

    // Set the payment-state to hold in the database
    // The hook is called so we have received the HTLC
    UpdatePaymentStateQuery {
        state: new_state,
        generation: payment_details.generation + 1,
        label: payment.label.to_string(),
    }
    .execute(&mut tx)
    .await?;

    tx.commit().await?;

    return Ok(InvoicePaymentHookResponse::Continue);
}

async fn try_open_channel() -> Result<(), ()> {
    // Doesn't actually open the channel
    // We just sleep to replicate the effect
    let _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    Ok(())
}
