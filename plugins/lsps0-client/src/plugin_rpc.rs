///
/// This is the schema of all RPC-commands created
/// by the LSP-client plugin.
///
/// I know, JSON-RPC is used for different communications
/// which adds to the confusion.
/// - core-Lightning RPC (scope of this file)
/// - RPC between LSP-client and LSP-server (not the scope of this file)
///
use anyhow::Result;
use serde::{Deserialize, Serialize};

use lsp_primitives::lsps0::common_schemas::{
    Network, NetworkCheckable, NetworkChecked, NetworkUnchecked, NetworkValidation, OnchainAddress,
    SatAmount,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct ListProtocolsRequest {
    pub peer_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListProtocolsResponse {
    pub protocols: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Lsps1GetInfoRequest {
    pub peer_id: String,
}

#[derive(Deserialize, Debug)]
pub struct Lsps1CreateOrderRequest<V: NetworkValidation> {
    pub peer_id: String,
    pub lsp_balance_sat: SatAmount,
    pub client_balance_sat: SatAmount,
    pub confirms_within_blocks: u8,
    pub channel_expiry_blocks: u32,
    pub token: Option<String>,
    #[serde(bound(deserialize = "OnchainAddress<V> : Deserialize<'de>"))]
    pub refund_onchain_address: Option<OnchainAddress<V>>,
    pub announce_channel: bool,
}

impl NetworkCheckable for Lsps1CreateOrderRequest<NetworkUnchecked> {
    type Checked = Lsps1CreateOrderRequest<NetworkChecked>;

    fn require_network(self, network: Network) -> Result<Self::Checked> {
        Ok(Self::Checked {
            peer_id: self.peer_id,
            lsp_balance_sat: self.lsp_balance_sat,
            client_balance_sat: self.client_balance_sat,
            confirms_within_blocks: self.confirms_within_blocks,
            channel_expiry_blocks: self.channel_expiry_blocks,
            token: self.token,
            refund_onchain_address: self.refund_onchain_address.require_network(network)?,
            announce_channel: self.announce_channel,
        })
    }
}
