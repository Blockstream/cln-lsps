mod conversion;
mod schema;

use anyhow::{anyhow, Context, Result};

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqliteQueryResult};
use uuid::Uuid;

use crate::db::schema::{Lsps1Channel, Lsps1Order, Lsps1PaymentDetails};
use crate::db::sqlite::conversion::IntoSqliteInteger;
use crate::db::sqlite::schema::{
    Lsps1Order as Lsps1OrderSqlite, Lsps1PaymentDetails as Lsps1PaymentDetailsSqlite,
};

use lsp_primitives::lsps0::common_schemas::IsoDatetime;
use lsp_primitives::lsps1::schema::PaymentState;

#[derive(Clone)]
pub struct Database {
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
               bolt11_invoice_label,
               onchain_address,
               onchain_block_confirmations_required,
               minimum_fee_for_0conf
            ) VALUES 
            (
              ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            RETURNING id;
            "#,
            order_id.id,
            payment.fee_total_sat,
            payment.order_total_sat,
            payment.bolt11_invoice,
            payment.bolt11_invoice_label,
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
                created_at,
                generation
             ) VALUES (
                ?1, ?2, ?3, ?4
             );"#,
            order_id.id,
            order.order_state,
            now,
            order.generation
        )
        .execute(&mut *tx)
        .await?;

        // Give the payment a state
        sqlx::query!(
            r#"INSERT INTO lsps1_payment_state (
              payment_details_id,
              payment_state,
              created_at,
              generation
            ) VALUES (
               ?1, ?2, ?3, ?4
            );"#,
            payment_id.id,
            payment.state,
            now,
            payment.generation
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        return Ok(());
    }

    pub async fn get_order_by_uuid(&self, uuid: &Uuid) -> Result<Option<Lsps1Order>> {
        let uuid_string = uuid.to_string();
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
        .fetch_optional(&self.pool)
        .await
        .context("Failed to execute query")?;

        match result {
            Some(r) => Ok(Some(Lsps1Order::try_from(&r)?)),
            None => Ok(None),
        }
    }

    pub async fn get_payment_details_by_uuid(
        &self,
        uuid: Uuid,
    ) -> Result<Option<Lsps1PaymentDetails>> {
        let uuid_string = uuid.to_string();

        let result = sqlx::query_as!(
            Lsps1PaymentDetailsSqlite,
            r#"SELECT
               p.fee_total_sat,
               p.order_total_sat,
               p.bolt11_invoice,
               p.bolt11_invoice_label,
               p.onchain_address,
               p.onchain_block_confirmations_required,
               p.minimum_fee_for_0conf,
               ps.payment_state as state,
               ps.generation
               FROM lsps1_payment_details as p
               JOIN lsps1_order as o
               ON o.id = p.order_id
               JOIN lsps1_payment_state as ps
               ON ps.payment_details_id = p.id
               WHERE o.uuid = ?1
               ORDER BY ps.generation DESC
               LIMIT 1;
               "#,
            uuid_string
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to execute query")?;

        match result {
            Some(r) => Ok(Some(Lsps1PaymentDetails::try_from(&r)?)),
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

        // Remove the associated channel
        sqlx::query!(
            r#"
          WITH to_delete AS (SELECT id from lsps1_order where uuid =?1)
          DELETE FROM lsps1_channel WHERE order_id in to_delete;"#,
            uuid_string
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!("DELETE FROM lsps1_order where uuid = ?1;", uuid_string)
            .execute(&mut *tx)
            .await?;

        Ok(())
    }

    pub(crate) async fn get_payment_details_by_label(
        &self,
        label: &str,
    ) -> Result<Option<Lsps1PaymentDetails>> {
        log::debug!("Get payment by label={}", label);

        let payment_details = sqlx::query_as!(
            Lsps1PaymentDetailsSqlite,
            r#"
            SELECT 
                fee_total_sat, order_total_sat, bolt11_invoice, 
                bolt11_invoice_label, minimum_fee_for_0conf, 
                onchain_address, onchain_block_confirmations_required,
                ps.payment_state as state,
                ps.generation
            FROM lsps1_payment_details AS pd
            JOIN lsps1_payment_state AS ps
            ON pd.id = ps.payment_details_id
            WHERE pd.bolt11_invoice_label = ?1
            ORDER by ps.generation DESC
            LIMIT 1
            "#,
            label
        )
        .fetch_optional(&self.pool)
        .await?;

        match payment_details {
            Some(r) => Ok(Some(Lsps1PaymentDetails::try_from(&r)?)),
            None => Ok(None),
        }
    }

    pub(crate) async fn update_payment_state(
        &self,
        label: &str,
        state: PaymentState,
        generation: u64,
    ) -> Result<()> {
        log::debug!("Update payment_state label={} to {:?}", label, state);
        let state = state.into_sqlite_integer()?;
        let created_at = IsoDatetime::now().into_sqlite_integer()?;
        let generation = generation.into_sqlite_integer()?;
        let new_generation = generation + 1;

        let result: SqliteQueryResult = sqlx::query!(
            r#"
            INSERT INTO lsps1_payment_state 
                (payment_details_id, payment_state, created_at, generation)
            SELECT id, ?1, ?2, ?3
                FROM lsps1_payment_details
                WHERE bolt11_invoice_label = ?4
            "#,
            state,
            created_at,
            new_generation,
            label
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 1 {
            Ok(())
        } else {
            Err(anyhow!(
                "Error in updating state. Query affected {} rows",
                result.rows_affected()
            ))
        }
    }

    pub(crate) async fn create_channel(
        &self,
        order_id: &Uuid,
        channel: Lsps1Channel,
    ) -> Result<()> {
        let order_uuid = order_id.to_string();

        let result = sqlx::query!(
            r#"
             INSERT INTO lsps1_channel (order_id, channel_id)
             SELECT o.id, ?2 FROM lsps1_order as o
             where o.uuid = ?1;
             "#,
            order_uuid,
            channel.channel_id
        )
        .execute(&self.pool)
        .await?;

        match result.rows_affected() {
            0 => Err(anyhow!(
                "Failed to find order '{}' and could not create channel",
                order_uuid
            )),
            1 => Ok(()),
            _ => Err(anyhow!(
                "Error in updating state. Query affected {} rows",
                result.rows_affected()
            )),
        }
    }

    pub(crate) async fn get_channel(&self, order_id: &Uuid) -> Result<Lsps1Channel> {
        let order_uuid = order_id.to_string();

        sqlx::query_as!(
            Lsps1Channel,
            r#"
             SELECT c.channel_id FROM lsps1_channel as c
             JOIN lsps1_order as od
              ON c.order_id = od.id
              WHERE od.uuid = ?1
              "#,
            order_uuid
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| anyhow!("db.get_channel Failed: {}", e))?
        .ok_or_else(|| {
            anyhow!(
                "Failed to find order '{}' and could not create channel",
                order_uuid
            )
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use lsp_primitives::lsps0::common_schemas::{IsoDatetime, PublicKey, SatAmount};
    use lsp_primitives::lsps1::schema::OrderState;

    async fn get_db() -> Database {
        let options = SqliteConnectOptions::default()
            .filename("./data/lsp_server.db")
            .create_if_missing(true);
        Database::connect_with_options(options).await.unwrap()
    }

    fn create_test_order() -> Lsps1Order {
        // Create the order_uuid
        let uuid = Uuid::new_v4();

        // Show the database connection
        let created_at = IsoDatetime::now();
        let expires_at = IsoDatetime::now();

        Lsps1Order {
            uuid,
            client_node_id: PublicKey::from_hex(
                "026d58c2b93d278acef549167e34cf6c541fc2332b1e36e7fe57e54576cd5fa170",
            )
            .unwrap(),
            lsp_balance_sat: SatAmount::new(100_000),
            client_balance_sat: SatAmount::new(0),
            confirms_within_blocks: 0,
            created_at,
            expires_at,
            refund_onchain_address: None,
            token: None,
            channel_expiry_blocks: 6 * 24 * 30,
            announce_channel: false,
            order_state: OrderState::Created,
            generation: 0,
        }
    }

    fn create_test_payment(order: &Lsps1Order) -> Lsps1PaymentDetails {
        Lsps1PaymentDetails {
            fee_total_sat: SatAmount::new(500),
            order_total_sat: SatAmount::new(500),
            bolt11_invoice: format!("bolt11_invoice.{}", order.uuid),
            bolt11_invoice_label: format!("test.order.{}", order.uuid),
            onchain_address: None,
            minimum_fee_for_0conf: None,
            onchain_block_confirmations_required: None,
            state: PaymentState::ExpectPayment,
            generation: 0,
        }
    }

    #[tokio::test]
    async fn test_create_order() {
        // Create a database connection
        let db = get_db().await;

        let initial_order = create_test_order();
        let initial_payment = create_test_payment(&initial_order);
        let uuid = initial_order.uuid;

        let _ = db
            .create_order(&initial_order, &initial_payment)
            .await
            .unwrap();

        // Confirm the order is in the database
        // There are a lot of conversions going on. (Sqlite only supports i64)
        // We test here that all conversions happen correctly and we are not corrupting any data in
        // the process
        let order = db
            .get_order_by_uuid(&uuid)
            .await
            .expect("Query succeeded")
            .expect("Has a result");

        assert_eq!(order.uuid, order.uuid);
        assert_eq!(order.lsp_balance_sat, SatAmount::new(100_000));
        assert_eq!(order.client_balance_sat, SatAmount::new(0));
        assert_eq!(order.confirms_within_blocks, 0);
        assert_eq!(
            order.created_at.unix_timestamp(),
            initial_order.created_at.unix_timestamp()
        );
        assert_eq!(
            order.expires_at.unix_timestamp(),
            initial_order.expires_at.unix_timestamp()
        );
        assert!(order.refund_onchain_address.is_none());
        assert!(order.token.is_none());
        assert_eq!(order.channel_expiry_blocks, 6 * 24 * 30);
        assert_eq!(order.announce_channel, false);

        // Remove the entry from the database
        db.delete_order_by_uuid(uuid).await.unwrap();
        println!("Deleted orders");
    }

    #[tokio::test]
    async fn update_order_payment_state() {
        // Create a database connection
        let db = get_db().await;

        let order = create_test_order();
        let payment = create_test_payment(&order);
        let uuid = order.uuid;
        let label = payment.bolt11_invoice_label.clone();

        let _ = db.create_order(&order, &payment).await.unwrap();

        println!("Attempting to update the payment state");
        let _ = db
            .update_payment_state(&label, PaymentState::Paid, payment.generation)
            .await
            .unwrap();

        let result_label = db
            .get_payment_details_by_label(&label)
            .await
            .unwrap()
            .unwrap();
        let result_uuid = db.get_payment_details_by_uuid(uuid).await.unwrap().unwrap();

        assert_eq!(
            result_uuid.state,
            PaymentState::Paid,
            "Bad state using uuid"
        );

        assert_eq!(
            result_label.state,
            PaymentState::Paid,
            "Bad state using label"
        );

        // db.delete_order_by_uuid(uuid).await.unwrap();
    }

    #[tokio::test]
    async fn store_channel_in_database() {
        // Create a database connection
        let db = get_db().await;

        // Store the order and payment in the database
        let order = create_test_order();
        let payment = create_test_payment(&order);
        let _ = db.create_order(&order, &payment).await.unwrap();

        // Store the channel in the database
        let channel_id = format!("channel.{}", order.uuid);
        let channel = Lsps1Channel {
            channel_id: channel_id.clone(),
        };
        let _ = db.create_channel(&order.uuid, channel).await.unwrap();

        // Get the channel form the database
        let returned_channel = db.get_channel(&order.uuid).await.unwrap();

        assert_eq!(channel_id, returned_channel.channel_id);

        // Clean up the test
        db.delete_order_by_uuid(order.uuid).await.unwrap();
    }
}
