use anyhow::{anyhow, Context, Result};
use uuid::Uuid;

use sqlx::{Sqlite, Transaction};

use crate::db::schema::Lsps1PaymentDetails;
use crate::db::sqlite::schema::Lsps1PaymentDetails as Lsps1PaymentDetailsSqlite;

pub enum GetPaymentDetailsQuery {
    ByUuid(Uuid),
    ByLabel(String),
}

impl GetPaymentDetailsQuery {
    pub fn by_uuid(uuid: Uuid) -> Self {
        Self::ByUuid(uuid)
    }

    pub fn by_label(label: String) -> Self {
        Self::ByLabel(label)
    }
}

impl GetPaymentDetailsQuery {
    pub async fn execute(
        &self,
        tx: &mut Transaction<'static, Sqlite>,
    ) -> Result<Option<Lsps1PaymentDetails>> {
        match &self {
            Self::ByUuid(uuid) => Self::execute_by_uuid(uuid, tx).await,
            Self::ByLabel(label) => Self::execute_by_label(label, tx).await,
        }
    }

    async fn execute_by_uuid(
        uuid: &Uuid,
        tx: &mut Transaction<'static, Sqlite>,
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
        .fetch_optional(&mut **tx)
        .await
        .context("Failed to execute query")?;

        match result {
            Some(r) => Ok(Some(Lsps1PaymentDetails::try_from(&r)?)),
            None => Ok(None),
        }
    }

    pub(crate) async fn execute_by_label(
        label: &str,
        tx: &mut Transaction<'_, Sqlite>,
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
        .fetch_optional(&mut **tx)
        .await?;

        match payment_details {
            Some(r) => Ok(Some(Lsps1PaymentDetails::try_from(&r)?)),
            None => Ok(None),
        }
    }
}
