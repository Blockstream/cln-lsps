use anyhow::{Context, Result};
use uuid::Uuid;

use sqlx::{Sqlite, Transaction};

use crate::db::schema::Lsps1Order;
use crate::db::sqlite::schema::Lsps1Order as Lsps1OrderSqlite;

pub struct GetOrderQuery {
    pub(crate) order_id: Uuid,
}

impl GetOrderQuery {
    pub fn by_uuid(order_id: Uuid) -> Self {
        Self { order_id }
    }
}

impl GetOrderQuery {
    pub async fn execute<'b>(
        &self,
        tx: &'b mut Transaction<'static, Sqlite>,
    ) -> Result<Option<Lsps1Order>> {
        let uuid_string = self.order_id.to_string();
        let result = sqlx::query_as!(
            Lsps1OrderSqlite,
            r#"SELECT
                uuid, client_node_id, lsp_balance_sat,
                client_balance_sat, confirms_within_blocks, channel_expiry_blocks,
                token, refund_onchain_address, announce_channel,
                ord.created_at, expires_at, os.order_state_enum_id as order_state,
                generation
            FROM lsps1_order AS ord
            JOIN lsps1_order_state AS os ON ord.id = os.order_id
            WHERE uuid = ?
            ORDER BY os.generation
            LIMIT 1;"#,
            uuid_string
        )
        .fetch_optional(&mut **tx)
        .await
        .context("Failed to execute query")?;

        match result {
            Some(r) => Ok(Some(Lsps1Order::try_from(&r)?)),
            None => Ok(None),
        }
    }
}
