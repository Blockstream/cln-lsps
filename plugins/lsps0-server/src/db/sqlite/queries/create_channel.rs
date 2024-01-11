use anyhow::{anyhow, Result};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;
use crate::db::schema::Lsps1Channel;

pub struct CreateChannelQuery {
    pub(crate) order_id: Uuid,
    pub(crate) channel : Lsps1Channel
}

impl CreateChannelQuery {
    pub fn new(order_id: Uuid, channel : Lsps1Channel) -> Self {
        Self {
            order_id,
            channel
        }
    }
}

impl CreateChannelQuery {
    pub(crate) async fn execute(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<()> {
        let uuid_str = self.order_id.to_string();
        let result = sqlx::query!(
            r#"
             INSERT INTO lsps1_channel (order_id, funding_tx, outnum)
             SELECT o.id, ?2, ?3 FROM lsps1_order as o
             where o.uuid = ?1;
             "#,
            uuid_str,
            self.channel.funding_tx,
            self.channel.outnum
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

#[cfg(test)]
mod test {

    use super::*;
    use crate::db::sqlite::test::{get_db, create_order_query};
    use crate::db::schema::Lsps1Channel;
    use crate::db::sqlite::queries::GetChannelQuery;

    #[tokio::test]
    async fn store_channel_in_database() {
        // Create a database connection
        let db = get_db().await;

        // Create the order query
        let query = create_order_query();
        let uuid = query.order.uuid;
        let initial_order = query.order.clone();

        // Execute the query
        let mut tx = db.pool.begin().await.unwrap();
        query.execute(&mut tx).await.unwrap();

        // Store the channel in the database
        let channel = Lsps1Channel { funding_tx : "abcefghij".to_string(), outnum : 0 };
        CreateChannelQuery::new(uuid, channel.clone())
            .execute(&mut tx)
            .await
            .unwrap();

        // Get the channel form the database(
        let returned_channel = GetChannelQuery::by_order_id(initial_order.uuid.clone())
            .execute(&mut tx)
            .await
            .unwrap();

        tx.commit().await.unwrap();
        assert_eq!(channel.funding_tx, returned_channel.funding_tx);
        assert_eq!(channel.outnum, returned_channel.outnum);
    }
}
