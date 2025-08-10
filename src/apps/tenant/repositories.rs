use crate::app_core::app_state::AppState;
use crate::apps::tenant::models::{Tenant, UpdateTenantRequest};
use chrono::{DateTime, Utc};
// use sqlx::Row;
use uuid::Uuid;

pub struct TenantRepository<'a> {
    app_state: &'a AppState,
}

#[allow(dead_code)]
impl<'a> TenantRepository<'a> {
    pub fn new(app_state: &'a AppState) -> Self {
        Self { app_state }
    }

    pub async fn find_all(&self) -> Result<Vec<Tenant>, sqlx::Error> {
        let rows = sqlx::query!(
            "SELECT id, user_id, tenant_type, dt_created, dt_updated, dt_deleted FROM tenants WHERE dt_deleted IS NULL"
        )
        .fetch_all(&self.app_state.db)
        .await?;

        let tenants = rows
            .into_iter()
            .map(|row| Tenant {
                id: row.id,
                user_id: row.user_id,
                tenant_type: row.tenant_type,
                dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
                dt_updated: DateTime::from_naive_utc_and_offset(row.dt_updated, Utc),
                dt_deleted: row
                    .dt_deleted
                    .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
            })
            .collect();

        Ok(tenants)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Tenant>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT id, user_id, tenant_type, dt_created, dt_updated, dt_deleted FROM tenants WHERE id = $1 AND dt_deleted IS NULL",
            id
        )
        .fetch_optional(&self.app_state.db)
        .await?;

        Ok(row.map(|row| Tenant {
            id: row.id,
            user_id: row.user_id,
            tenant_type: row.tenant_type,
            dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
            dt_updated: DateTime::from_naive_utc_and_offset(row.dt_updated, Utc),
            dt_deleted: row
                .dt_deleted
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        }))
    }

    pub async fn find_by_user_id(&self, user_id: Uuid) -> Result<Tenant, sqlx::Error> {
        if let Some(tenant_user) = sqlx::query!(
            "SELECT id, user_id, tenant_id, dt_created, dt_updated, dt_deleted FROM tenant_users WHERE user_id = $1 AND dt_deleted IS NULL",
            user_id
        )
        .fetch_optional(&self.app_state.db)
        .await?
        {
            let row = sqlx::query!(
                "SELECT id, user_id, tenant_type, dt_created, dt_updated, dt_deleted FROM tenants WHERE id = $1 AND dt_deleted IS NULL",
                tenant_user.tenant_id
            )
            .fetch_one(&self.app_state.db)
            .await?;

            Ok(Tenant {
                id: row.id,
                user_id: user_id,
                tenant_type: row.tenant_type.to_string(),
                dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
                dt_updated: DateTime::from_naive_utc_and_offset(row.dt_updated, Utc),
                dt_deleted: row
                    .dt_deleted
                    .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
            })
        } else {
            // Se não existe tenant_user, cria um tenant padrão e retorna diretamente
            self.create(user_id, "default").await
        }
    }

    pub async fn create(&self, user_id: Uuid, tenant_type: &str) -> Result<Tenant, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let row = sqlx::query!(
            "INSERT INTO tenants (id, user_id, tenant_type, dt_created, dt_updated) VALUES ($1, $2, $3, $4, $5) RETURNING id, dt_created, dt_updated, dt_deleted",
            id,
            user_id,
            tenant_type,
            now.naive_utc(),
            now.naive_utc()
        )
        .fetch_one(&self.app_state.db)
        .await?;

        sqlx::query!(
            "INSERT INTO tenant_users (id, user_id, tenant_id, dt_created, dt_updated) VALUES ($1, $2, $3, $4, $5) RETURNING id, dt_created, dt_updated, dt_deleted",
            Uuid::new_v4(),
            user_id,
            row.id,
            now.naive_utc(),
            now.naive_utc()
        )
        .fetch_one(&self.app_state.db)
        .await?;

        Ok(Tenant {
            id: row.id,
            user_id: user_id,
            tenant_type: tenant_type.to_string(),
            dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
            dt_updated: DateTime::from_naive_utc_and_offset(row.dt_updated, Utc),
            dt_deleted: row
                .dt_deleted
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        })
    }

    pub async fn create_tenant_user(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<Tenant, sqlx::Error> {
        let now = Utc::now();

        sqlx::query!(
            "INSERT INTO tenant_users (id, user_id, tenant_id, dt_created, dt_updated) VALUES ($1, $2, $3, $4, $5) RETURNING id, dt_created, dt_updated, dt_deleted",
            Uuid::new_v4(),
            user_id,
            tenant_id,
            now.naive_utc(),
            now.naive_utc()
        )
        .fetch_one(&self.app_state.db)
        .await?;

        let row = sqlx::query!(
            "SELECT id, user_id, tenant_type, dt_created, dt_updated, dt_deleted FROM tenants WHERE id = $1 AND dt_deleted IS NULL",
            tenant_id
        )
        .fetch_one(&self.app_state.db)
        .await?;

        Ok(Tenant {
            id: row.id,
            user_id: row.user_id,
            tenant_type: row.tenant_type,
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
        _request: UpdateTenantRequest,
    ) -> Result<Option<Tenant>, sqlx::Error> {
        let now = Utc::now();

        let row = sqlx::query!(
            "UPDATE tenants SET dt_updated = $1 WHERE id = $2 AND dt_deleted IS NULL RETURNING id, user_id, tenant_type, dt_created, dt_updated, dt_deleted",
            now.naive_utc(),
            id
        )
        .fetch_optional(&self.app_state.db)
        .await?;

        Ok(row.map(|row| Tenant {
            id: row.id,
            user_id: row.user_id,
            tenant_type: row.tenant_type,
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
            "UPDATE tenants SET dt_deleted = $1 WHERE id = $2 AND dt_deleted IS NULL",
            now.naive_utc(),
            id
        )
        .execute(&self.app_state.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
