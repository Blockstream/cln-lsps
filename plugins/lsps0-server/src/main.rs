mod custom_msg;
mod lsps1_utils;
mod options;

use anyhow::{Context, Result};
use log;

use cln_plugin::{Builder, Plugin};

use lsp_primitives::json_rpc::{
    DefaultError, ErrorData, JsonRpcId, JsonRpcRequest, JsonRpcResponse,
};

use lsp_primitives::lsps0::builders::ListprotocolsResponseBuilder;
use lsp_primitives::methods::server as server_methods;
use lsp_primitives::methods::server::JsonRpcMethodEnum;

use cln_lsps0::client::LSPS_MESSAGE_ID;
use cln_lsps0::custom_msg_hook::RpcCustomMsgMessage;

use serde_json::json;

use crate::custom_msg::context::{CustomMsgContext, CustomMsgContextBuilder};
use crate::custom_msg::util::send_response;

#[derive(Clone)]
struct PluginState {}

impl PluginState {
    fn new() -> Self {
        Self {}
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    log::warn!("Do not use this implementation in any production context!!");
    log::warn!(
        "This implementation is developped for testing the corresponding LSP-client implementation"
    );

    let configured_plugin =
        match Builder::<PluginState, _, _>::new(tokio::io::stdin(), tokio::io::stdout())
            .option(options::lsps1_info_website())
            .option(options::lsps1_minimum_channel_confirmations())
            .option(options::lsps1_minimum_onchain_payment_confirmations())
            .option(options::lsps1_supports_zero_channel_reserve())
            .option(options::lsps1_max_channel_expiry_blocks())
            .option(options::lsps1_min_onchain_payment_size_sat())
            .option(options::lsps1_min_capacity())
            .option(options::lsps1_max_capacity())
            .option(options::lsps1_fee_computation_base_fee_sat())
            .option(options::lsps1_fee_computation_onchain_ppm())
            .option(options::lsps1_fee_computation_liquidity_ppb())
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

fn do_continue() -> Result<serde_json::Value> {
    Ok(json!({"result" : "continue"}))
}

async fn handle_custom_msg(
    plugin: Plugin<PluginState>,
    request: serde_json::Value,
) -> Result<serde_json::Value> {
    log::debug!("LSP-server received a custom-msg {:?}", request);

    // Opening the cln-rpc connection. We'll use this to send
    // custom messages
    let rpc_path = plugin.configuration().rpc_file;
    let mut cln_rpc = cln_rpc::ClnRpc::new(rpc_path).await?;

    // Parsing the customMsgHook
    // Struct of peer_id and payload
    let rpc_message = serde_json::from_value::<RpcCustomMsgMessage>(request)
        .with_context(|| "Failed to parse custom msg hook")?;
    let raw_message = rpc_message.to_raw()?;
    let peer_id = raw_message.peer_id();

    // Ignore the custom message if it is unrelated to LSPS
    // We use continue because other plug-ins might still be interested
    // in the message
    if raw_message.bolt_8_msg_id() != LSPS_MESSAGE_ID {
        return do_continue();
    }

    // In a production implementation we probably want to exclude to
    // overly long mesages here as well
    // TODO: Check message length

    // We'll expect that all incoming messages are JSON-RPC Requests.
    // Here we parse the JSON and receive a `serde_json::Value`-struct
    // This will succeed as long as the `raw_message` is valid json
    // If it isn't we'll respond to our peer with a parse_error
    let json_msg: Result<serde_json::Value, _> = serde_json::from_slice(raw_message.msg());
    let json_msg = match json_msg {
        Ok(ok) => ok,
        Err(_) => {
            let error = ErrorData::parse_error(format!("Invalid JSON"));
            let rpc_response = JsonRpcResponse::<(), DefaultError>::error(JsonRpcId::None, error);
            send_response(&mut cln_rpc, peer_id.clone(), rpc_response).await?;
            return do_continue();
        }
    };

    // Let's try to read id of the JSON-rpc request
    // If the json doesn't include an id, we'll respond to the peer and tell
    // them they send an invalid message.
    let id: Option<&serde_json::Value> = json_msg.get("id");
    let id = match id {
        Some(value) => serde_json::from_value::<JsonRpcId>(value.clone()).unwrap(),
        None => {
            let error = ErrorData::invalid_request(format!("Missing field `id`"));
            let rpc_response = JsonRpcResponse::<(), DefaultError>::error(JsonRpcId::None, error);
            send_response(&mut cln_rpc, peer_id.clone(), rpc_response).await?;
            return do_continue();
        }
    };

    // We'll parse to `JsonRpcRequest<serde_json::Value>`.
    // Here we ensure it is a valid JsonRpcRequest. (id-field, jsonrpc="2.0")
    // However, we don't parse the params yet
    let json_rpc_request = serde_json::from_value::<JsonRpcRequest<serde_json::Value>>(json_msg);
    let json_rpc_request = match json_rpc_request {
        Ok(request) => request,
        Err(parse_error) => {
            let error =
                ErrorData::invalid_request(format!("Invalid JSON-RPC request. {}", parse_error));
            let rpc_response = JsonRpcResponse::<(), DefaultError>::error(id.clone(), error);
            send_response(&mut cln_rpc, peer_id.clone(), rpc_response).await?;
            return do_continue();
        }
    };

    // Let's check if we know the method in the JSON-rpc request.
    // We return a user to the error of the JsonRpcMethod isn't known
    let method_str = json_rpc_request.method.clone();
    let method = JsonRpcMethodEnum::from_method_name(&method_str);
    let method = match method {
        Ok(m) => m,
        Err(_) => {
            let error = ErrorData::unknown_method(format!("Method unknown '{}'", method_str));
            let rpc_response = JsonRpcResponse::<(), DefaultError>::error(id.clone(), error);
            send_response(&mut cln_rpc, peer_id.clone(), rpc_response).await?;
            return do_continue();
        }
    };

    // Execute the handler for the specific method
    // Each handler accepts a `CustomMsgContext` containing all relevant information
    // Each handler returns a `Result<serde_json::Value>`. The handler should return
    // {"result" : "continue" }
    let mut context = CustomMsgContextBuilder::new()
        .request(json_rpc_request)
        .plugin(plugin)
        .peer_id(peer_id.clone())
        .cln_rpc(cln_rpc)
        .build()?;

    return match method {
        JsonRpcMethodEnum::Lsps0ListProtocols(m) => do_list_protocols(m, context).await,
        JsonRpcMethodEnum::Lsps1Info(m) => do_lsps1_get_info(m, context).await,
        _ => {
            let error = ErrorData::unknown_method(format!("Method unknown '{}'", method_str));
            let rpc_response = JsonRpcResponse::<(), DefaultError>::error(id.clone(), error);
            send_response(&mut context.cln_rpc, peer_id.clone(), rpc_response).await?;
            return do_continue();
        }
    };
}

async fn do_list_protocols(
    _method: server_methods::Lsps0ListProtocols,
    mut context: CustomMsgContext<PluginState>,
) -> Result<serde_json::Value> {
    // Opening the cln-rpc connection. We'll use this to send
    // custom messages
    // Create the response object
    let protocol_list = ListprotocolsResponseBuilder::new()
        .protocols(vec![0, 1])
        .build();

    let response = match protocol_list {
        Ok(pl) => JsonRpcResponse::success(context.request.id, pl),
        Err(err) => {
            log::warn!("Error in handling lsps1.list_protocols call");
            log::warn!("Error {:?}", err);
            let error = ErrorData::<DefaultError>::internal_error("Internal server error".into());
            JsonRpcResponse::error(context.request.id, error)
        }
    };

    send_response(&mut context.cln_rpc, context.peer_id, response).await?;
    do_continue()
}

async fn do_lsps1_get_info(
    _method: server_methods::Lsps1Info,
    mut context: CustomMsgContext<PluginState>,
) -> Result<serde_json::Value> {
    log::debug!("lsps1_get_info");

    let info_response = lsps1_utils::info_response(context.plugin.options());

    let response = match info_response {
        Ok(ok) => JsonRpcResponse::success(context.request.id, ok),
        Err(err) => {
            log::warn!("Error in handling lsps1.list_protocols call");
            log::warn!("Error {:?}", err);
            let error = ErrorData::<DefaultError>::internal_error("Internal server error".into());
            JsonRpcResponse::error(context.request.id, error)
        }
    };

    send_response(&mut context.cln_rpc, context.peer_id, response).await?;
    do_continue()
}
