use crate::app_core::app_state::AppState;
use crate::apps::cart::models::{Cart, CreateCartRequest, UpdateCartRequest};
use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

pub struct CartRepository<'a> {
    app_state: &'a AppState,
}

impl<'a> CartRepository<'a> {
    pub fn new(app_state: &'a AppState) -> Self {
        Self { app_state }
    }

    pub async fn find_all(&self) -> Result<Vec<Cart>, sqlx::Error> {
        let rows = sqlx::query!(
            "SELECT id, dt_created, dt_updated, dt_deleted FROM carts WHERE dt_deleted IS NULL"
        )
        .fetch_all(&self.app_state.db)
        .await?;

        let carts = rows
            .into_iter()
            .map(|row| Cart {
                id: row.id,
                dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
                dt_updated: DateTime::from_naive_utc_and_offset(row.dt_updated, Utc),
                dt_deleted: row
                    .dt_deleted
                    .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
            })
            .collect();

        Ok(carts)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Cart>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT id, dt_created, dt_updated, dt_deleted FROM carts WHERE id = $1 AND dt_deleted IS NULL",
            id
        )
        .fetch_optional(&self.app_state.db)
        .await?;

        Ok(row.map(|row| Cart {
            id: row.id,
            dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
            dt_updated: DateTime::from_naive_utc_and_offset(row.dt_updated, Utc),
            dt_deleted: row
                .dt_deleted
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        }))
    }

    pub async fn create(&self, _request: CreateCartRequest) -> Result<Cart, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let row = sqlx::query!(
            "INSERT INTO carts (id, dt_created, dt_updated) VALUES ($1, $2, $3) RETURNING id, dt_created, dt_updated, dt_deleted",
            id,
            now.naive_utc(),
            now.naive_utc()
        )
        .fetch_one(&self.app_state.db)
        .await?;

        Ok(Cart {
            id: row.id,
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
        _request: UpdateCartRequest,
    ) -> Result<Option<Cart>, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query!(
            "UPDATE carts SET dt_updated = $1 WHERE id = $2 AND dt_deleted IS NULL RETURNING id, dt_created, dt_updated, dt_deleted",
            now.naive_utc(),
            id
        )
        .fetch_optional(&self.app_state.db)
        .await?;

        Ok(row.map(|row| Cart {
            id: row.id,
            dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
            dt_updated: DateTime::from_naive_utc_and_offset(row.dt_updated, Utc),
            dt_deleted: row
                .dt_deleted
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        }))
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let now = Utc::now();

        let result = sqlx::query!(
            "UPDATE carts SET dt_deleted = $1 WHERE id = $2 AND dt_deleted IS NULL",
            now.naive_utc(),
            id
        )
        .execute(&self.app_state.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
