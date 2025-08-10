use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Tenant {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_type: String,
    pub dt_created: DateTime<Utc>,
    pub dt_updated: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub dt_deleted: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CreateTenantRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_type: String,
    pub dt_created: DateTime<Utc>,
    pub dt_updated: DateTime<Utc>,
    pub dt_deleted: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTenantRequest {
    // Adicione os campos necessários aqui
}

#[derive(Debug, Serialize)]
pub struct TenantResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_type: String,
    pub dt_created: DateTime<Utc>,
    pub dt_updated: DateTime<Utc>,
}
