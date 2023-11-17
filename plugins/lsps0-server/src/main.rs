use anyhow::{Context, Result};
use log;
use std::str::FromStr;

use cln_rpc::model::requests::SendcustommsgRequest;
use cln_rpc::ClnRpc;

use cln_plugin::{Builder, Plugin};

use lsp_primitives::json_rpc::{
    DefaultError, ErrorData, JsonRpcId, JsonRpcRequest, JsonRpcResponse, NoParams,
};
use lsp_primitives::lsps0::schema::ListprotocolsResponse;
use lsp_primitives::message::JsonRpcMethodEnum;

use cln_lsps0::client::{PubKey, LSPS_MESSAGE_ID};
use cln_lsps0::custom_msg_hook::{RawCustomMsgMessage, RpcCustomMsgMessage};

use serde::Serialize;
use serde_json::json;

#[derive(Clone)]
struct PluginState;

impl PluginState {
    fn new() -> Self {
        Self
    }
}

#[derive(Debug)]
enum PluginError {
    Error(anyhow::Error),
    Continue,
}

impl From<anyhow::Error> for PluginError {
    fn from(err: anyhow::Error) -> Self {
        PluginError::Error(err)
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
    peer_id: PubKey,
}

impl CustomMsgResponder {
    fn new(rpc: ClnRpc, peer_id: PubKey) -> Self {
        Self { rpc, peer_id }
    }

    async fn send_custom_msg<S>(&mut self, msg: S) -> Result<()>
    where
        S: Serialize,
    {
        let data: Vec<u8> = serde_json::to_vec(&msg)?;
        let bolt8_msg_id = LSPS_MESSAGE_ID;
        let raw_msg = RawCustomMsgMessage::create(self.peer_id.clone(), &bolt8_msg_id, &data)?;
        let rpc_msg = raw_msg.to_rpc()?;

        let send_custom_msg_request = SendcustommsgRequest {
            node_id: cln_rpc::primitives::PublicKey::from_str(&rpc_msg.peer_id)?,
            msg: rpc_msg.payload,
        };

        let _result = self.rpc.call_typed(send_custom_msg_request).await?;
        Ok(())
    }

    async fn respond_and_continue_on_err<I, E, F>(
        &mut self,
        result: Result<I, E>,
        id: JsonRpcId,
        f: F,
    ) -> Result<I, PluginError>
    where
        F: FnOnce(E) -> ErrorData<DefaultError>,
    {
        match result {
            Ok(v) => Ok(v),
            Err(e) => {
                let error_data = f(e);
                let response = error_data.into_response::<()>(id);
                self.send_custom_msg(response).await?;
                return Err(PluginError::Continue);
            }
        }
    }
}

async fn handle_custom_msg_inner(
    plugin: Plugin<PluginState>,
    request: serde_json::Value,
) -> Result<(), PluginError> {
    log::info!("Handle custom msg inner");

    // Opening the cln-rpc connection. We'll use this to send
    // custom messages
    let rpc_path = plugin.configuration().rpc_file;
    let cln_rpc = cln_rpc::ClnRpc::new(rpc_path).await?;

    // Parsing the customMsgHook
    // Struct of peer_id and payload
    let rpc_message = serde_json::from_value::<RpcCustomMsgMessage>(request)
        .with_context(|| "Failed to parse custom msg hook")?;
    let raw_message = rpc_message.to_raw()?;

    // Ignore the custom message if it is unrelated to LSPS
    if raw_message.bolt_8_msg_id() != LSPS_MESSAGE_ID {
        return Err(PluginError::Continue);
    }

    let mut custom_msg_sender = CustomMsgResponder::new(cln_rpc, raw_message.peer_id().clone());

    // In a production implementation we probably want to exclude to
    // overly long mesages here as well

    // We'll expect that all incoming messages are JSON-RPC Requests.
    // We parse it to `serde_json::Value`. We'll continue parsing as long as it is valid json
    // If invalid, we respond with a parse_error
    log::info!("Parsing custom msg");
    let json_msg: Result<serde_json::Value, _> = serde_json::from_slice(raw_message.msg());
    let json_msg = custom_msg_sender
        .respond_and_continue_on_err(json_msg, JsonRpcId::None, |_| {
            ErrorData::parse_error("Failed to parse json".to_string())
        })
        .await?;

    // Let's extract the message id from the msg
    log::info!("Extracting msg id");
    let id: Option<&serde_json::Value> = json_msg.get("id");
    let id = match id {
        // TODO: Not nice in production
        // We should respond
        Some(value) => serde_json::from_value(value.clone()).unwrap(),
        None => JsonRpcId::None,
    };

    // Let's go one step further.
    // We'll parse to `JsonRpcRequest<serde_json::Value>`.
    // This works as long as the JsonRpcRequest is a valid JsonRpcRequest
    log::info!("Parsing JsonRpcRequest<serde_json::Value>");
    let json_rpc_request = serde_json::from_value::<JsonRpcRequest<serde_json::Value>>(json_msg);
    log::warn!("This makes things go wrong {:?}", json_rpc_request);
    let json_rpc_request = custom_msg_sender
        .respond_and_continue_on_err(json_rpc_request, id.clone(), |e| {
            ErrorData::invalid_request(format!("Invalid json rpc request. {}", e))
        })
        .await?;

    // Let's check if we know the method in the JSON-rpc Request
    // We return a user to the error of the JsonRpcMethod isn't known
    log::info!("Checking the method");
    let method = JsonRpcMethodEnum::from_method_name(&json_rpc_request.method);
    let method = custom_msg_sender
        .respond_and_continue_on_err(method, id.clone(), |e| {
            ErrorData::unknown_method(format!("Unknown method '{}'", e))
        })
        .await?;

    // Parse the params and return invalid_params if it doesn't work
    log::info!("Parsing in detail");
    match method {
        JsonRpcMethodEnum::Lsps0ListProtocols(m) => {
            let request = m.into_typed_request(json_rpc_request);
            let request = custom_msg_sender
                .respond_and_continue_on_err(request, id.clone(), |e| {
                    ErrorData::invalid_params(format!("invalid params: '{}'", e))
                })
                .await?;

            // Send the response back to the client
            let response = list_protocols(request);
            custom_msg_sender.send_custom_msg(response).await?;
            log::info!("Continue happily");
            return Err(PluginError::Continue);
        }
        _ => {
            let response = ErrorData::unknown_method(format!("Unknown method '{}'", method.name()))
                .into_response::<()>(id);
            custom_msg_sender.send_custom_msg(response).await?;
            return Err(PluginError::Continue);
        }
    }
}

async fn handle_custom_msg(
    plugin: Plugin<PluginState>,
    request: serde_json::Value,
) -> Result<serde_json::Value> {
    log::info!("Handling custom msg {:?}", request);

    let result = handle_custom_msg_inner(plugin, request).await;

    log::info!("{:?}", result);
    return match result {
        Ok(_) => Ok(json!({"result" : "continue"})),
        Err(PluginError::Continue) => Ok(json!({"result" : "continue"})),
        Err(PluginError::Break) => Ok(json!({"result" : "break"})),
        Err(PluginError::Error(err)) => Err(err),
    };
}

fn list_protocols(
    request: JsonRpcRequest<NoParams>,
) -> JsonRpcResponse<ListprotocolsResponse, DefaultError> {
    log::info!("ListProtocols");
    let response = ListprotocolsResponse::new(vec![0, 1]);
    JsonRpcResponse::success(request.id, response)
}
