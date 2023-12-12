/// LSPS uses a JSON-rpc based communication schema
/// on top of the BOLT8 custom msg's.
/// 
/// This is the implementation of the boilerplate for the server
///
///
use anyhow::{Context, Result};
use cln_plugin::Plugin;

use lsp_primitives::json_rpc::{JsonRpcRequest, JsonRpcResponse, JsonRpcResponseFailure, DefaultError};

use crate::state::PluginState;

/// The hook that is attached to core-lightning
/// This hook ensures that
/// - any request is logged
/// - a reply is sent for every request
/// - {"result" : "continue"} is always returned to core Lightning
/// 
pub(crate) async fn handle_custom_msg(
    plugin : Plugin<PluginState>,
    request : serde_json::Value) -> Result<serde_json::Value> {

    // Opening the cln-rpc connection. We'll use this to send
    // custom messages
    let rpc_path = plugin.configuration().rpc_file;
    let mut cln_rpc = cln_rpc::ClnRpc::new(rpc_path).await?;


 
    let parse_result = parse_custom_msg(request);
    match parse_result {

    }

    Ok(serde_json::json!({"result" : "do_continue"}))
}


/// Reply if the client doesn't provide a valid
/// JSON-rpc request.
///
async fn parse_custom_msg(
    request : serde_json::Value
) -> Result<JsonRpcRequest<serde_json::Value>, JsonRpcResponseFailure<DefaultError>>
{
    todo!("Implement");

}

async fn handle_custom_msg2(
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
            let error = ErrorData::unknown_method(&method_str);
            let rpc_response = JsonRpcResponse::<(), DefaultError>::error(id.clone(), error);
            send_response(&mut cln_rpc, peer_id.clone(), rpc_response).await?;
            return do_continue();
        }
    };

    // TODO: Send a nicer response
    let network = parse_network(&plugin.configuration().network).unwrap();

    // Execute the handler for the specific method
    // Each handler accepts a `CustomMsgContext` containing all relevant information
    // Each handler returns a `Result<serde_json::Value>`. The handler should return
    // {"result" : "continue" }
    let mut context = CustomMsgContextBuilder::new()
        .network(network)
        .request(json_rpc_request)
        .plugin(plugin)
        .peer_id(peer_id.clone())
        .cln_rpc(cln_rpc)
        .build()?;

    // Process the incoming custom msg
    // TODO: FInd a way to handle the errors which isn't super verbose
    type JRM = JsonRpcMethodEnum;
    let result = match method {
        JRM::Lsps0ListProtocols(m) => do_list_protocols(m, &mut context)
            .await
            .map(|x| serde_json::to_value(x).unwrap()),
        JRM::Lsps1Info(m) => do_lsps1_get_info(m, &mut context)
            .await
            .map(|x| serde_json::to_value(x).unwrap()),
        JRM::Lsps1CreateOrder(m) => do_lsps1_create_order(m, &mut context)
            .await
            .map(|x| serde_json::to_value(x).unwrap()),
        JRM::Lsps1GetOrder(m) => do_lsps1_get_order(m, &mut context)
            .await
            .map(|x| serde_json::to_value(x).unwrap()),
    };

    match result {
        Ok(result) => {
            let json_rpc_response = JsonRpcResponse::<_, DefaultError>::success(id, result);
            send_response(&mut context.cln_rpc, *peer_id, json_rpc_response).await?;
        }
        Err(err) => {
            log::warn!("Error {:?}", err);
            let error_data = ErrorData::try_from(err);
            if error_data.is_ok() {
                let json_rpc_response =
                    JsonRpcResponse::<(), DefaultError>::error(id, error_data.unwrap());
                send_response(&mut context.cln_rpc, *peer_id, json_rpc_response).await?;
            } else {
                log::debug!("Ignored message {:?}.{:?}", peer_id, id);
                log::debug!("Reason {:?}", error_data.unwrap_err());
            }
        }
    };

    do_continue()
}


#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn serialize_custom_msg_response() {
        let value = serde_json::to_value(CustomMsgResponse::Continue).unwrap();
        assert_eq!(value, serde_json::json!({"result" : "continue"}));
    }
}
