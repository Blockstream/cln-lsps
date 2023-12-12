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
    pub(crate) async fn execute(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<()> {
        log::debug!(
            "Update payment_state label={} to {:?}",
            self.label,
            self.state
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
