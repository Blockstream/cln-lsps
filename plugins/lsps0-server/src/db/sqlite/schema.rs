use anyhow::Context;
use uuid::Uuid;

use crate::db::schema::{
    Lsps1Order as Lsps1OrderBase, Lsps1PaymentDetails as Lsps1PaymentDetailsBase,
};
use crate::db::sqlite::conversion::{FromSqliteInteger, IntoSqliteInteger};
use lsp_primitives::lsps0::common_schemas::{FeeRate, IsoDatetime, PublicKey, SatAmount};
use lsp_primitives::lsps1::schema::{OrderState, PaymentState};

#[derive(sqlx::FromRow)]
pub struct Lsps1Order {
    pub(crate) uuid: String,
    pub(crate) client_node_id: String,
    pub(crate) lsp_balance_sat: i64,
    pub(crate) client_balance_sat: i64,
    pub(crate) confirms_within_blocks: i64,
    pub(crate) channel_expiry_blocks: i64,
    pub(crate) token: Option<String>,
    pub(crate) refund_onchain_address: Option<String>,
    pub(crate) announce_channel: bool,
    pub(crate) created_at: i64,
    pub(crate) expires_at: i64,
    pub(crate) order_state: i64,
    pub(crate) generation: i64,
}

#[derive(sqlx::FromRow)]
pub struct Lsps1PaymentDetails {
    pub(crate) order_uuid: String,
    pub(crate) fee_total_sat: i64,
    pub(crate) order_total_sat: i64,
    pub(crate) bolt11_invoice: String,
    pub(crate) bolt11_invoice_label: String,
    pub(crate) onchain_address: Option<String>,
    pub(crate) onchain_block_confirmations_required: Option<i64>,
    pub(crate) minimum_fee_for_0conf: Option<i64>,
    pub(crate) state: i64,
    pub(crate) generation: i64,
}

impl TryFrom<&Lsps1PaymentDetailsBase> for Lsps1PaymentDetails {
    type Error = anyhow::Error;

    fn try_from(payment: &Lsps1PaymentDetailsBase) -> Result<Self, Self::Error> {
        let min_0conf = payment
            .minimum_fee_for_0conf
            .clone()
            .map(|f| f.to_sats_per_kwu())
            .map(|f| i64::try_from(f))
            .transpose()?;
        let block_conf = payment
            .onchain_block_confirmations_required
            .map(|n| i64::try_from(n))
            .transpose()?;

        Ok(Self {
            order_uuid: payment.order_uuid.to_string(),
            fee_total_sat: i64::try_from(payment.fee_total_sat.sat_value())?,
            order_total_sat: i64::try_from(payment.order_total_sat.sat_value())?,
            bolt11_invoice: payment.bolt11_invoice.clone(),
            bolt11_invoice_label: payment.bolt11_invoice_label.clone(),
            onchain_address: payment.onchain_address.clone(),
            onchain_block_confirmations_required: block_conf,
            minimum_fee_for_0conf: min_0conf,
            state: payment.state.into_sqlite_integer()?,
            generation: payment.generation.into_sqlite_integer()?,
        })
    }
}

impl TryFrom<&Lsps1PaymentDetails> for Lsps1PaymentDetailsBase {
    type Error = anyhow::Error;

    fn try_from(payment: &Lsps1PaymentDetails) -> Result<Self, Self::Error> {
        let onchain_block_confirmations_required = payment
            .onchain_block_confirmations_required
            .map(|p| u8::from_sqlite_integer(p))
            .transpose()?;

        let minimum_fee_for_0conf = payment
            .onchain_block_confirmations_required
            .map(|p| FeeRate::from_sqlite_integer(p))
            .transpose()?;

        Ok(Self {
            order_uuid: Uuid::parse_str(&payment.order_uuid)
                .context("order_uuid is not a valid uuid")?,
            fee_total_sat: SatAmount::from_sqlite_integer(payment.fee_total_sat)?,
            order_total_sat: SatAmount::from_sqlite_integer(payment.fee_total_sat)?,
            bolt11_invoice: payment.bolt11_invoice.clone(),
            bolt11_invoice_label: payment.bolt11_invoice_label.clone(),
            onchain_address: payment.onchain_address.clone(),
            onchain_block_confirmations_required,
            minimum_fee_for_0conf,
            state: PaymentState::from_sqlite_integer(payment.state)?,
            generation: u64::from_sqlite_integer(payment.generation)?,
        })
    }
}

impl TryFrom<&Lsps1Order> for Lsps1OrderBase {
    type Error = anyhow::Error;

    fn try_from(order: &Lsps1Order) -> Result<Self, Self::Error> {
        Ok(Self {
            uuid: Uuid::parse_str(&order.uuid)?,
            client_node_id: PublicKey::from_hex(&order.client_node_id)?,
            lsp_balance_sat: SatAmount::new(u64::try_from(order.lsp_balance_sat)?),
            client_balance_sat: SatAmount::new(u64::try_from(order.client_balance_sat)?),
            confirms_within_blocks: u8::try_from(order.confirms_within_blocks)?,
            channel_expiry_blocks: u32::try_from(order.channel_expiry_blocks)?,
            token: order.token.clone(),
            refund_onchain_address: order.refund_onchain_address.clone(),
            announce_channel: order.announce_channel,
            created_at: IsoDatetime::from_unix_timestamp(order.created_at)?,
            expires_at: IsoDatetime::from_unix_timestamp(order.expires_at)?,
            order_state: OrderState::from_sqlite_integer(order.order_state)?,
            generation: u64::from_sqlite_integer(order.generation)?,
        })
    }
}

impl TryFrom<&Lsps1OrderBase> for Lsps1Order {
    type Error = anyhow::Error;

    fn try_from(order: &Lsps1OrderBase) -> Result<Self, Self::Error> {
        Ok(Self {
            uuid: order.uuid.to_string(),
            client_node_id: order.client_node_id.to_hex(),
            lsp_balance_sat: i64::try_from(order.lsp_balance_sat.sat_value())?,
            client_balance_sat: i64::try_from(order.client_balance_sat.sat_value())?,
            confirms_within_blocks: i64::try_from(order.confirms_within_blocks)?,
            channel_expiry_blocks: i64::try_from(order.channel_expiry_blocks)?,
            token: order.token.clone(),
            refund_onchain_address: order.refund_onchain_address.clone(),
            announce_channel: order.announce_channel,
            created_at: order.created_at.unix_timestamp(),
            expires_at: order.expires_at.unix_timestamp(),
            order_state: order.order_state.into_sqlite_integer()?,
            generation: order.generation.into_sqlite_integer()?,
        })
    }
}
