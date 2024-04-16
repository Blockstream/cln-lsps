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
    pub funding_confirms_within_blocks: Option<u8>,
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

type RpcMethodBuilder = cln_plugin::RpcMethodBuilder<crate::PluginState>;

pub fn lsps0_list_servers_method() -> RpcMethodBuilder {
    RpcMethodBuilder::new("lsps0-list-servers", crate::list_lsp_servers)
        .description("List all lsps-servers that have publicly announced themselves")
}

pub fn lsps0_list_protocols_method() -> RpcMethodBuilder {
    RpcMethodBuilder::new("lsps0-list-protocols", crate::list_protocols)
        .description("List all lsps-servers that have publicly announced themselves")
        .usage("peer_id")
}

pub fn lsps0_send_request() -> RpcMethodBuilder {
    RpcMethodBuilder::new("lsps0-send-request", crate::lsps0_send_request)
        .usage("For devs: Send request to an LSP-server")
        .usage("peer_id method [params]")
}

pub fn lsps1_get_info() -> RpcMethodBuilder {
    RpcMethodBuilder::new("lsps1-get-info", crate::lsps1_get_info)
        .description("Get info and pricing to purchase a channel from an LSP")
        .usage("peer_id")
}

pub fn lsps1_create_order() -> RpcMethodBuilder {
    RpcMethodBuilder::new("lsps1-create-order", crate::lsps1_create_order)
        .description("Order a channel from an LSP")
        .usage("peer_id lsp_balance_sat channel_expiry_blocks [client_balance_sat] [confirms_within_blocks] [token] [refund_onchain_address] [announce_channel]")
}

pub fn lsps1_get_order() -> RpcMethodBuilder {
    RpcMethodBuilder::new("lsps1-get-order", crate::lsps1_get_order)
        .description("Request info about an order")
        .usage("peer_id order_id")
}
