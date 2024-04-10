mod channel_open;
mod cln;
mod custom_msg;
mod db;
mod lsps1;
mod network;
mod options;
mod state;

use std::str::FromStr;

use anyhow::{Context, Result};
use log;

use cln_plugin::{Builder, FeatureBitsKind, Plugin};

use lsp_primitives::json_rpc::{
    DefaultError, ErrorData, JsonRpcId, JsonRpcRequest, JsonRpcResponse,
};

use lsp_primitives::lsps0::builders::ListprotocolsResponseBuilder;
use lsp_primitives::lsps0::schema::ListprotocolsResponse;
use lsp_primitives::methods;
use lsp_primitives::methods::JsonRpcMethodEnum;

use cln_lsps::client::{LSPS_MESSAGE_ID, LSPS_MESSAGE_ID_U16};
use cln_lsps::custom_msg_hook::RpcCustomMsgMessage;

use serde_json::json;

use crate::custom_msg::context::{CustomMsgContext, CustomMsgContextBuilder};
use crate::custom_msg::util::send_response;

use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection};
use sqlx::Connection;

use crate::cln::hooks::invoice_payment::{InvoicePaymentHookData, InvoicePaymentHookResponse};
use crate::db::sqlite::Database;
use crate::lsps1::hooks::{
    do_lsps1_create_order, do_lsps1_get_info, do_lsps1_get_order,
    invoice_payment as lsps1_invoice_payment,
};
use crate::network::parse_network;
use crate::state::PluginState;

const FEATURE_BIT_STRING : & str = "0200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

#[tokio::main]
async fn main() -> Result<()> {
    let configured_plugin =
        match Builder::<PluginState, _, _>::new(tokio::io::stdin(), tokio::io::stdout())
            .option(options::lsp_server_database_url())
            .option(options::lsps1_enable())
            .option(options::lsps1_info_website())
            .option(options::lsps1_min_required_channel_confirmations())
            .option(options::lsps1_min_onchain_payment_confirmations())
            .option(options::lsps1_min_funding_confirms_within_blocks())
            .option(options::lsps1_supports_zero_channel_reserve())
            .option(options::lsps1_max_channel_expiry_blocks())
            .option(options::lsps1_min_onchain_payment_size_sat())
            .option(options::lsps1_fee_computation_base_fee_sat())
            .option(options::lsps1_fee_computation_onchain_ppm())
            .option(options::lsps1_fee_computation_liquidity_ppb())
            .option(options::lsps1_order_lifetime_seconds())
            .option(options::lsps1_min_initial_client_balance_sat())
            .option(options::lsps1_max_initial_client_balance_sat())
            .option(options::lsps1_min_initial_lsp_balance_sat())
            .option(options::lsps1_max_initial_lsp_balance_sat())
            .option(options::lsps1_min_channel_balance_sat())
            .option(options::lsps1_max_channel_balance_sat())
            .custommessages(vec![LSPS_MESSAGE_ID_U16])
            .hook("custommsg", handle_custom_msg)
            .hook("invoice_payment", handle_paid_invoice)
            .featurebits(FeatureBitsKind::Node, String::from(FEATURE_BIT_STRING))
            .featurebits(FeatureBitsKind::Init, String::from(FEATURE_BIT_STRING))
            .configure()
            .await?
        {
            Some(p) => p,
            None => return Ok(()),
        };

    log::warn!("Do not use this implementation in any production context!!");
    log::warn!(
        "This implementation is developped for testing the corresponding LSP-client implementation"
    );

    let lsps1_info = match crate::lsps1::state::get_state(&configured_plugin) {
        Ok(info) => {
            log::info!("{:?}", info);
            info
        }
        Err(err) => {
            log::warn!("Failed to read configuration for LSPS1");
            configured_plugin
                .disable(&format!("Invalid configuration: {}", err))
                .await?;
            return Err(err);
        }
    };

    // Connect to the database and run migration scripts
    let connection_string: String =
        match configured_plugin.option(&options::lsp_server_database_url()) {
            Ok(None) => {
                log::info!("No config found for 'lsp_server_database_url'");
                let lightning_dir = configured_plugin.configuration().lightning_dir;
                let connection_string = format!("sqlite://{}/lsp_server.db", lightning_dir);
                log::info!("Using default: '{}'", connection_string);
                connection_string
            }
            Ok(Some(connection_string)) => connection_string, // User configured a database
            Err(_) => panic!(
                "Failed to read config variable {}",
                options::lsp_server_database_url().name
            ),
        };

    let options = SqliteConnectOptions::from_str(&connection_string)?.create_if_missing(true);
    let mut connection = SqliteConnection::connect_with(&options).await.unwrap();
    log::info!("Running database migration scripts");
    sqlx::migrate!().run(&mut connection).await?;
    log::info!("Successfully executed migrations");
    log::warn!("Trying database connection");

    let database = Database::connect_with_options(options).await?;

    let plugin = configured_plugin
        .start(PluginState::new(database, lsps1_info))
        .await?;

    plugin.join().await.unwrap();

    return Ok(());
}

