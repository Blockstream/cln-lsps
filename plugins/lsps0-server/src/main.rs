mod lsps1_utils;
mod options;

use anyhow::{Context, Result};
use log;

use cln_rpc::model::requests::SendcustommsgRequest;
use cln_rpc::ClnRpc;

use cln_plugin::{Builder, Plugin};

use lsp_primitives::json_rpc::{
    DefaultError, ErrorData, JsonRpcId, JsonRpcRequest, JsonRpcResponse,
};

use lsp_primitives::lsps0::builders::ListprotocolsResponseBuilder;
use lsp_primitives::lsps0::schema::PublicKey;
use lsp_primitives::methods::server::JsonRpcMethodEnum;

use cln_lsps0::client::LSPS_MESSAGE_ID;
use cln_lsps0::custom_msg_hook::{RawCustomMsgMessage, RpcCustomMsgMessage};

use serde::Serialize;
use serde_json::json;

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

struct CustomMsgResponder {
    rpc: ClnRpc,
    peer_id: PublicKey,
}

impl CustomMsgResponder {
    fn new(rpc: ClnRpc, peer_id: PublicKey) -> Self {
        Self { rpc, peer_id }
    }

    async fn send_response<O, E>(&mut self, response: JsonRpcResponse<O, E>) -> Result<()>
    where
        O: Serialize,
        E: Serialize,
    {
        let data: Vec<u8> = serde_json::to_vec(&response)?;
        let bolt8_msg_id = LSPS_MESSAGE_ID;
        let raw_msg = RawCustomMsgMessage::create(self.peer_id.clone(), &bolt8_msg_id, &data)?;
        let rpc_msg = raw_msg.to_rpc()?;

        let send_custom_msg_request = SendcustommsgRequest {
            node_id: cln_rpc::primitives::PublicKey::from_slice(
                &rpc_msg.peer_id.inner().serialize(),
            )?,
            msg: rpc_msg.payload,
        };

        let _result = self.rpc.call_typed(send_custom_msg_request).await?;
        Ok(())
    }
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
    let cln_rpc = cln_rpc::ClnRpc::new(rpc_path).await?;

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

    let mut custom_msg_sender = CustomMsgResponder::new(cln_rpc, peer_id.clone());

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
            custom_msg_sender.send_response(rpc_response).await?;
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
            custom_msg_sender.send_response(rpc_response).await?;
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
            custom_msg_sender.send_response(rpc_response).await?;
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
            custom_msg_sender.send_response(rpc_response).await?;
            return do_continue();
        }
    };

    // Execute the handler for the specific method
    // A handler has a `anyhow::Result<()>` return-type.
    //
    // If an error occurs we'll log it and continue.
    let result: Result<()> = match method {
        JsonRpcMethodEnum::Lsps0ListProtocols(_) => {
            do_list_protocols(plugin, peer_id.clone(), json_rpc_request).await
        }
        JsonRpcMethodEnum::Lsps1Info(_) => {
            do_lsps1_get_info(plugin, peer_id.clone(), json_rpc_request).await
        }
        _ => {
            let error = ErrorData::unknown_method(format!("Method unknown '{}'", method_str));
            let rpc_response = JsonRpcResponse::<(), DefaultError>::error(id.clone(), error);
            custom_msg_sender.send_response(rpc_response).await?;
            return do_continue();
        }
    };

    match result {
        Err(err) => log::warn!("Error in processing '{}'-request: {}", method_str, err),
        Ok(_) => {}
    }

    return do_continue();
}

async fn do_list_protocols(
    plugin: Plugin<PluginState>,
    peer_id: PublicKey,
    request: JsonRpcRequest<serde_json::Value>,
) -> Result<()> {
    // Opening the cln-rpc connection. We'll use this to send
    // custom messages
    let rpc_path = plugin.configuration().rpc_file;
    let cln_rpc = cln_rpc::ClnRpc::new(rpc_path).await?;
    let mut custom_msg_sender = CustomMsgResponder::new(cln_rpc, peer_id);

    // Create the response object
    let protocol_list = ListprotocolsResponseBuilder::new()
        .protocols(vec![0, 1])
        .build();

    let response = match protocol_list {
        Ok(pl) => JsonRpcResponse::success(request.id, pl),
        Err(err) => {
            log::warn!("Error in handling lsps1.list_protocols call");
            log::warn!("Error {:?}", err);
            let error = ErrorData::<DefaultError>::internal_error("Internal server error".into());
            JsonRpcResponse::error(request.id, error)
        }
    };

    custom_msg_sender.send_response(response).await?;
    return Ok(());
}

async fn do_lsps1_get_info(
    plugin: Plugin<PluginState>,
    peer_id: PublicKey,
    request: JsonRpcRequest<serde_json::Value>,
) -> Result<()> {
    log::debug!("lsps1_get_info");

    // Opening the cln-rpc connection. We'll use this to send
    // custom messages
    let rpc_path = plugin.configuration().rpc_file;
    let cln_rpc = cln_rpc::ClnRpc::new(rpc_path).await?;
    let mut custom_msg_sender = CustomMsgResponder::new(cln_rpc, peer_id);

    let info_response = lsps1_utils::info_response(plugin.options());

    let response = match info_response {
        Ok(ok) => JsonRpcResponse::success(request.id, ok),
        Err(err) => {
            log::warn!("Error in handling lsps1.list_protocols call");
            log::warn!("Error {:?}", err);
            let error = ErrorData::<DefaultError>::internal_error("Internal server error".into());
            JsonRpcResponse::error(request.id, error)
        }
    };

    custom_msg_sender.send_response(response).await?;
    Ok(())
}
