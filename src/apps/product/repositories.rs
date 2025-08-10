use crate::app_core::app_state::AppState;
use crate::apps::product::models::{
    CreateProductRequest, Product, ProductListParams, UpdateProductRequest,
};
use crate::utils::pagination::PaginatedResponse;
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::{Postgres, QueryBuilder, Row, postgres::PgRow};
use uuid::Uuid;

fn cents_to_bigdecimal(cents: i64) -> BigDecimal {
    BigDecimal::from(cents) / BigDecimal::from(100)
}

pub struct ProductRepository<'a> {
    app_state: &'a AppState,
}

impl<'a> ProductRepository<'a> {
    pub fn new(app_state: &'a AppState) -> Self {
        Self { app_state }
    }

    pub async fn find_all(
        &self,
        params: ProductListParams,
    ) -> Result<PaginatedResponse<Product>, sqlx::Error> {
        let limit = params.limit.unwrap_or(20).clamp(1, 100); // limites saudáveis
        let offset = params.offset.unwrap_or(0).max(0);

        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT
            p.id,
            p.tenant_id,
            p.name,
            p.slug,
            p.short_description,
            p.description,
            p.price,
            p.stock_quantity,
            p.attributes,
            p.is_active,
            p.dt_created,
            p.dt_updated,
            p.dt_deleted,
            COUNT(*) OVER() AS total_count
         FROM products p
         WHERE ",
        );

        // WHERE obrigatório: tenant, não deletado
        {
            let mut cond = qb.separated(" AND ");
            // cond.push("p.tenant_id = ").push_bind(tenant_id);
            cond.push("p.dt_deleted IS NULL");
        }

        // Filtros opcionais
        if let Some(name) = params.name.filter(|s| !s.trim().is_empty()) {
            qb.push(" AND p.name ILIKE ")
                .push_bind(format!("%{}%", name));
        }

        if let Some(min_cents) = params.min_price {
            let bd = cents_to_bigdecimal(min_cents);
            qb.push(" AND p.price >= ").push_bind(bd);
        }

        if let Some(max_cents) = params.max_price {
            let bd = cents_to_bigdecimal(max_cents);
            qb.push(" AND p.price <= ").push_bind(bd);
        }

        if let Some(is_active) = params.is_active {
            qb.push(" AND p.is_active = ").push_bind(is_active);
        }

        // Ordenação padrão (ajuste se preferir)
        qb.push(" ORDER BY p.dt_created DESC, p.id DESC");

        // Paginação
        qb.push(" LIMIT ").push_bind(limit);
        qb.push(" OFFSET ").push_bind(offset);

        let rows: Vec<PgRow> = qb.build().fetch_all(&self.app_state.db).await?;

        // Extrai total de forma segura
        let total = rows
            .get(0)
            .map(|r| r.get::<i64, _>("total_count"))
            .unwrap_or(0);

        let items = rows
            .into_iter()
            .map(|row| Product {
                id: row.get("id"),
                tenant_id: row.get("tenant_id"),
                name: row.get("name"),
                slug: row.get("slug"),
                short_description: row.get("short_description"),
                description: row.get("description"),
                price: row.get("price"),
                stock_quantity: row.get("stock_quantity"),
                attributes: row.get("attributes"),
                is_active: row.get("is_active"),
                dt_created: DateTime::from_naive_utc_and_offset(
                    row.get::<NaiveDateTime, _>("dt_created"),
                    Utc,
                ),
                dt_updated: DateTime::from_naive_utc_and_offset(
                    row.get::<NaiveDateTime, _>("dt_updated"),
                    Utc,
                ),
                dt_deleted: row
                    .get::<Option<NaiveDateTime>, _>("dt_deleted")
                    .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
            })
            .collect();

        Ok(PaginatedResponse {
            count: total,
            results: items,
            limit,
            offset,
        })

        // let rows =
        //     sqlx::query!("SELECT * FROM products WHERE dt_deleted IS NULL AND is_active = true")
        //         .fetch_all(&self.app_state.db)
        //         .await?;

        // let products = rows
        //     .into_iter()
        //     .map(|row| Product {
        //         id: row.id,
        //         tenant_id: row.tenant_id,
        //         name: row.name,
        //         slug: row.slug,
        //         short_description: row.short_description,
        //         description: row.description,
        //         price: row.price,
        //         stock_quantity: row.stock_quantity,
        //         attributes: row.attributes,
        //         is_active: row.is_active,
        //         dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
        //         dt_updated: DateTime::from_naive_utc_and_offset(row.dt_updated, Utc),
        //         dt_deleted: row
        //             .dt_deleted
        //             .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        //     })
        //     .collect();

