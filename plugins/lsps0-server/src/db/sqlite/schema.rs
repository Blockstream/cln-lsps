use uuid::Uuid;

use crate::db::schema::Lsps1Order as Lsps1OrderBase;
use lsp_primitives::lsps0::common_schemas::{SatAmount, IsoDatetime, PublicKey};


#[derive(sqlx::FromRow)]
pub struct Lsps1Order {
    pub (crate) uuid : String,
    pub (crate) client_node_id : String,
    pub (crate) lsp_balance_sat : i64,
    pub (crate) client_balance_sat : i64,
    pub (crate) confirms_within_blocks : i64,
    pub (crate) channel_expiry_blocks : i64,
    pub (crate) token : Option<String>,
    pub (crate) refund_onchain_address : Option<String>,
    pub (crate) announce_channel : bool,
    pub (crate) created_at : i64,
    pub (crate) expires_at : i64
}

impl TryFrom<&Lsps1Order> for Lsps1OrderBase {

    type Error = anyhow::Error;

    fn try_from(order: &Lsps1Order) -> Result<Self, Self::Error> {
        Ok(
            Self {
                uuid : Uuid::parse_str(&order.uuid)?,
                client_node_id : PublicKey::from_hex(&order.client_node_id)?,
                lsp_balance_sat : SatAmount::new(u64::try_from(order.lsp_balance_sat)?),
                client_balance_sat : SatAmount::new(u64::try_from(order.client_balance_sat)?),
                confirms_within_blocks : u8::try_from(order.confirms_within_blocks)?,
                channel_expiry_blocks : u32::try_from(order.channel_expiry_blocks)?,
                token : order.token.clone(),
                refund_onchain_address : order.refund_onchain_address.clone(),
                announce_channel : order.announce_channel,
                created_at : IsoDatetime::from_unix_timestamp(order.created_at)?,
                expires_at : IsoDatetime::from_unix_timestamp(order.expires_at)?
            })

    }
}

impl TryFrom<&Lsps1OrderBase> for Lsps1Order {
    type Error = anyhow::Error;

    fn try_from(order: &Lsps1OrderBase) -> Result<Self, Self::Error> {
        Ok(
            Self {
                uuid : order.uuid.to_string(),
                client_node_id : order.client_node_id.to_hex(),
                lsp_balance_sat : i64::try_from(order.lsp_balance_sat.sat_value())?,
                client_balance_sat : i64::try_from(order.client_balance_sat.sat_value())?,
                confirms_within_blocks : i64::try_from(order.confirms_within_blocks)?,
                channel_expiry_blocks : i64::try_from(order.channel_expiry_blocks)?,
                token : order.token.clone(),
                refund_onchain_address : order.refund_onchain_address.clone(),
                announce_channel : order.announce_channel,
                created_at : order.created_at.unix_timestamp(),
                expires_at : order.expires_at.unix_timestamp()
            })

    }
}

