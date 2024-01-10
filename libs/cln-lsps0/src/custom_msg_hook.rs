use anyhow::{anyhow, Context, Result};
use lsp_primitives::lsps0::common_schemas::PublicKey;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcCustomMsgMessage {
    pub peer_id: PublicKey,
    pub payload: String,
}

impl RpcCustomMsgMessage {
    pub fn to_raw(&self) -> Result<RawCustomMsgMessage> {
        let peer_id = self.peer_id;
        let payload = hex::decode(&self.payload).with_context(|| "payload is not valid hex")?;

        RawCustomMsgMessage::new(peer_id, payload)
    }
}

#[derive(Debug)]
pub struct RawCustomMsgMessage {
    peer_id: PublicKey,
    // bolt-8 message id appended by the message content
    payload: Vec<u8>,
}

impl RawCustomMsgMessage {
    fn new(peer_id: PublicKey, payload: Vec<u8>) -> Result<Self> {
        if payload.len() < 2 {
            return Err(anyhow!(
                "Payload in custommsg should be at least 2 bytes to allow for the BOLT_8_MSG_ID"
            ));
        }
        Ok(Self { peer_id, payload })
    }

    pub fn create(peer_id: PublicKey, bolt_8_msg_id: &[u8; 2], message: &[u8]) -> Result<Self> {
        // The payload is a vec<u8>
        // We'll first write the bolt_8_msg_id
        let mut payload = Vec::new();
        payload.extend_from_slice(bolt_8_msg_id);
        payload.extend_from_slice(message);

        let result = Self { peer_id, payload };

        Ok(result)
    }

    pub fn to_rpc(&self) -> Result<RpcCustomMsgMessage> {
        let msg = RpcCustomMsgMessage {
            peer_id: self.peer_id,
            payload: hex::encode(&self.payload),
        };

        Ok(msg)
    }

    pub fn msg(&self) -> &[u8] {
        &self.payload[2..]
    }

    pub fn bolt_8_msg_id(&self) -> [u8; 2] {
        let mut result = [0; 2];
        result[0] = self.payload[0];
        result[1] = self.payload[1];
        result
    }

    pub fn peer_id(&self) -> &PublicKey {
        &self.peer_id
    }
}
