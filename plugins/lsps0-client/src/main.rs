use anyhow::{anyhow, Context, Result};
use cln_plugin::{Builder, Error, Plugin};
use cln_rpc::ClnRpc;
use tokio;

use serde::{Deserialize, Serialize};
use serde_json::json;

use lsp_primitives::json_rpc::{JsonRpcId, JsonRpcResponse, NoParams};
use lsp_primitives::message::LSPS0_LIST_PROTOCOLS;

use cln_lsps0::client::{LspClient, PubKey, RequestId, LSPS_MESSAGE_ID};
use cln_lsps0::cln_rpc_client::ClnRpcLspClient;
use cln_lsps0::custom_msg_hook::RpcCustomMsgMessage;
use cln_lsps0::transport::RequestResponseMatcher as RRM;

type RequestResponseMatcher = RRM<RequestId, serde_json::Value>;

use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct PluginState {
    matcher: Arc<Mutex<RequestResponseMatcher>>,
}

impl PluginState {
    fn new() -> Self {
        Self {
            matcher: Arc::new(Mutex::new(RequestResponseMatcher::new())),
        }
    }
}

async fn create_lsp_client_from_plugin(plugin: Plugin<PluginState>) -> Result<impl LspClient> {
    // Create an LSPClient
    let matcher = plugin.state().matcher.clone();
    let rpc_file = plugin.configuration().rpc_file;
    let rpc = ClnRpc::new(rpc_file.clone()).await?;
    return Ok(ClnRpcLspClient::new(matcher, rpc));
}

#[tokio::main]
async fn main() -> Result<()> {
    log::info!("Configure plugin 'lsps0-client'");
    let configured_plugin =
        match Builder::<PluginState, _, _>::new(tokio::io::stdin(), tokio::io::stdout())
            .rpcmethod(
                "lsps0-list-servers",
                "List all lsps-servers that have publicly announced themselves",
                list_lsp_servers,
            )
            .rpcmethod(
                "lsps0-list-protocols",
                "List all protocols supported by an LSP-server",
                list_protocols,
            )
            .hook("custommsg", handle_custom_msg)
            .configure()
            .await?
        {
            Some(p) => p,
            None => return Ok(()),
        };

    let plugin = configured_plugin.start(PluginState::new()).await?;
    plugin.join().await?;
    return Ok(());
}

async fn handle_custom_msg(
    plugin: Plugin<PluginState>,
    notification: serde_json::Value,
) -> Result<serde_json::Value>
where
{
    log::debug!("Process incoming custom msg");
    let rpc_message = serde_json::from_value::<RpcCustomMsgMessage>(notification)?;
    let raw_message = rpc_message.to_raw()?;

    // Ignore the message if the BOLT_8_MSG id doesn't match
    // This message is not related to LSPS
    if raw_message.bolt_8_msg_id() != LSPS_MESSAGE_ID {
        return Ok(serde_json::json!({"result" : "continue"}));
    }

    // Parse the JSONRpc-Response message
    let response_msg: serde_json::Value = serde_json::from_slice(raw_message.msg())
        .with_context(|| "Failed to parse custommsg as json")?;
    let json_rpc_id = response_msg.get("id");
    let json_rpc_id = match json_rpc_id {
        None => JsonRpcId::None,
        Some(v) => serde_json::from_value(v.clone())?,
    };

    let request_id = RequestId::new(raw_message.peer_id().clone(), json_rpc_id.clone());

    // Match the message with outgoing requests
    let mut matcher = plugin.state().matcher.lock().unwrap();
    matcher.process_response(&request_id, response_msg);
    return Ok(serde_json::json!({"result" : "continue"}));
}

async fn list_lsp_servers(
    plugin: Plugin<PluginState>,
    _request: serde_json::value::Value,
) -> Result<serde_json::value::Value, Error> {
    log::debug!("Listing lsp-servers");

    // Create an LSP-client from the plugin-state
    let mut client = create_lsp_client_from_plugin(plugin).await?;

    // Get all LSP-servers from the client
    let result = client.list_lsps().await?;

    // Return the response
    Ok(serde_json::json!(result))
}

#[derive(Serialize, Deserialize)]
struct ListProtocolsRpcRequest {
    peer_id: String,
}

async fn list_protocols(
    plugin: Plugin<PluginState>,
    request: serde_json::value::Value,
) -> Result<serde_json::value::Value, Error> {
    log::debug!("Listing protocols for peer {:?}", request);

    // Create an LSP-client from the plugin-state
    let mut client = create_lsp_client_from_plugin(plugin).await?;

    // Parse the users request
    let request: ListProtocolsRpcRequest =
        serde_json::from_value(request).with_context(|| "Failed to parse RPC-request")?;
    let pubkey: PubKey = PubKey::from_hex(&request.peer_id)?;

    // Make the request to the LSP-server and return the result
    let lsp_protocol_list = client
        .request(&pubkey, LSPS0_LIST_PROTOCOLS, NoParams)
        .await?;

    match lsp_protocol_list {
        JsonRpcResponse::Ok(response) => Ok(json!(response.result)),
        JsonRpcResponse::Error(err) => Err(anyhow!("{:?}", err)),
    }
}
