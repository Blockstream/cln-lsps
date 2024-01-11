use anyhow::{anyhow, Context, Result};
use cln_plugin::Plugin;
use cln_rpc::ClnRpc;

use lsp_primitives::lsps0::common_schemas::SatAmount;
use lsp_primitives::lsps1::schema::{OrderState, PaymentState};

use crate::channel_open::{fundchannel_fallible, ChannelDetails};
use crate::cln::hooks::invoice_payment::InvoicePaymentHookResponse;
use crate::cln::hooks::invoice_payment::Payment;
use crate::db::schema::Lsps1Channel;
use crate::db::sqlite::queries::{GetOrderQuery, UpdateOrderStateQuery, CreateChannelQuery};
use crate::db::sqlite::queries::{GetPaymentDetailsQuery, UpdatePaymentStateQuery, };
use crate::state::PluginState;

pub(crate) async fn invoice_payment(
    plugin: Plugin<PluginState>,
    payment: &Payment,
) -> Result<InvoicePaymentHookResponse> {
    let db = &plugin.state().database;
    let mut tx = db.begin().await?;

    log::debug!("Looking for payment with label in database");
    // Check if we should handle the invoice_payment hook
    // We'll only handle the hook if we are sure the payment
    // is lsps1-related
    let payment_details: Option<crate::db::schema::Lsps1PaymentDetails> =
        GetPaymentDetailsQuery::ByLabel(String::from(payment.label.clone()))
            .execute(&mut tx)
            .await
            .with_context(|| {
                "Failed to execute 'get_payment_details_by_label'-query on database"
            })?;

    if payment_details.is_none() {
        // The lsps1-plugin can ignore this payment
        // This payment is unrelated
        return Ok(InvoicePaymentHookResponse::Continue);
    }
    let payment_details = payment_details.ok_or_else(|| anyhow!("No payment details"))?;

    // Set the payment-state to hold in the database
    // The hook is called so we have received the HTLC
    UpdatePaymentStateQuery {
        state: PaymentState::Hold,
        generation: payment_details.generation,
        label: payment.label.to_string(),
    }
    .execute(&mut tx)
    .await?;

    tx.commit().await?;

    let mut tx = db.begin().await?;

    let order_details: crate::db::schema::Lsps1Order =
        GetOrderQuery::by_uuid(payment_details.order_uuid)
            .execute(&mut tx)
            .await
            .context("Failed to execute 'get_order_details'-query on database")?
            .context("Failed to find order that corresponds to payment")?;
    let peer_id = order_details.client_node_id;
    tx.commit().await?;

    // Here we attempt to open the channel
    // This might take a while because we need to reach out
    // to our peer and might want to wait for channel confirmation
    //
    // This op should be cancellable
    let rpc_path = plugin.configuration().rpc_file;
    log::debug!("Attempt to connect to rpc-interface '{}'", rpc_path);
    let mut rpc = ClnRpc::new(rpc_path).await?;

    let timeout = std::time::Duration::from_secs(60);
    let amount = order_details
            .client_balance_sat
            .checked_add(&order_details.lsp_balance_sat)
            .context("Overflow when computing channel capacity")?;

    let mindepth = Some(
            order_details
                .confirms_within_blocks
                .checked_sub(6)
                .unwrap_or(0),
        );

    let channel_details = ChannelDetails {
        peer_id: order_details.client_node_id,
        amount: amount,
        mindepth : mindepth,
        push_msat: Some(order_details.client_balance_sat),
        reserve: Some(SatAmount::new(0)),
        close_to: None,
        feerate: None,
    };

    log::debug!("Atempting to open channel ");
    let channel_result = fundchannel_fallible(&mut rpc, &channel_details, timeout).await;

    let mut tx = db.begin().await?;
    match channel_result {
        Ok(channelopen_response) => {
            log::info!(
                "Successfully opened channel to {:?} and received payment",
                peer_id
            );

            UpdatePaymentStateQuery {
                state: PaymentState::Paid,
                generation: payment_details.generation + 1,
                label: payment.label.to_string(),
            }
            .execute(&mut tx)
            .await?;

            UpdateOrderStateQuery {
                order_uuid : order_details.uuid,
                state : OrderState::Completed,
            }.execute(&mut tx)
            .await?;

            tx.commit().await?;
            return Ok(InvoicePaymentHookResponse::Continue);
        }
        Err(err) => {
            log::info!("Refund payment for LSPS1-channel. Channel open failed");
            log::warn!("Error: {}", err);
            UpdatePaymentStateQuery {
                state: PaymentState::Refunded,
                generation: payment_details.generation + 1,
                label: payment.label.to_string(),
            }
            .execute(&mut tx)
            .await?;

            UpdateOrderStateQuery {
                order_uuid : order_details.uuid,
                state : OrderState::Failed,
            }.execute(&mut tx)
            .await?;

            tx.commit().await?;
            return Ok(InvoicePaymentHookResponse::Reject);
        }
    }
}
