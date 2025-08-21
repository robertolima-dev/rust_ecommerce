use crate::app_core::app_state::AppState;
use crate::apps::cart::models::{Cart, CartStatus};
use bigdecimal::BigDecimal;
// use chrono::NaiveDateTime;
use chrono::{DateTime, Utc};
// use sqlx::Row;
use uuid::Uuid;

pub struct CartRepository<'a> {
    app_state: &'a AppState,
}

impl<'a> CartRepository<'a> {
    pub fn new(app_state: &'a AppState) -> Self {
        Self { app_state }
    }

    pub async fn find_all(&self, tenant_id: Uuid) -> Result<Vec<Cart>, sqlx::Error> {
        let rows = sqlx::query_as!(
            Cart,
            r#"
            SELECT
                id,
                tenant_id,
                user_id,
                status as "status: CartStatus",
                currency,
                (subtotal)::numeric as "subtotal!: BigDecimal",
                (discount_total)::numeric as "discount_total!: BigDecimal",
                (tax_total)::numeric as "tax_total!: BigDecimal",
                (shipping_total)::numeric as "shipping_total!: BigDecimal",
                (grand_total)::numeric as "grand_total!: BigDecimal",
                version,
                (expires_at AT TIME ZONE 'UTC') as "expires_at?: DateTime<Utc>",
                (dt_created AT TIME ZONE 'UTC') as "dt_created!: DateTime<Utc>",
                (dt_updated AT TIME ZONE 'UTC') as "dt_updated!: DateTime<Utc>",
                (dt_deleted AT TIME ZONE 'UTC') as "dt_deleted?: DateTime<Utc>"
            FROM carts
            WHERE tenant_id = $1
            "#,
            tenant_id
        )
        .fetch_all(&self.app_state.db)
        .await?;

        let carts = rows
            .into_iter()
            .map(|row| Cart {
                id: row.id,
                tenant_id: row.tenant_id,
                user_id: row.user_id,
                status: row.status,
                currency: row.currency,
                subtotal: row.subtotal,
                discount_total: row.discount_total,
                tax_total: row.tax_total,
                shipping_total: row.shipping_total,
                grand_total: row.grand_total,
                version: row.version,
                expires_at: row.expires_at,
                dt_created: row.dt_created,
                dt_updated: row.dt_updated,
                dt_deleted: row.dt_deleted,
            })
            .collect();

        Ok(carts)
    }

    pub async fn find_by_tenant_id(&self, tenant_id: Uuid) -> Result<Option<Cart>, sqlx::Error> {
        let row = sqlx::query_as!(
            Cart,
            r#"
            SELECT
                id,
                tenant_id,
                user_id,
                status as "status: CartStatus",
                currency,
                (subtotal)::numeric as "subtotal!: BigDecimal",
                (discount_total)::numeric as "discount_total!: BigDecimal",
                (tax_total)::numeric as "tax_total!: BigDecimal",
                (shipping_total)::numeric as "shipping_total!: BigDecimal",
                (grand_total)::numeric as "grand_total!: BigDecimal",
                version,
                (expires_at AT TIME ZONE 'UTC') as "expires_at?: DateTime<Utc>",
                (dt_created AT TIME ZONE 'UTC') as "dt_created!: DateTime<Utc>",
                (dt_updated AT TIME ZONE 'UTC') as "dt_updated!: DateTime<Utc>",
                (dt_deleted AT TIME ZONE 'UTC') as "dt_deleted?: DateTime<Utc>"
            FROM carts
            WHERE dt_deleted IS NULL AND status = 'ACTIVE' AND tenant_id = $1
            "#,
            tenant_id
        )
        .fetch_optional(&self.app_state.db)
        .await?;

        Ok(row)
    }

    pub async fn create(&self, tenant_id: Uuid, user_id: Uuid) -> Result<Cart, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now().naive_utc();

        let row = sqlx::query_as!(
            Cart,
            r#"
            INSERT INTO carts (
                id,
                tenant_id,
                user_id,
                status,
                currency,
                subtotal,
                discount_total,
                tax_total,
                shipping_total,
                version,
                dt_created,
                dt_updated
            )
            VALUES ($1, $2, $3, $4, $5, 0, 0, 0, 0, 0, $6, $7)
            RETURNING
                id,
                tenant_id,
                user_id,
                status        as "status: CartStatus",
                currency,
                (subtotal)::numeric        as "subtotal!: BigDecimal",
                (discount_total)::numeric  as "discount_total!: BigDecimal",
                (tax_total)::numeric       as "tax_total!: BigDecimal",
                (shipping_total)::numeric  as "shipping_total!: BigDecimal",
                (grand_total)::numeric     as "grand_total!: BigDecimal",
                version,
                (expires_at AT TIME ZONE 'UTC') as "expires_at?: DateTime<Utc>",
                (dt_created AT TIME ZONE 'UTC') as "dt_created!: DateTime<Utc>",
                (dt_updated AT TIME ZONE 'UTC') as "dt_updated!: DateTime<Utc>",
                (dt_deleted AT TIME ZONE 'UTC') as "dt_deleted?: DateTime<Utc>"
            "#,
            id,
            tenant_id,
            user_id,
            CartStatus::ACTIVE as _, // ou passar status
            "BRL",                   // currency
            now,
            now
        )
        .fetch_one(&self.app_state.db)
        .await?;

        Ok(row)
    }

    pub async fn delete(&self, id: Uuid, tenant_id: Uuid) -> Result<bool, sqlx::Error> {
        let exists = sqlx::query_as!(
            Cart,
            r#"
            SELECT
                id,
                tenant_id,
                user_id,
                status as "status: CartStatus",
                currency,
                (subtotal)::numeric as "subtotal!: BigDecimal",
                (discount_total)::numeric as "discount_total!: BigDecimal",
                (tax_total)::numeric as "tax_total!: BigDecimal",
                (shipping_total)::numeric as "shipping_total!: BigDecimal",
                (grand_total)::numeric as "grand_total!: BigDecimal",
                version,
                (expires_at AT TIME ZONE 'UTC') as "expires_at?: DateTime<Utc>",
                (dt_created AT TIME ZONE 'UTC') as "dt_created!: DateTime<Utc>",
                (dt_updated AT TIME ZONE 'UTC') as "dt_updated!: DateTime<Utc>",
                (dt_deleted AT TIME ZONE 'UTC') as "dt_deleted?: DateTime<Utc>"
            FROM carts
            WHERE dt_deleted IS NULL AND id = $1 AND tenant_id = $2
            "#,
            id,
            tenant_id
        )
        .fetch_optional(&self.app_state.db)
        .await?;

        // Se não existe, retorna false direto
        if exists.is_none() {
            return Ok(false);
        }

        let now = Utc::now();

        let result = sqlx::query!(
            "UPDATE carts SET dt_deleted = $1, status = $2 WHERE id = $3 AND tenant_id = $4 AND dt_deleted IS NULL",
            now.naive_utc(),
            CartStatus::CANCELLED as _,
            id,
            tenant_id
        )
        .execute(&self.app_state.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
