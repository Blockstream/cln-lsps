use crate::db::sqlite::Database;

#[derive(Clone)]
pub(crate) struct PluginState {
    pub(crate) database: Database, // Already uses Arc under the hood. Cheap and safe to clone
}

impl PluginState {
    pub(crate) fn new(database: Database) -> Self {
        Self { database }
    }
}
