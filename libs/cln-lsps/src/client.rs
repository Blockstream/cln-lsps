use std::io::{Cursor, Write};

use anyhow::{anyhow, Context, Result};
use lsp_primitives::json_rpc::{
    generate_random_rpc_id, JsonRpcId, JsonRpcMethod, JsonRpcResponse, NoParams,
};

use lsp_primitives::lsps0;
use lsp_primitives::lsps0::common_schemas::PublicKey;
use lsp_primitives::lsps1;
use lsp_primitives::methods;

use async_trait::async_trait;

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub struct RequestId {
    peer_id: PublicKey,
    msg_id: JsonRpcId,
}

impl RequestId {
    pub fn new(peer_id: PublicKey, msg_id: JsonRpcId) -> Self {
        Self { peer_id, msg_id }
    }
}

// BOLT8 message ID 37913
pub const LSPS_MESSAGE_ID: [u8; 2] = [0x94, 0x19];
pub const LSPS_MESSAGE_ID_U16: u16 = 30971;
pub const TIMEOUT_MILLIS: u128 = 30_000;

/// The LspClient
///
/// It can send JSON-rpc requests to an LSP-server
/// and parses the responses. But also has other utilities such as the
/// discovery of LSP-servers.
///
/// The trait has two implementations depending on how the client
/// or plug-in connects to the Core Lightning-node.
///
/// - `RpcLspClient`: Talks to Core Lightning using JSON-RPC. Recommended when
///   the LSP-client is implemented as a Core Lightning
///   plug-in. The rpc-client is enabled by default and the user doesn't
///   need to specify any additional configuration.
/// - `GrpcLspClient`: Talks to Core Lighting using gRPC. Recommended when
///   the LSP-client and Core Lightning node are not running on the same
///   device. (e.g: Greenlight)
#[async_trait]
pub trait LspClient {
    /// Make a JSON-RPC 2.0 request to an LSP-server and specify the
    /// message-id explicitly.
    ///
    /// We recommend using `request` over `request_with_id` because it
    /// creates LSPS0-compliant id for the caller. LSPS0 prescribes
    /// that the id should have at least 80 bytes of entropy.
    ///
    /// However, when writing tests it is useful to be able
    /// to manipulate or access the `rpc_id` explicitly.
    /// We recommend using `request` over `request_with_id` because it the
    async fn request_with_id<'a, I, O, E>(
        &mut self,
        peer_id: &PublicKey,
        method: JsonRpcMethod<'a, I, O, E>,
        param: I,
        rpc_id: JsonRpcId,
    ) -> Result<JsonRpcResponse<O, E>>
    where
        I: serde::Serialize + Send,
        O: serde::de::DeserializeOwned + Send,
        E: serde::de::DeserializeOwned + Send;

    async fn list_lsps(&mut self) -> Result<Vec<PublicKey>>;

    /// Make a JSON-RPC 2.0 request to an LSP-server
    async fn request<'a, I, O, E>(
        &mut self,
        peer_id: &PublicKey,
        method: JsonRpcMethod<'a, I, O, E>,
        param: I,
    ) -> Result<JsonRpcResponse<O, E>>
    where
        I: serde::Serialize + Send,
        O: serde::de::DeserializeOwned + Send,
        E: serde::de::DeserializeOwned + Send,
    {
        let rpc_id = generate_random_rpc_id();
        self.request_with_id(peer_id, method, param, rpc_id).await
    }

    async fn lsps0_list_protocols(
        &mut self,
        peer_id: &PublicKey,
    ) -> Result<lsps0::schema::ListprotocolsResponse> {
        let response = self
            .request(peer_id, methods::LSPS0_LIST_PROTOCOLS, NoParams)
            .await?;
        match response {
            JsonRpcResponse::Error(err) => {
                Err(anyhow!("{} : {}", err.error.code, err.error.message))
            }
            JsonRpcResponse::Ok(ok) => Ok(ok.result),
        }
    }

    async fn lsps1_get_info(
        &mut self,
        peer_id: &PublicKey,
    ) -> Result<lsps1::schema::Lsps1GetInfoResponse> {
        let response = self
            .request(peer_id, methods::LSPS1_GETINFO, NoParams)
            .await?;
        match response {
            JsonRpcResponse::Error(err) => {
                Err(anyhow!("{} : {}", err.error.code, err.error.message))
            }
            JsonRpcResponse::Ok(ok) => Ok(ok.result),
        }
    }

    async fn lsps1_create_order(
        &mut self,
        peer_id: &PublicKey,
        order_request: lsps1::schema::Lsps1CreateOrderRequest,
    ) -> Result<lsps1::schema::Lsps1CreateOrderResponse> {
        let response = self
            .request(peer_id, methods::LSPS1_CREATE_ORDER, order_request)
            .await?;

        // TODO: We probably want to store this order in the data-store
        match response {
            JsonRpcResponse::Error(err) => {
                Err(anyhow!("{} :  {}", err.error.code, err.error.message))
            }
            JsonRpcResponse::Ok(ok) => Ok(ok.result),
        }
    }
}

pub fn rpc_request_to_data<I, O, E>(
    json_rpc_id: &JsonRpcId,
    method: JsonRpcMethod<I, O, E>,
    params: I,
) -> Result<String>
where
    I: serde::Serialize,
{
    let request = method.create_request(params, json_rpc_id.clone());

    // In CoreLightning the sendcustommsg rpc command expects that the message
    // data starts with 2-bytes that represent the BOLT-8 message id followed
    // by the message payload
    let mut cursor: Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
    cursor.write_all(&LSPS_MESSAGE_ID)?;
    serde_json::to_writer(&mut cursor, &request)
        .with_context(|| "Failed to parse JsonRpcRequest")?;

    let request_hex = hex::encode(cursor.into_inner());
    Ok(request_hex)
}
