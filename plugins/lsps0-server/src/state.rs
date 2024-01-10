use lsp_primitives::{lsps1, methods::Lsps1GetInfoResponse};

use crate::db::sqlite::Database;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct PluginState {
    pub(crate) database: Database, // Already uses Arc under the hood. Cheap and safe to clone
    pub(crate) lsps1_info: Arc<Option<Lsps1GetInfoResponse>>, //
}

impl PluginState {
    pub(crate) fn new(database: Database, lsps1_info: Option<Lsps1GetInfoResponse>) -> Self {
        Self {
            database,
            lsps1_info: Arc::new(lsps1_info),
        }
    }
}
