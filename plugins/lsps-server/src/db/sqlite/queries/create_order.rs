use anyhow::anyhow;

use sqlx::Sqlite;
use sqlx::Transaction;

use lsp_primitives::lsps0::common_schemas::IsoDatetime;

use crate::db::schema::{Lsps1Order, Lsps1PaymentDetails};
use crate::db::sqlite::schema::{
    Lsps1Order as Lsps1OrderSqlite, Lsps1PaymentDetails as Lsps1PaymentDetailsSqlite,
};

pub struct Lsps1CreateOrderQuery {
    pub order: Lsps1Order,
    pub payment: Lsps1PaymentDetails,
}

impl Lsps1CreateOrderQuery {
    pub async fn execute(
        &self,
        tx: &mut Transaction<'static, Sqlite>,
    ) -> Result<(), anyhow::Error> {
        // Convert all data to sqlite types
        // All integer types are i64, OnchainAddresses become String, ...
        let now = IsoDatetime::now().unix_timestamp();
        let order = Lsps1OrderSqlite::try_from(&self.order)?;
        let payment = Lsps1PaymentDetailsSqlite::try_from(&self.payment)?;

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
        .fetch_optional(&mut **tx)
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
        .fetch_one(&mut **tx)
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
        .execute(&mut **tx)
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
        .execute(&mut **tx)
        .await?;

        return Ok(());
    }
}
