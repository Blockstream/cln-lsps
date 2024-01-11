use anyhow::{anyhow, Result};

use sqlx::Sqlite;
use sqlx::Transaction;

use std::convert::TryFrom;
use crate::db::schema::Lsps1Channel as Lsps1ChannelBase;
use crate::db::sqlite::schema::Lsps1Channel;
use uuid::Uuid;

pub(crate) struct GetChannelQuery {
    order_uuid: Uuid,
}

impl GetChannelQuery {
    pub(crate) fn by_order_id(uuid: Uuid) -> Self {
        GetChannelQuery { order_uuid: uuid }
    }
}

impl GetChannelQuery {
    pub async fn execute(
        &self,
        tx: &mut Transaction<'static, Sqlite>,
    ) -> Result<Lsps1ChannelBase, anyhow::Error> {
        let order_str = self.order_uuid.to_string();

        sqlx::query_as!(
            Lsps1Channel,
            r#"
             SELECT c.funding_tx, c.outnum FROM lsps1_channel as c
             JOIN lsps1_order as od
              ON c.order_id = od.id
              WHERE od.uuid = ?1
              "#,
            order_str
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| anyhow!("db.get_channel Failed: {}", e))?
        .ok_or_else(|| {
            anyhow!(
                "Failed to find order '{}' and could not create channel",
                self.order_uuid
            )
        })
        .and_then(|channel| Lsps1ChannelBase::try_from(&channel))
    }
}
