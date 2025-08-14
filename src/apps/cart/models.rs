use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cart {
    pub id: Uuid,
    pub dt_created: DateTime<Utc>,
    pub dt_updated: DateTime<Utc>,
    pub dt_deleted: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCartRequest {
    // Adicione os campos necessários aqui
}

#[derive(Debug, Deserialize)]
pub struct UpdateCartRequest {
    // Adicione os campos necessários aqui
}

#[derive(Debug, Serialize)]
pub struct CartResponse {
    pub id: Uuid,
    pub dt_created: DateTime<Utc>,
    pub dt_updated: DateTime<Utc>,
}
