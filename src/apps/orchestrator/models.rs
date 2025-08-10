use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Orchestrator {
    pub id: Uuid,
    pub app_name: String,
    pub app_url: String,
    pub app_token: Uuid,
    pub dt_created: DateTime<Utc>,
    pub dt_updated: DateTime<Utc>,
    pub dt_deleted: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateOrchestratorRequest {
    #[validate(length(
        min = 2,
        max = 100,
        message = "O nome do app deve ter entre 2 e 100 caracteres"
    ))]
    pub app_name: String,

    #[validate(url(message = "URL do app inválida"))]
    pub app_url: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrchestratorRequest {
    pub app_name: Option<String>,
    pub app_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OrchestratorResponse {
    pub id: Uuid,
    pub app_name: String,
    pub app_url: String,
    pub app_token: Uuid,
    pub dt_created: DateTime<Utc>,
    pub dt_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct AppAuthorizationResponse {
    pub app_name: String,
    pub status: String, // "authorized" ou "unauthorized"
}

#[derive(Debug, Deserialize, Validate)]
pub struct SyncAllUsersRequest {
    #[validate(length(
        min = 2,
        max = 100,
        message = "O nome do app deve ter entre 2 e 100 caracteres"
    ))]
    pub app_name: String,
}

impl From<Orchestrator> for OrchestratorResponse {
    fn from(orchestrator: Orchestrator) -> Self {
        Self {
            id: orchestrator.id,
            app_name: orchestrator.app_name,
            app_url: orchestrator.app_url,
            app_token: orchestrator.app_token,
            dt_created: orchestrator.dt_created,
            dt_updated: orchestrator.dt_updated,
        }
    }
}
