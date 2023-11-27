mod plugin_rpc;

use anyhow::{anyhow, Context, Result};
use cln_plugin::{Builder, Error, Plugin};
use cln_rpc::ClnRpc;
use tokio;

use serde::{Deserialize, Serialize};
use serde_json::json;

use lsp_primitives::json_rpc::{JsonRpcId, JsonRpcResponse, NoParams};
use lsp_primitives::lsps0::common_schemas::{
    Network, NetworkCheckable, NetworkUnchecked, PublicKey,
};
use lsp_primitives::lsps1;
use lsp_primitives::methods::client as client_methods;

use cln_lsps0::client::{LspClient, RequestId, LSPS_MESSAGE_ID};
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
            .rpcmethod(
                "lsps1-get-info",
                "Get info and pricing to purchase a channel from an LSP",
                lsps0_get_info,
            )
            .rpcmethod(
                "lsps1-create-order",
                "Order a channel from an LSP",
                lsps1_create_order,
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
    log::debug!("Created client");

    // Parse the users request
    let request: plugin_rpc::ListProtocolsRequest =
        serde_json::from_value(request).with_context(|| "Failed to parse RPC-request")?;
    log::debug!("plugin_rpc_request created {:?}", request);
    let pubkey: PublicKey = PublicKey::from_hex(&request.peer_id)?;

    // Make the request to the LSP-server and return the result
    let lsp_protocol_list = client
        .request(&pubkey, client_methods::LSPS0_LIST_PROTOCOLS, NoParams)
        .await?;
    log::debug!("ProtocolList Request {:?}", lsp_protocol_list);

    match lsp_protocol_list {
        JsonRpcResponse::Ok(response) => Ok(json!(response.result)),
        JsonRpcResponse::Error(err) => Err(anyhow!("{:?}", err)),
    }
}

async fn lsps0_get_info(
    plugin: Plugin<PluginState>,
    request: serde_json::Value,
) -> Result<serde_json::Value, Error> {
    // Create an LSP-client from the plugin-state
    let mut client = create_lsp_client_from_plugin(plugin).await?;

    let request: plugin_rpc::Lsps1GetInfoRequest = serde_json::from_value(request)?;
    let pubkey = PublicKey::from_hex(&request.peer_id)?;

    // Make the request to the LSP-server and return the result
    let response = client
        .request(
            &pubkey,
            client_methods::LSPS1_GETINFO,
            lsps1::schema::Lsps1InfoRequest {},
        )
        .await?;

    match response {
        JsonRpcResponse::Ok(response) => Ok(json!(response.result)),
        JsonRpcResponse::Error(err) => Err(anyhow!("{:?}", err)),
    }
}

async fn lsps1_create_order(
    plugin: Plugin<PluginState>,
    request: serde_json::Value,
) -> Result<serde_json::Value, Error> {
    let network = str_to_network(&plugin.configuration().network)?;
    let mut client = create_lsp_client_from_plugin(plugin).await?;

    let request: plugin_rpc::Lsps1CreateOrderRequest<NetworkUnchecked> =
        serde_json::from_value(request)?;
    let request = request.require_network(network)?;
    let pubkey = PublicKey::from_hex(&request.peer_id)?;

    let create_order_request = lsps1::builders::Lsps1CreateOrderRequestBuilder::new()
        .api_version(1)
        .lsp_balance_sat(request.lsp_balance_sat)
        .client_balance_sat(request.client_balance_sat)
        .confirms_within_blocks(request.confirms_within_blocks)
        .channel_expiry_blocks(request.channel_expiry_blocks)
        .token(request.token)
        .refund_onchain_address(request.refund_onchain_address)
        .announce_channel(request.announce_channel)
        .build()
        .unwrap();

    // Make the request to the LSP-server and return the result
    let response = client
        .request(
            &pubkey,
            client_methods::LSPS1_CREATE_ORDER,
            create_order_request,
        )
        .await?;

    match response {
        JsonRpcResponse::Ok(ok) => return Ok(json!(ok.result)),
        JsonRpcResponse::Error(err) => {
            return Err(anyhow!("{} : {}", err.error.code, err.error.message))
        }
    }
}

fn str_to_network(network: &str) -> Result<Network> {
    match network {
        "bitcoin" => Ok(Network::Bitcoin),
        "regtest" => Ok(Network::Regtest),
        "signet" => Ok(Network::Signet),
        "testnet" => Ok(Network::Testnet),
        _ => Err(anyhow!("Unknown network '{}", network)),
    }
}
