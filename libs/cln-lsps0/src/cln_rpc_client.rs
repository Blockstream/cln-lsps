use crate::client::{rpc_request_to_data, LspClient, RequestId};
use crate::transport::RequestResponseMatcher;
use lsp_primitives::json_rpc::{JsonRpcId, JsonRpcMethod, JsonRpcResponse, MapErrorCode};
use lsp_primitives::lsps0::common_schemas::PublicKey;
use lsp_primitives::lsps0::util::{is_feature_bit_enabled, FeatureBitMap, LSP_SERVER_FEATURE_BIT};

use async_trait::async_trait;
use cln_rpc::{
    model::{
        requests::{ListnodesRequest, SendcustommsgRequest},
        Request, Response,
    },
    ClnRpc,
};
use std::str::FromStr;

use anyhow::{Context, Result};

use std::sync::{Arc, Mutex};

use log::info;

type Matcher = Arc<Mutex<RequestResponseMatcher<RequestId, serde_json::Value>>>;

pub struct ClnRpcLspClient {
    matcher: Matcher,
    rpc: ClnRpc,
}

impl ClnRpcLspClient {
    pub fn new(matcher: Matcher, rpc: ClnRpc) -> Self {
        Self { matcher, rpc }
    }
}

#[async_trait]
impl LspClient for ClnRpcLspClient {
    async fn request_with_id<'a, I, O, E>(
        &mut self,
        peer_id: &PublicKey,
        method: JsonRpcMethod<'a, I, O, E>,
        params: I,
        json_rpc_id: JsonRpcId,
    ) -> Result<JsonRpcResponse<O, E>>
    where
        E: MapErrorCode,
        I: serde::Serialize + Send,
        O: serde::de::DeserializeOwned + Send,
        E: serde::de::DeserializeOwned + Send,
    {
        // Construct the request
        // The request_data is hex-encoded message, The first two bytes represent the BOLT-8 msg id
        let request_data: String = rpc_request_to_data(&json_rpc_id, method, params).unwrap();
        let request_id = RequestId::new(peer_id.clone(), json_rpc_id);

        log::debug!("JSON-rpc request '{}'", request_data);

        // Start listening for the response
        // We do this before sending it to avoid race-conditions
        let response_future = self.matcher.lock().unwrap().process_request(request_id);

        // Send the custom message
        let cln_rpc_pubkey =
            cln_rpc::primitives::PublicKey::from_slice(&peer_id.inner().serialize())
                .context("Unexpected failure in PublicKey")?;
        let request_data = SendcustommsgRequest {
            node_id: cln_rpc_pubkey,
            msg: request_data,
        };
        let request = Request::SendCustomMsg(request_data);
        log::debug!("SendCustomMessageRequest {:?}", request.clone());
        let result =
            self.rpc.call(request).await.with_context(|| {
                "Failed to SendcustomMsg to peer. Are you connected to the peer?"
            })?;

        match result {
            Response::SendCustomMsg(status) => {
                // TODO: 
                // We should be able to get some error information from the status.
                // It is unclear what that information would be
                info!("Status {:?}", status)
            },
            _ => panic!("An unexpected error occured from which cannot be recovered. Should have responded with SendcustomMsg-response.")
        }

        // Wait for the response
        let timeout = std::time::Duration::from_secs(10);
        let response_value: serde_json::Value = tokio::time::timeout(timeout, response_future)
            .await
            .with_context(|| "Time-out, waiting for peer to respond")?;

        // Parse the response and return the value
        serde_json::from_value(response_value)
            .with_context(|| "Failed to parse response from LSPS-server")
    }

    async fn list_lsps(&mut self) -> Result<Vec<PublicKey>> {
        let list_nodes_request = ListnodesRequest { id: None };
        let response = self
            .rpc
            .call(Request::ListNodes(list_nodes_request))
            .await?;

        if let Response::ListNodes(list_nodes_response) = response {
            list_nodes_response
                .nodes
                .iter()
                .filter_map(|n| {
                    // fix me: spurious clone
                    let features = FeatureBitMap::from_str(&n.features.clone()?).ok()?;

                    if is_feature_bit_enabled(&features, LSP_SERVER_FEATURE_BIT) {
                        return Some(PublicKey::from_hex(&n.nodeid.to_string()));
                    } else {
                        return None;
                    }
                })
                .collect()
        } else {
            panic!("Should respond with listNodes")
        }
    }
}
