use crate::app_core::app_state::AppState;
use crate::apps::cart::models::{Cart, CartItem, CartStatus};
use bigdecimal::{BigDecimal, ToPrimitive};
// use chrono::NaiveDateTime;
use chrono::{DateTime, Utc};
// use sqlx::Row;
use serde_json::Value;
use sqlx::types::Json;
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
            ORDER BY dt_updated desc
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

    pub async fn update_cart_data(
        &self,
        cart_id: Uuid,
        new_subtotal: &BigDecimal,
        discount_total: &BigDecimal,
        tax_total: &BigDecimal,
        shipping_total: &BigDecimal,
    ) -> Result<bool, sqlx::Error> {
        let now = Utc::now().naive_utc();

        // Converter BigDecimal para i64 (centavos)
        let subtotal_cents = (new_subtotal * BigDecimal::from(100)).to_i64().unwrap_or(0);
        let discount_total_cents = (discount_total * BigDecimal::from(100))
            .to_i64()
            .unwrap_or(0);
        let tax_cents = (tax_total * BigDecimal::from(100)).to_i64().unwrap_or(0);
        let shipping_cents = (shipping_total * BigDecimal::from(100))
            .to_i64()
            .unwrap_or(0);

        let result = sqlx::query!(
            r#"
            UPDATE carts 
            SET 
                subtotal = $1,
                tax_total = $2,
                shipping_total = $3,
                discount_total = $4,
                dt_updated = $5
            WHERE id = $6 AND dt_deleted IS NULL
            "#,
            subtotal_cents,
            tax_cents,
            shipping_cents,
            discount_total_cents,
            now,
            cart_id
        )
        .execute(&self.app_state.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn create_cart_item(
        &self,
        cart_id: Uuid,
        product_id: Uuid,
        variant_id: Uuid,
        unit_price: i64,
        quantity: i32,
        line_discount_total: i64,
        line_tax_total: i64,
        attributes_snapshot: Json<Value>,
    ) -> Result<bool, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now().naive_utc();

        sqlx::query!(
            r#"
            INSERT INTO cart_items (
                id,
                cart_id,
                product_id,
                variant_id,
                unit_price,
                quantity,
                line_discount_total,
                line_tax_total,
                attributes_snapshot,
                dt_created,
                dt_updated
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING
                id,
                cart_id,
                product_id,
                variant_id,
                unit_price,
                quantity,
                line_discount_total,
                line_tax_total,
                line_total,
                attributes_snapshot,
                attributes_hash,
                dt_created,
                dt_updated,
                dt_deleted
            "#,
            id,
            cart_id,
            product_id,
            variant_id,
            unit_price,
            quantity,
            line_discount_total,
            line_tax_total,
            attributes_snapshot.0,
            now,
            now
        )
        .fetch_one(&self.app_state.db)
        .await?;

        Ok(true)
    }

    pub async fn list_cart_items(&self, cart_id: Uuid) -> Result<Vec<CartItem>, sqlx::Error> {
        let rows = sqlx::query_as!(
            CartItem,
            r#"
            SELECT
                id as "id!",
                cart_id as "cart_id!",
                product_id as "product_id!",
                variant_id as "variant_id?",
                unit_price as "unit_price!",
                quantity as "quantity!",
                line_discount_total as "line_discount_total!",
                line_tax_total as "line_tax_total!",
                line_total as "line_total!",
                attributes_snapshot as "attributes_snapshot!",
                attributes_hash as "attributes_hash!",
                (dt_created AT TIME ZONE 'UTC') as "dt_created!: DateTime<Utc>",
                (dt_updated AT TIME ZONE 'UTC') as "dt_updated!: DateTime<Utc>",
                (dt_deleted AT TIME ZONE 'UTC') as "dt_deleted?: DateTime<Utc>"
            FROM
                cart_items
            WHERE
                cart_id = $1
                AND dt_deleted IS NULL
            "#,
            cart_id
        )
        .fetch_all(&self.app_state.db)
        .await?;

        Ok(rows)
    }

    /// Atualiza a quantidade de um item do carrinho
    pub async fn update_cart_item_quantity(
        &self,
        item_id: Uuid,
        new_quantity: i32,
        unit_price: i64,
    ) -> Result<bool, sqlx::Error> {
        let now = Utc::now().naive_utc();

        let result = sqlx::query!(
            r#"
            UPDATE cart_items 
            SET 
                quantity = $1,
                unit_price = $2,
                dt_updated = $3
            WHERE id = $4 AND dt_deleted IS NULL
            "#,
            new_quantity,
            unit_price,
            now,
            item_id
        )
        .execute(&self.app_state.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Remove um item do carrinho (soft delete)
    pub async fn delete_cart_item(&self, item_id: Uuid) -> Result<bool, sqlx::Error> {
        let now = Utc::now().naive_utc();

        let result = sqlx::query!(
            r#"
            UPDATE cart_items 
            SET 
                dt_deleted = $1,
                dt_updated = $2
            WHERE id = $3 AND dt_deleted IS NULL
            "#,
            now,
            now,
            item_id
        )
        .execute(&self.app_state.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
