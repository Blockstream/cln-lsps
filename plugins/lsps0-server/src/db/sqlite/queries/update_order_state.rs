use anyhow::{anyhow, Result};

use sqlx::sqlite::SqliteQueryResult;
use sqlx::{Sqlite, Transaction};

use uuid::Uuid;
use lsp_primitives::lsps0::common_schemas::IsoDatetime;
use lsp_primitives::lsps1::schema::OrderState;

use crate::db::sqlite::conversion::IntoSqliteInteger;

pub struct UpdateOrderStateQuery {
    pub(crate) order_uuid : Uuid,
    pub(crate) state: OrderState,
}

impl UpdateOrderStateQuery {
    pub(crate) async fn execute<'b>(&self, tx: &'b mut Transaction<'_, Sqlite>) -> Result<()> {
        log::debug!(
            "Update order_state order={} to {:?}",
            self.order_uuid,
            self.state,
        );
        let state = self.state.into_sqlite_integer()?;
        let created_at = IsoDatetime::now().into_sqlite_integer()?;
        let order_uuid = self.order_uuid.to_string();

        let result: SqliteQueryResult = sqlx::query!(
            r#"
            INSERT INTO lsps1_order_state
                (order_id, order_state_enum_id, created_at, generation)
            SELECT o.id, ?1, ?2, os.generation+1
                FROM lsps1_order as o
                JOIN lsps1_order_state as os
                ON o.id = os.order_id
                WHERE o.uuid = ?3
                ORDER BY os.generation DESC
                LIMIT 1
            "#,
            state,
            created_at,
            order_uuid
        )
        .execute(&mut **tx)
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
}

#[cfg(test)]
mod test {

    use sqlx::query::Query;

    use crate::db::sqlite::queries::GetPaymentDetailsQuery;
    use crate::db::sqlite::queries::GetOrderQuery;
    use crate::db::sqlite::test::{get_db, create_order_query};
    use super::*;

    #[tokio::test]
    async fn update_payment_state() {
        // Create a database connection
        let db = get_db().await;

        // Create the order query
        let query = create_order_query();
        let uuid = query.order.uuid;
        let initial_payment = query.payment.clone();
        let initial_order = query.order.clone();

        // Execute the query
        let mut tx = db.pool.begin().await.unwrap();
        query.execute(&mut tx).await.unwrap();

        println!("Attempting to update the payment state");
        let query = UpdateOrderStateQuery {
            order_uuid : uuid,
            state : OrderState::Completed
        };

        query.execute(&mut tx).await.unwrap();

        let order = GetOrderQuery::by_uuid(uuid)
            .execute(&mut tx)
            .await
            .unwrap()
            .unwrap();

        tx.commit().await.unwrap();

        assert_eq!(
            order.order_state,
            OrderState::Completed,
            "Failed to update state"
        );

        println!("Attempting to update to payment state again");
        let mut tx = db.pool.begin().await.unwrap();
        let query = UpdateOrderStateQuery {
            order_uuid : uuid,
            state : OrderState::Failed
        }.execute(&mut tx).await.unwrap();

        let order = GetOrderQuery::by_uuid(uuid)
            .execute(&mut tx).await.unwrap().unwrap();

        assert_eq!(
            order.order_state,
            OrderState::Failed,
            "Failed to update state"
        );

        tx.commit().await.unwrap();




    }
}