fn do_continue() -> Result<serde_json::Value> {
    Ok(json!({"result" : "continue"}))
}

/// Handles an incoming custom message
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

    // BOLT-8 messages are already limited in length.
    // By consequence, we don't need to check the length of these messages

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

    // Let's try to read the id of the JSON-rpc request
    // If the json doesn't include an id, we'll respond to the peer and tell
    // them they've sent an invalid message.
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
            log::debug!(
                "Invalid rpc-method '{}' from peer '{:?}'",
                method_str,
                &peer_id
            );
            let error = ErrorData::method_not_found(&method_str);
            let rpc_response = JsonRpcResponse::<(), DefaultError>::error(id.clone(), error);
            send_response(&mut cln_rpc, peer_id.clone(), rpc_response).await?;
            return do_continue();
        }
    };

    let network = parse_network(&plugin.configuration().network).unwrap();

    let mut context = CustomMsgContextBuilder::new()
        .network(network)
        .request(json_rpc_request)
        .plugin(plugin)
        .peer_id(peer_id.clone())
        .cln_rpc(cln_rpc)
        .build()?;

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
                log::debug!("Reason {:?}", error_data);
            }
        }
    };

    do_continue()
}

/// Hook method for a paid invoice
async fn handle_paid_invoice(
    plugin: Plugin<PluginState>,
    value: serde_json::Value,
) -> Result<serde_json::Value> {
    // Parse the incoming hook data
    // If we fail we log an error and continue
    let parsed = serde_json::from_value::<InvoicePaymentHookData>(value);
    let hook_data = if let Ok(hook_data) = parsed {
        hook_data
    } else {
        log::warn!("Error in parsing payment_hook");
        return Ok(InvoicePaymentHookResponse::Continue.serialize());
    };

    // Handle the uinderlying plugin
    let result = lsps1_invoice_payment(plugin, &hook_data.payment).await;
    match result {
        Ok(hook_response) => Ok(hook_response.serialize()),
        Err(err) => {
            log::warn!("Error in processing payment_hook");
            log::warn!("{:?}", err);
            Ok(InvoicePaymentHookResponse::Continue.serialize())
        }
    }
}

async fn do_list_protocols(
    method: methods::Lsps0ListProtocols,
    context: &mut CustomMsgContext<PluginState>,
) -> Result<ListprotocolsResponse, ErrorData> {
    method.into_typed_request(context.request.clone())?;

    let lsps1_enabled = context.plugin.option(&options::lsps1_enable()).unwrap();

    let protocols = if lsps1_enabled { vec![0, 1] } else { vec![0] };

    ListprotocolsResponseBuilder::new()
        .protocols(protocols)
        .build()
        .map_err(ErrorData::internalize)
}