        // Ok(products)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Product>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT * FROM products WHERE id = $1 AND dt_deleted IS NULL AND is_active = true",
            id
        )
        .fetch_optional(&self.app_state.db)
        .await?;

        Ok(row.map(|row| Product {
            id: row.id,
            tenant_id: row.tenant_id,
            name: row.name,
            slug: row.slug,
            short_description: row.short_description,
            description: row.description,
            price: row.price,
            stock_quantity: row.stock_quantity,
            attributes: row.attributes,
            is_active: row.is_active,
            dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
            dt_updated: DateTime::from_naive_utc_and_offset(row.dt_updated, Utc),
            dt_deleted: row
                .dt_deleted
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        }))
    }

    pub async fn create(
        &self,
        _request: CreateProductRequest,
        tenant_id: Uuid,
    ) -> Result<Product, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let price_bd = cents_to_bigdecimal(_request.price);

        let row = sqlx::query!(
            r#"
            INSERT INTO products (id,tenant_id,name,slug,short_description,description,price,stock_quantity,attributes,is_active,dt_created,dt_updated)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
            RETURNING id, tenant_id, name, slug, short_description, description, price, stock_quantity, attributes, is_active, dt_created, dt_updated, dt_deleted
            "#,
            id,
            tenant_id,
            _request.name,
            _request.name,
            _request.short_description,
            _request.description,
            price_bd,
            _request.stock_quantity,
            _request.attributes,
            _request.is_active,
            now.naive_utc(),
            now.naive_utc()
        )
        .fetch_one(&self.app_state.db)
        .await?;

        Ok(Product {
            id: row.id,
            tenant_id: row.tenant_id,
            name: row.name,
            slug: row.slug,
            short_description: row.short_description,
            description: row.description,
            price: row.price,
            stock_quantity: row.stock_quantity,
            attributes: row.attributes,
            is_active: row.is_active,
            dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
            dt_updated: DateTime::from_naive_utc_and_offset(row.dt_updated, Utc),
            dt_deleted: row
                .dt_deleted
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        })
    }

    pub async fn update(
        &self,
        id: Uuid,
        tenant_id: Uuid,
        request: UpdateProductRequest,
    ) -> Result<Option<Product>, sqlx::Error> {
        let now = Utc::now().naive_utc();

        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE products SET ");
        let mut first = true;

        // helper pra inserir vírgula só depois do primeiro campo
        let mut push_set = |qb: &mut QueryBuilder<Postgres>, sql_prefix: &str| {
            if !first {
                qb.push(", ");
            } else {
                first = false;
            }
            qb.push(sql_prefix);
        };

        // dt_updated sempre
        push_set(&mut qb, "dt_updated = ");
        qb.push_bind(now);

        if let Some(name) = request.name {
            push_set(&mut qb, "name = ");
            qb.push_bind(name);
        }
        if let Some(short_description) = request.short_description {
            push_set(&mut qb, "short_description = ");
            qb.push_bind(short_description);
        }
        if let Some(description) = request.description {
            push_set(&mut qb, "description = ");
            qb.push_bind(description);
        }
        if let Some(price_cents) = request.price {
            let bd = cents_to_bigdecimal(price_cents);
            push_set(&mut qb, "price = ");
            qb.push_bind(bd);
        }
        if let Some(stock_quantity) = request.stock_quantity {
            push_set(&mut qb, "stock_quantity = ");
            qb.push_bind(stock_quantity);
        }
        if let Some(attributes) = request.attributes {
            push_set(&mut qb, "attributes = ");
            qb.push_bind(attributes);
        }
        if let Some(is_active) = request.is_active {
            push_set(&mut qb, "is_active = ");
            qb.push_bind(is_active);
        }

        qb.push(" WHERE id = ").push_bind(id);
        qb.push(" AND tenant_id = ").push_bind(tenant_id);
        qb.push(" AND dt_deleted IS NULL");
        qb.push(
            " RETURNING
            id, tenant_id, name, slug, short_description, description,
            price, stock_quantity, attributes, is_active,
            dt_created, dt_updated, dt_deleted",
        );

        let row_opt = qb.build().fetch_optional(&self.app_state.db).await?;

        Ok(row_opt.map(|row| Product {
            id: row.get("id"),
            tenant_id: row.get("tenant_id"),
            name: row.get("name"),
            slug: row.get("slug"),
            short_description: row.get("short_description"),
            description: row.get("description"),
            price: row.get("price"),
            stock_quantity: row.get("stock_quantity"),
            attributes: row.get("attributes"),
            is_active: row.get("is_active"),
            dt_created: DateTime::from_naive_utc_and_offset(row.get("dt_created"), Utc),
            dt_updated: DateTime::from_naive_utc_and_offset(row.get("dt_updated"), Utc),
            dt_deleted: row
                .get::<Option<NaiveDateTime>, _>("dt_deleted")
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        }))
    }

    pub async fn delete(&self, id: Uuid, tenant_id: Uuid) -> Result<bool, sqlx::Error> {
        let exists = sqlx::query!(
            "SELECT * FROM products WHERE id = $1 AND tenant_id = $2 AND dt_deleted IS NULL AND is_active = true",
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
            "UPDATE products SET dt_deleted = $1 WHERE id = $2 AND tenant_id = $3 AND dt_deleted IS NULL",
            now.naive_utc(),
            id,
            tenant_id
        )
        .execute(&self.app_state.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
