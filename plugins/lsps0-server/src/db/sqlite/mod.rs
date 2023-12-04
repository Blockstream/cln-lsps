mod conversion;
mod schema;

use anyhow::{Context, Result, anyhow};

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use uuid::Uuid;
use crate::db::schema::Lsps1Order;
use crate::db::sqlite::schema::Lsps1Order as Lsps1OrderSqlite;


struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn connect_with_options(options: SqliteConnectOptions) -> Result<Self> {
        let pool = SqlitePool::connect_with(options).await?;
        Ok(Database { pool })
    }

    pub async fn create_order(&self, order: &Lsps1Order) ->Result<()> {
        let order = Lsps1OrderSqlite::try_from(order)?;

       let query = sqlx::query!(r#"
            INSERT INTO lsps1_order (
              uuid, lsp_balance_sat, client_balance_sat, confirms_within_blocks, channel_expiry_blocks,
              token, refund_onchain_address, announce_channel, created_at, expires_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10
            );"#, 
            order.uuid, order.lsp_balance_sat, order.client_balance_sat, order.confirms_within_blocks, order.channel_expiry_blocks,
            order.token, order.refund_onchain_address, order.announce_channel,
            order.created_at, order.expires_at);
            
        let _query_result = query
            .execute(&self.pool)
            .await
            .map_err(|err| anyhow!("Error while executing query. {:?}", err))?;

        return Ok(())
    }

    pub async fn get_order_by_uuid(&self, uuid : Uuid) -> Result<Option<Lsps1Order>> {
        let uuid_string = uuid.to_string();
        let result = sqlx::query_as!(Lsps1OrderSqlite, r#"SELECT  * FROM lsps1_order where uuid = ?"#, uuid_string)
            .fetch_optional(&self.pool).await.context("Failed to execute query")?;

        match result {
            Some(r) => Ok(Some(Lsps1Order::try_from(&r)?)),
            None => Ok(None)
        }
    }

    #[cfg(test)]
    pub async fn delete_order_by_uuid(&self, uuid : Uuid) -> Result<()> {
        let uuid_string = uuid.to_string();
        sqlx::query!("DELETE FROM lsps1_order where uuid = ?1", uuid_string)
            .execute(&self.pool)
            .await
            .context("Failed to execute delete_order query")
            .map(|_| ())
    }


}

#[cfg(test)]
mod test {
    use super::*;
    use lsp_primitives::lsps0::common_schemas::{SatAmount, IsoDatetime};

    use std::time::{SystemTime, UNIX_EPOCH, Duration};

    async fn get_db() -> Database {
        let options = SqliteConnectOptions::default()
            .filename("./data/lsp_server.db")
            .create_if_missing(true);
        println!("{:?}", options);
        Database::connect_with_options(options).await.unwrap()
    }

    #[tokio::test]
    async fn create_order() {
        // Create a database connection
        let db = get_db().await;

        // Create the order_uuid
        let uuid = Uuid::new_v4();

        // Show the database connection
        let created_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let expires_at = SystemTime::now().checked_add(Duration::from_secs(60*60*24)).unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let order = Lsps1Order {
            uuid,
            lsp_balance_sat : SatAmount::new(100_000),
            client_balance_sat : SatAmount::new(0),
            confirms_within_blocks: 0,
            created_at : IsoDatetime::from_unix_timestamp(created_at).unwrap(),
            expires_at : IsoDatetime::from_unix_timestamp(expires_at).unwrap(),
            refund_onchain_address : None,
            token : None,
            channel_expiry_blocks: 6*24*30,
            announce_channel : false,

        };

        let _ = db.create_order(&order).await.unwrap();

        // Confirm the order is in the database
        // There are a lot of conversions going on. (Sqlite only supports i64)
        // We test here that all conversions happen correctly and we are not corrupting any data in
        // the process
        let order = db.get_order_by_uuid(uuid).await.expect("Query succeeded").expect("Has a result");
        assert_eq!(order.uuid, uuid);
        assert_eq!(order.lsp_balance_sat, SatAmount::new(100_000));
        assert_eq!(order.client_balance_sat, SatAmount::new(0));
        assert_eq!(order.confirms_within_blocks, 0);
        assert_eq!(order.created_at, IsoDatetime::from_unix_timestamp(created_at).unwrap());
        assert_eq!(order.expires_at, IsoDatetime::from_unix_timestamp(expires_at).unwrap());
        assert!(order.refund_onchain_address.is_none());
        assert!(order.token.is_none());
        assert_eq!(order.channel_expiry_blocks, 6*24*30);
        assert_eq!(order.announce_channel, false);


        // Remove the entry from the database
        db.delete_order_by_uuid(uuid).await.unwrap();
    }
}
