use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "cart_status", rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(non_camel_case_types)]
pub enum CartStatus {
    ACTIVE,
    CHECKOUT_IN_PROGRESS,
    CONVERTED_TO_ORDER,
    ABANDONED,
    CANCELLED,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cart {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub status: CartStatus,
    pub currency: String,
    #[serde_as(as = "DisplayFromStr")]
    pub subtotal: BigDecimal,
    #[serde_as(as = "DisplayFromStr")]
    pub discount_total: BigDecimal,
    #[serde_as(as = "DisplayFromStr")]
    pub tax_total: BigDecimal,
    #[serde_as(as = "DisplayFromStr")]
    pub shipping_total: BigDecimal,
    #[serde_as(as = "DisplayFromStr")]
    pub grand_total: BigDecimal,
    pub version: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub dt_created: DateTime<Utc>,
    pub dt_updated: DateTime<Utc>,
    pub dt_deleted: Option<DateTime<Utc>>,
}
