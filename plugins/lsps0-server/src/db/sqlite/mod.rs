mod conversion;
mod schema;
use async_trait::async_trait;
pub(crate) mod queries;

use anyhow::Result;

use sqlx::sqlite::{Sqlite, SqliteConnectOptions, SqlitePool};
use sqlx::Transaction;

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

#[async_trait]
pub trait Query<T> {
    async fn execute(self, executor: &mut Transaction<'static, Sqlite>) -> Result<T>;
}

impl Database {
    pub async fn connect_with_options(options: SqliteConnectOptions) -> Result<Self> {
        let pool = SqlitePool::connect_with(options).await?;
        Ok(Database { pool })
    }

    pub async fn begin(&self) -> Result<Transaction<'static, Sqlite>> {
        Ok(self.pool.begin().await?)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    use uuid::Uuid;

    use lsp_primitives::lsps0::common_schemas::{IsoDatetime, PublicKey, SatAmount};
    use lsp_primitives::lsps1::schema::{OrderState, PaymentState};

    use crate::db::schema::{Lsps1Order, Lsps1PaymentDetails};
    use crate::db::sqlite::queries::{GetOrderQuery, Lsps1CreateOrderQuery};

    pub async fn get_db() -> Database {
        let options = SqliteConnectOptions::default()
            .filename("./data/lsp_server.db")
            .create_if_missing(true);
        Database::connect_with_options(options).await.unwrap()
    }

    pub fn create_test_order() -> Lsps1Order {
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

    pub fn create_test_payment(order: &Lsps1Order) -> Lsps1PaymentDetails {
        Lsps1PaymentDetails {
            order_uuid: order.uuid,
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

    pub fn create_order_query() -> Lsps1CreateOrderQuery {
        let order = create_test_order();
        let payment = create_test_payment(&order);

        Lsps1CreateOrderQuery { payment, order }
    }

    #[tokio::test]
    async fn test_create_order() {
        // Create a database connection
        let db = get_db().await;

        // Create the order query
        let query = create_order_query();
        let uuid = query.order.uuid;
        let initial_order = query.order.clone();

        // Execute the query
        let mut tx = db.pool.begin().await.unwrap();
        query.execute(&mut tx).await.unwrap();

        // Confirm the order is in the database
        // There are a lot of conversions going on. (Sqlite only supports i64)
        // We test here that all conversions happen correctly and we are not corrupting any data in
        // the process
        let get_order_query = GetOrderQuery {
            order_id: uuid.clone(),
        };
        let order = get_order_query
            .execute(&mut tx)
            .await
            .expect("Query succeeded")
            .expect("is not None");

        tx.commit().await.unwrap();

        assert_eq!(order.uuid, uuid);
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
    }
}
