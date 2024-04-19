use lsp_primitives::lsps0::common_schemas::{
    FeeRate, IsoDatetime, PublicKey, SatAmount, TransactionId,
};
use lsp_primitives::lsps1::schema::{OrderState, PaymentState};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Lsps1Order {
    pub(crate) uuid: Uuid,
    pub(crate) client_node_id: PublicKey,
    pub(crate) lsp_balance_sat: SatAmount,
    pub(crate) client_balance_sat: SatAmount,
    pub(crate) funding_confirms_within_blocks: u16,
    pub(crate) required_channel_confirmations: u16,
    pub(crate) channel_expiry_blocks: u32,
    pub(crate) token: Option<String>,
    pub(crate) refund_onchain_address: Option<String>,
    pub(crate) announce_channel: bool,
    pub(crate) created_at: IsoDatetime,
    pub(crate) expires_at: IsoDatetime,
    pub(crate) order_state: OrderState,
    pub(crate) generation: u64,
}

#[derive(Debug, Clone)]
pub struct Lsps1PaymentDetails {
    pub(crate) order_uuid: Uuid,
    pub(crate) fee_total_sat: SatAmount,
    pub(crate) order_total_sat: SatAmount,
    pub(crate) bolt11_invoice: String,
    pub(crate) bolt11_invoice_label: String,
    pub(crate) onchain_address: Option<String>,
    pub(crate) onchain_block_confirmations_required: Option<u16>,
    pub(crate) minimum_fee_for_0conf: Option<FeeRate>,
    pub(crate) state: PaymentState,
    pub(crate) generation: u64,
}

#[derive(Debug, Clone)]
pub struct Lsps1Channel {
    pub(crate) funding_txid: TransactionId,
    pub(crate) outnum: u32,
    pub(crate) funded_at: IsoDatetime,
}
