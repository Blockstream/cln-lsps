use lsp_primitives::lsps0::common_schemas::{FeeRate, IsoDatetime, PublicKey, SatAmount};
use uuid::Uuid;

pub struct Lsps1Order {
    pub(crate) uuid: Uuid,
    pub(crate) client_node_id: PublicKey,
    pub(crate) lsp_balance_sat: SatAmount,
    pub(crate) client_balance_sat: SatAmount,
    pub(crate) confirms_within_blocks: u8,
    pub(crate) channel_expiry_blocks: u32,
    pub(crate) token: Option<String>,
    pub(crate) refund_onchain_address: Option<String>,
    pub(crate) announce_channel: bool,
    pub(crate) created_at: IsoDatetime,
    pub(crate) expires_at: IsoDatetime,
}

pub struct Lsps1PaymentDetails {
    pub(crate) fee_total_sat: SatAmount,
    pub(crate) order_total_sat: SatAmount,
    pub(crate) bolt11_invoice: String,
    pub(crate) onchain_address: Option<String>,
    pub(crate) onchain_block_confirmations_required: Option<u8>,
    pub(crate) minimum_fee_for_0conf: Option<FeeRate>,
}
