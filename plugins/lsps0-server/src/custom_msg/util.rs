use anyhow::Result;

use cln_rpc::model::requests::SendcustommsgRequest;
use cln_rpc::ClnRpc;

use cln_lsps0::client::LSPS_MESSAGE_ID;
use cln_lsps0::custom_msg_hook::RawCustomMsgMessage;
use lsp_primitives::json_rpc::JsonRpcResponse;
use lsp_primitives::lsps0::common_schemas::PublicKey;

use serde::Serialize;

pub async fn send_response<O, E>(
    cln_rpc: &mut ClnRpc,
    peer_id: PublicKey,
    response: JsonRpcResponse<O, E>,
) -> Result<()>
where
    O: Serialize,
    E: Serialize,
{
    let data: Vec<u8> = serde_json::to_vec(&response)?;
    let bolt8_msg_id = LSPS_MESSAGE_ID;
    let raw_msg = RawCustomMsgMessage::create(peer_id.clone(), &bolt8_msg_id, &data)?;
    let rpc_msg = raw_msg.to_rpc()?;
    log::debug!("Sending response to peer={:?} data={}", rpc_msg.peer_id, rpc_msg.payload);

    let send_custom_msg_request = SendcustommsgRequest {
        node_id: cln_rpc::primitives::PublicKey::from_slice(&rpc_msg.peer_id.inner().serialize())?,
        msg: rpc_msg.payload,
    };

    let _result = cln_rpc.call_typed(&send_custom_msg_request).await?;
    Ok(())
}
