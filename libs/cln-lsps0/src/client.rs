use std::io::{Cursor, Write};

use anyhow::{anyhow, Context, Result};
use lsp_primitives::json_rpc::{
    generate_random_rpc_id, JsonRpcId, JsonRpcMethod, JsonRpcResponse, MapErrorCode,
};

use async_trait::async_trait;
use serde::de::{Deserializer, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub struct RequestId {
    peer_id: PubKey,
    msg_id: JsonRpcId,
}

impl RequestId {
    pub fn new(peer_id: PubKey, msg_id: JsonRpcId) -> Self {
        Self { peer_id, msg_id }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PubKey {
    key: [u8; 33],
}

impl std::fmt::Display for PubKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let hex = hex::encode(self.key);
        write!(f, "{}", hex)
    }
}

impl AsRef<[u8]> for PubKey {
    fn as_ref(&self) -> &[u8] {
        &self.key
    }
}

impl PubKey {
    pub fn new(key: [u8; 33]) -> Self {
        return Self { key };
    }

    pub fn from_hex(key: &str) -> Result<Self> {
        let data = hex::decode(key)
            .with_context(|| "Failed to parse PubKey. Expected a hex-representation")?;
        Self::try_new(data)
    }

    pub fn try_new<T: AsRef<[u8]>>(key: T) -> Result<Self> {
        let src: &[u8] = key.as_ref();
        if !src.len() == 33 {
            return Err(anyhow!("Expected a 33 byte ECDSA key"));
        }

        let mut dest: [u8; 33] = [0; 33];
        dest.copy_from_slice(src);

        Ok(Self { key: dest })
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.key.to_vec()
    }
}

impl Serialize for PubKey {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let str_value = self.to_string();
        serializer.serialize_str(&str_value)
    }
}

struct PubKeyVisitor;

impl<'de> Visitor<'de> for PubKeyVisitor {
    type Value = PubKey;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("A hex-encoded compressed public key of length 33 bytes")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        PubKey::from_hex(value).map_err(|_| E::custom(format!("Failed to parse PubKey as hex")))
    }
}

impl<'de> Deserialize<'de> for PubKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(PubKeyVisitor)
    }
}

// BOLT8 message ID 37913
pub const LSPS_MESSAGE_ID: [u8; 2] = [0x94, 0x19];
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
        peer_id: &PubKey,
        method: JsonRpcMethod<I, O, E>,
        param: I,
        rpc_id: JsonRpcId,
    ) -> Result<JsonRpcResponse<O, E>>
    where
        E: MapErrorCode,
        I: serde::Serialize + Send,
        O: serde::de::DeserializeOwned + Send,
        E: serde::de::DeserializeOwned + Send;

    async fn list_lsps(&mut self) -> Result<Vec<PubKey>>;

    /// Make a JSON-RPC 2.0 request to an LSP-server
    async fn request<'a, I, O, E>(
        &mut self,
        peer_id: &PubKey,
        method: JsonRpcMethod<I, O, E>,
        param: I,
    ) -> Result<JsonRpcResponse<O, E>>
    where
        E: MapErrorCode,
        I: serde::Serialize + Send,
        O: serde::de::DeserializeOwned + Send,
        E: serde::de::DeserializeOwned + Send,
    {
        let rpc_id = generate_random_rpc_id();
        self.request_with_id(peer_id, method, param, rpc_id).await
    }
}

pub fn rpc_request_to_data<I, O, E>(
    json_rpc_id: &JsonRpcId,
    method: JsonRpcMethod<I, O, E>,
    params: I,
) -> Result<String>
where
    E: MapErrorCode,
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
