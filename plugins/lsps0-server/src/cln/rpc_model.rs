use cln_rpc::primitives as rpc_primitives;
use cln_rpc::TypedRequest;
/// The cln-rpc crate doesn't expose requests for
/// for `fundchannel_start`, `fundchannel_complete`, `fundchannel_cancel`
///  (yet).
///
/// I'll delete this file once a sustainable approach is available in core lightning.
// TODO:    Remove this file once a sustainable solution is available in cln
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FundChannelStartRequest {
    pub id: rpc_primitives::PublicKey,
    pub amount: rpc_primitives::Amount,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feerate: Option<rpc_primitives::Feerate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_msat: Option<rpc_primitives::Amount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mindepth: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reserve: Option<rpc_primitives::Amount>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FundChannelStartResponse {
    pub funding_address: String,
    pub script_pubkey: Option<String>,
    pub close_to: Option<String>,
}

impl TypedRequest for FundChannelStartRequest {
    type Response = FundChannelStartResponse;

    fn method(&self) -> &str {
        "fundchannel_start"
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FundChannelCompleteRequest {
    pub id: rpc_primitives::PublicKey,
    pub psbt: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FundChannelCompleteResponse {
    pub channel_id: String,
    pub commitments_secured: bool,
}

impl TypedRequest for FundChannelCompleteRequest {
    type Response = FundChannelCompleteResponse;

    fn method(&self) -> &str {
        "fundchannel_complete"
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FundChannelCancelRequest {
    pub id: rpc_primitives::PublicKey,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FundChannelCancelResponse {
    pub cancelled: String,
}

impl TypedRequest for FundChannelCancelRequest {
    type Response = FundChannelCancelResponse;

    fn method(&self) -> &str {
        "fundchannel_cancel"
    }
}
