mod conversion;
mod schema;

use anyhow::{anyhow, Context, Result};

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use uuid::Uuid;

use crate::db::schema::{Lsps1Order, Lsps1PaymentDetails};
use crate::db::sqlite::schema::{
    Lsps1Order as Lsps1OrderSqlite, Lsps1PaymentDetails as Lsps1PaymentDetailsSqlite,
};
use lsp_primitives::lsps0::common_schemas::IsoDatetime;

struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn connect_with_options(options: SqliteConnectOptions) -> Result<Self> {
        let pool = SqlitePool::connect_with(options).await?;
        Ok(Database { pool })
    }

    /// Stores the order and related payment details in the database
    pub async fn create_order(
        &self,
        order: &Lsps1Order,
        payment: &Lsps1PaymentDetails,
    ) -> Result<()> {
        // Convert all data to sqlite types
        // All integer types are i64, OnchainAddresses become String, ...
        let now = IsoDatetime::now().unix_timestamp();
        let order = Lsps1OrderSqlite::try_from(order)?;
        let payment = Lsps1PaymentDetailsSqlite::try_from(payment)?;

        // Create the transaction
        let mut tx = self.pool.begin().await?;

        // Insert the order
        struct IdType {
            pub id: i64,
        }

        // Create the entry in the lsps1_order table
        let order_id = sqlx::query_as!(
            IdType,
            r#"
            INSERT INTO lsps1_order (
              uuid, client_node_id,
              lsp_balance_sat, client_balance_sat,
              confirms_within_blocks, channel_expiry_blocks,
              token, refund_onchain_address,
              announce_channel, created_at,
              expires_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11
            )
            RETURNING id;"#,
            order.uuid,
            order.client_node_id,
            order.lsp_balance_sat,
            order.client_balance_sat,
            order.confirms_within_blocks,
            order.channel_expiry_blocks,
            order.token,
            order.refund_onchain_address,
            order.announce_channel,
            order.created_at,
            order.expires_at
        )
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(anyhow!("Failed to insert order into database"))?;

        // Create the payment
        let payment_id = sqlx::query_as!(
            IdType,
            r#"
            INSERT INTO lsps1_payment_details (
               order_id,
               fee_total_sat,
               order_total_sat,
               bolt11_invoice,
               onchain_address,
               onchain_block_confirmations_required,
               minimum_fee_for_0conf
            ) VALUES 
            (
              ?1, ?2, ?3, ?4, ?5, ?6, ?7)
            RETURNING id;
            "#,
            order_id.id,
            payment.fee_total_sat,
            payment.order_total_sat,
            payment.bolt11_invoice,
            payment.onchain_address,
            payment.onchain_block_confirmations_required,
            payment.minimum_fee_for_0conf
        )
        .fetch_one(&mut *tx)
        .await?;

        // Give the order a state
        sqlx::query!(
            r#"INSERT INTO lsps1_order_state (
                order_id, order_state_enum_id,
                created_at
             ) VALUES (
                ?1, ?2, ?3
             );"#,
            order_id.id,
            1,
            now
        )
        .execute(&mut *tx)
        .await?;

        // Give the payment a state
        sqlx::query!(
            r#"INSERT INTO lsps1_payment_state (
              payment_details_id,
              payment_state,
              created_at
            ) VALUES (
               ?1, 1, ?2
            );"#,
            payment_id.id,
            now
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        return Ok(());
    }

    pub async fn get_order_by_uuid(&self, uuid: Uuid) -> Result<Option<Lsps1Order>> {
        let uuid_string = uuid.to_string();
        let result = sqlx::query_as!(
            Lsps1OrderSqlite,
            r#"SELECT
                uuid, client_node_id, lsp_balance_sat,
                client_balance_sat, confirms_within_blocks, channel_expiry_blocks,
                token, refund_onchain_address, announce_channel,
                created_at, expires_at
            FROM lsps1_order where uuid = ?"#,
            uuid_string
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to execute query")?;

        match result {
            Some(r) => Ok(Some(Lsps1Order::try_from(&r)?)),
            None => Ok(None),
        }
    }

    #[cfg(test)]
    pub async fn delete_order_by_uuid(&self, uuid: Uuid) -> Result<()> {
        let uuid_string = uuid.to_string();

        // Start the transaction
        let mut tx = self.pool.begin().await?;

        // Remove the state of the order
        sqlx::query!(
            r#"
            WITH to_delete AS (SELECT id FROM lsps1_order WHERE uuid = ?1)
            DELETE FROM lsps1_order_state WHERE order_id in to_delete;"#,
            uuid_string
        )
        .execute(&mut *tx)
        .await?;

        // Remove the payment state
        sqlx::query!(
            r#"
            WITH to_delete AS (
              SELECT payment.id FROM lsps1_order AS ord
              JOIN lsps1_payment_details AS payment
              ON ord.id = payment.order_id
              WHERE ord.uuid = ?1
            )
            DELETE FROM lsps1_payment_state WHERE payment_details_id in to_delete;
            "#,
            uuid_string
        )
        .execute(&mut *tx)
        .await?;

        // Remove the payment associated to the order
        sqlx::query!(
            r#"
            WITH to_delete as (SELECT id from lsps1_order WHERE uuid = ?1)
            DELETE FROM lsps1_payment_details WHERE order_id in to_delete;"#,
            uuid_string
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!("DELETE FROM lsps1_order where uuid = ?1;", uuid_string)
            .execute(&mut *tx)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use lsp_primitives::lsps0::common_schemas::{IsoDatetime, PublicKey, SatAmount};

    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    async fn get_db() -> Database {
        let options = SqliteConnectOptions::default()
            .filename("./data/lsp_server.db")
            .create_if_missing(true);
        Database::connect_with_options(options).await.unwrap()
    }

    #[tokio::test]
    async fn create_order() {
        // Create a database connection
        let db = get_db().await;

        // Create the order_uuid
        let uuid = Uuid::new_v4();

        // Show the database connection
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let expires_at = SystemTime::now()
            .checked_add(Duration::from_secs(60 * 60 * 24))
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let order = Lsps1Order {
            uuid,
            client_node_id: PublicKey::from_hex(
                "026d58c2b93d278acef549167e34cf6c541fc2332b1e36e7fe57e54576cd5fa170",
            )
            .unwrap(),
            lsp_balance_sat: SatAmount::new(100_000),
            client_balance_sat: SatAmount::new(0),
            confirms_within_blocks: 0,
            created_at: IsoDatetime::from_unix_timestamp(created_at).unwrap(),
            expires_at: IsoDatetime::from_unix_timestamp(expires_at).unwrap(),
            refund_onchain_address: None,
            token: None,
            channel_expiry_blocks: 6 * 24 * 30,
            announce_channel: false,
        };

        let payment = Lsps1PaymentDetails {
            fee_total_sat: SatAmount::new(500),
            order_total_sat: SatAmount::new(500),
            bolt11_invoice: String::from("erik"),
            onchain_address: None,
            minimum_fee_for_0conf: None,
            onchain_block_confirmations_required: None,
        };

        let _ = db.create_order(&order, &payment).await.unwrap();

        // Confirm the order is in the database
        // There are a lot of conversions going on. (Sqlite only supports i64)
        // We test here that all conversions happen correctly and we are not corrupting any data in
        // the process
        let order = db
            .get_order_by_uuid(uuid)
            .await
            .expect("Query succeeded")
            .expect("Has a result");
        assert_eq!(order.uuid, uuid);
        assert_eq!(order.lsp_balance_sat, SatAmount::new(100_000));
        assert_eq!(order.client_balance_sat, SatAmount::new(0));
        assert_eq!(order.confirms_within_blocks, 0);
        assert_eq!(
            order.created_at,
            IsoDatetime::from_unix_timestamp(created_at).unwrap()
        );
        assert_eq!(
            order.expires_at,
            IsoDatetime::from_unix_timestamp(expires_at).unwrap()
        );
        assert!(order.refund_onchain_address.is_none());
        assert!(order.token.is_none());
        assert_eq!(order.channel_expiry_blocks, 6 * 24 * 30);
        assert_eq!(order.announce_channel, false);

        // Remove the entry from the database
        db.delete_order_by_uuid(uuid).await.unwrap();
        println!("Deleted orders");
    }
}
