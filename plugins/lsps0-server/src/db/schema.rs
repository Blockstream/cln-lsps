use lsp_primitives::lsps0::common_schemas::{SatAmount, IsoDatetime};
use uuid::Uuid;

pub struct Lsps1Order {
    pub (crate) uuid : Uuid,
    pub (crate) lsp_balance_sat : SatAmount,
    pub (crate) client_balance_sat : SatAmount,
    pub (crate) confirms_within_blocks : u8,
    pub (crate) channel_expiry_blocks : u32,
    pub (crate) token : Option<String>,
    pub (crate) refund_onchain_address : Option<String>,
    pub (crate) announce_channel : bool,
    pub (crate) created_at : IsoDatetime,
    pub (crate) expires_at : IsoDatetime
}
