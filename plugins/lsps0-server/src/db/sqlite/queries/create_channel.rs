use anyhow::{anyhow, Result};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

pub struct CreateChannelQuery {
    pub(crate) order_id: Uuid,
    pub(crate) channel_id: String,
}

impl CreateChannelQuery {
    pub fn new(order_id: Uuid, channel_id: String) -> Self {
        Self {
            order_id,
            channel_id,
        }
    }
}

impl CreateChannelQuery {
    pub(crate) async fn execute(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<()> {
        let uuid_str = self.order_id.to_string();
        let result = sqlx::query!(
            r#"
             INSERT INTO lsps1_channel (order_id, channel_id)
             SELECT o.id, ?2 FROM lsps1_order as o
             where o.uuid = ?1;
             "#,
            uuid_str,
            self.channel_id
        )
        .execute(&mut **tx)
        .await?;

        match result.rows_affected() {
            0 => Err(anyhow!(
                "Failed to find order '{}' and could not create channel",
                self.order_id.to_string()
            )),
            1 => Ok(()),
            _ => Err(anyhow!(
                "Error in updating state. Query affected {} rows",
                result.rows_affected()
            )),
        }
    }
}
