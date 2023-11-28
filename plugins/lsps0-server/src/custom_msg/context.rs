use anyhow::{Context, Result};

use cln_plugin::Plugin;
use cln_rpc::ClnRpc;

use lsp_primitives::json_rpc::JsonRpcRequest;
use lsp_primitives::lsps0::common_schemas::PublicKey;

pub struct CustomMsgContext<PluginState>
where
    PluginState: Send + Clone,
{
    pub plugin: Plugin<PluginState>,
    pub cln_rpc: ClnRpc,
    pub peer_id: PublicKey,
    pub request: JsonRpcRequest<serde_json::Value>,
    pub(crate) _private: (),
}

pub struct CustomMsgContextBuilder<PluginState>
where
    PluginState: Send + Clone,
{
    plugin: Option<Plugin<PluginState>>,
    cln_rpc: Option<ClnRpc>,
    peer_id: Option<PublicKey>,
    request: Option<JsonRpcRequest<serde_json::Value>>,
}

impl<PluginState> CustomMsgContextBuilder<PluginState>
where
    PluginState: Send + Clone,
{
    pub fn new() -> Self {
        Self {
            plugin: None,
            cln_rpc: None,
            peer_id: None,
            request: None,
        }
    }

    pub fn plugin(mut self, plugin: Plugin<PluginState>) -> Self {
        self.plugin = Some(plugin);
        self
    }

    pub fn cln_rpc(mut self, cln_rpc: ClnRpc) -> Self {
        self.cln_rpc = Some(cln_rpc);
        self
    }

    pub fn peer_id(mut self, peer_id: PublicKey) -> Self {
        self.peer_id = Some(peer_id);
        self
    }

    pub fn request(mut self, request: JsonRpcRequest<serde_json::Value>) -> Self {
        self.request = Some(request);
        self
    }

    pub fn build(self) -> Result<CustomMsgContext<PluginState>> {
        let plugin = self.plugin.context("Missing value for 'plugin'")?;
        let cln_rpc = self.cln_rpc.context("Missing value for 'cln_rpc'")?;
        let peer_id = self.peer_id.context("Missing value for 'peer_id'")?;
        let request = self.request.context("Missing value for 'request'")?;

        Ok(CustomMsgContext {
            plugin,
            cln_rpc,
            peer_id,
            request,
            _private: (),
        })
    }
}
