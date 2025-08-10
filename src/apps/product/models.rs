use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use uuid::Uuid;

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub slug: String,
    pub short_description: Option<String>,
    pub description: Option<String>,
    #[serde_as(as = "DisplayFromStr")]
    pub price: BigDecimal,
    pub stock_quantity: i32,
    pub attributes: Option<serde_json::Value>,
    pub is_active: bool,
    pub dt_created: DateTime<Utc>,
    pub dt_updated: DateTime<Utc>,
    pub dt_deleted: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub short_description: Option<String>,
    pub description: Option<String>,
    pub price: i64,
    pub stock_quantity: i32,
    pub attributes: Option<serde_json::Value>,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub short_description: Option<String>,
    pub description: Option<String>,
    pub price: Option<i64>,
    pub stock_quantity: Option<i32>,
    pub attributes: Option<serde_json::Value>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub id: Uuid,
    pub dt_created: DateTime<Utc>,
    pub dt_updated: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ProductListParams {
    pub name: Option<String>,
    pub min_price: Option<i64>,
    pub max_price: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub is_active: Option<bool>,
}
