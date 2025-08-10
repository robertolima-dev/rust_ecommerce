use crate::app_core::app_state::AppState;
use crate::apps::orchestrator::models::{
    CreateOrchestratorRequest, Orchestrator, UpdateOrchestratorRequest,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

pub struct OrchestratorRepository<'a> {
    app_state: &'a AppState,
}

impl<'a> OrchestratorRepository<'a> {
    pub fn new(app_state: &'a AppState) -> Self {
        Self { app_state }
    }

    pub async fn find_all(&self) -> Result<Vec<Orchestrator>, sqlx::Error> {
        let rows = sqlx::query!(
            "SELECT id, app_name, app_url, app_token, dt_created, dt_updated, dt_deleted FROM orchestrators WHERE dt_deleted IS NULL"
        )
        .fetch_all(&self.app_state.db)
        .await?;

        let orchestrators = rows
            .into_iter()
            .map(|row| Orchestrator {
                id: row.id,
                app_name: row.app_name,
                app_url: row.app_url,
                app_token: row.app_token,
                dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
                dt_updated: DateTime::from_naive_utc_and_offset(row.dt_updated, Utc),
                dt_deleted: row
                    .dt_deleted
                    .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
            })
            .collect();

        Ok(orchestrators)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Orchestrator>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT id, app_name, app_url, app_token, dt_created, dt_updated, dt_deleted FROM orchestrators WHERE id = $1 AND dt_deleted IS NULL",
            id
        )
        .fetch_optional(&self.app_state.db)
        .await?;

        Ok(row.map(|row| Orchestrator {
            id: row.id,
            app_name: row.app_name,
            app_url: row.app_url,
            app_token: row.app_token,
            dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
            dt_updated: DateTime::from_naive_utc_and_offset(row.dt_updated, Utc),
            dt_deleted: row
                .dt_deleted
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        }))
    }

    pub async fn find_by_app_token(
        &self,
        app_token: Uuid,
    ) -> Result<Option<Orchestrator>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT id, app_name, app_url, app_token, dt_created, dt_updated, dt_deleted FROM orchestrators WHERE app_token = $1 AND dt_deleted IS NULL",
            app_token
        )
        .fetch_optional(&self.app_state.db)
        .await?;

        Ok(row.map(|row| Orchestrator {
            id: row.id,
            app_name: row.app_name,
            app_url: row.app_url,
            app_token: row.app_token,
            dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
            dt_updated: DateTime::from_naive_utc_and_offset(row.dt_updated, Utc),
            dt_deleted: row
                .dt_deleted
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        }))
    }

    pub async fn create(
        &self,
        request: CreateOrchestratorRequest,
    ) -> Result<Orchestrator, sqlx::Error> {
        let id = Uuid::new_v4();
        let app_token = Uuid::new_v4();
        let now = Utc::now();

        let row = sqlx::query!(
            "INSERT INTO orchestrators (id, app_name, app_url, app_token, dt_created, dt_updated) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, app_name, app_url, app_token, dt_created, dt_updated, dt_deleted",
            id,
            request.app_name,
            request.app_url,
            app_token,
            now.naive_utc(),
            now.naive_utc()
        )
        .fetch_one(&self.app_state.db)
        .await?;

        Ok(Orchestrator {
            id: row.id,
            app_name: row.app_name,
            app_url: row.app_url,
            app_token: row.app_token,
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
        request: UpdateOrchestratorRequest,
    ) -> Result<Option<Orchestrator>, sqlx::Error> {
        let now = Utc::now();

        // Construir a query dinamicamente baseada nos campos fornecidos
        let mut query = String::from("UPDATE orchestrators SET dt_updated = $1");
        let mut params: Vec<String> = vec![];
        let mut param_count = 1;

        if let Some(app_name) = &request.app_name {
            param_count += 1;
            query.push_str(&format!(", app_name = ${}", param_count));
            params.push(app_name.clone());
        }

        if let Some(app_url) = &request.app_url {
            param_count += 1;
            query.push_str(&format!(", app_url = ${}", param_count));
            params.push(app_url.clone());
        }

        param_count += 1;
        query.push_str(&format!(" WHERE id = ${} AND dt_deleted IS NULL RETURNING id, app_name, app_url, app_token, dt_created, dt_updated, dt_deleted", param_count));

        let mut query_builder = sqlx::query(&query);
        query_builder = query_builder.bind(now.naive_utc());

        for param in params {
            query_builder = query_builder.bind(param);
        }

        query_builder = query_builder.bind(id);

        let row = query_builder.fetch_optional(&self.app_state.db).await?;

        Ok(row.map(|row| Orchestrator {
            id: row.get("id"),
            app_name: row.get("app_name"),
            app_url: row.get("app_url"),
            app_token: row.get("app_token"),
            dt_created: DateTime::from_naive_utc_and_offset(row.get("dt_created"), Utc),
            dt_updated: DateTime::from_naive_utc_and_offset(row.get("dt_updated"), Utc),
            dt_deleted: row
                .get::<Option<NaiveDateTime>, _>("dt_deleted")
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        }))
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        let now = Utc::now();

        let result = sqlx::query!(
            "UPDATE orchestrators SET dt_deleted = $1 WHERE id = $2 AND dt_deleted IS NULL",
            now.naive_utc(),
            id
        )
        .execute(&self.app_state.db)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
