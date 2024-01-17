use anyhow::{anyhow, Result};

use sqlx::sqlite::SqliteQueryResult;
use sqlx::{Sqlite, Transaction};

use lsp_primitives::lsps0::common_schemas::IsoDatetime;
use lsp_primitives::lsps1::schema::PaymentState;

use crate::db::sqlite::conversion::IntoSqliteInteger;

pub struct UpdatePaymentStateQuery {
    pub(crate) state: PaymentState,
    pub(crate) generation: u64,
    pub(crate) label: String,
}

impl UpdatePaymentStateQuery {
    pub(crate) async fn execute<'b>(&self, tx: &'b mut Transaction<'_, Sqlite>) -> Result<()> {
        log::debug!(
            "Update payment_state label={} to {:?} at generation {}",
            self.label,
            self.state,
            self.generation
        );
        let state = self.state.into_sqlite_integer()?;
        let created_at = IsoDatetime::now().into_sqlite_integer()?;
        let generation = self.generation.into_sqlite_integer()?;
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
            self.label
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

    use super::*;
    use crate::db::sqlite::queries::GetPaymentDetailsQuery;
    use crate::db::sqlite::test::{create_order_query, get_db};

    #[tokio::test]
    async fn update_payment_state() {
        // Create a database connection
        let db = get_db().await;

        // Create the order query
        let query = create_order_query();
        let uuid = query.order.uuid;
        let initial_payment = query.payment.clone();

        // Execute the query
        let mut tx = db.pool.begin().await.unwrap();
        query.execute(&mut tx).await.unwrap();

        println!("Attempting to update the payment state");
        let query = UpdatePaymentStateQuery {
            generation: initial_payment.generation,
            label: initial_payment.bolt11_invoice_label.clone(),
            state: PaymentState::Hold,
        };

        query.execute(&mut tx).await.unwrap();

        let result_label =
            GetPaymentDetailsQuery::by_label(initial_payment.bolt11_invoice_label.to_string())
                .execute(&mut tx)
                .await
                .unwrap()
                .unwrap();
        let result_uuid = GetPaymentDetailsQuery::by_uuid(uuid)
            .execute(&mut tx)
            .await
            .unwrap()
            .unwrap();

        tx.commit().await.unwrap();

        assert_eq!(
            result_uuid.state,
            PaymentState::Hold,
            "Bad state using uuid"
        );

        assert_eq!(
            result_label.state,
            PaymentState::Hold,
            "Bad state using label"
        );
    }
}
