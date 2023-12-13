///
/// This is the schema of all RPC-commands created
/// by the LSP-client plugin.
///
/// I know, JSON-RPC is used for different communications
/// which adds to the confusion.
/// - core-Lightning RPC (scope of this file)
/// - RPC between LSP-client and LSP-server (not the scope of this file)
///
use serde::{Deserialize, Serialize};

use lsp_primitives::lsps0::common_schemas::{OnchainAddress, SatAmount};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListProtocolsRequest {
    pub peer_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ListProtocolsResponse {
    pub protocols: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Lsps1GetInfoRequest {
    pub peer_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Lsps1CreateOrderRequest {
    pub peer_id: String,
    pub lsp_balance_sat: SatAmount,
    pub client_balance_sat: Option<SatAmount>,
    pub confirms_within_blocks: Option<u8>,
    pub channel_expiry_blocks: u32,
    pub token: Option<String>,
    pub refund_onchain_address: Option<OnchainAddress>,
    pub announce_channel: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Lsps0SendRequest {
    pub peer_id: String,
    pub method: String,
    pub params: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Lsps1GetOrderRequest {
    pub peer_id: String,
    pub order_id: String,
}
