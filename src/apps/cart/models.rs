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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddProductCart {
    pub product_id: Uuid,
    pub quantity: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteProductCart {
    pub product_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CartItem {
    pub id: Uuid,
    pub cart_id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub unit_price: i64, // em centavos
    pub quantity: i32,
    pub line_discount_total: i64, // em centavos
    pub line_tax_total: i64,      // em centavos
    pub line_total: i64,          // em centavos - calculado automaticamente
    pub attributes_snapshot: serde_json::Value,
    pub attributes_hash: String, // calculado automaticamente
    pub dt_created: DateTime<Utc>,
    pub dt_updated: DateTime<Utc>,
    pub dt_deleted: Option<DateTime<Utc>>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CartWithItems {
    // Dados do carrinho
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

    // Lista de produtos no carrinho
    pub items: Vec<CartItemWithProduct>,

    // Metadados úteis
    pub item_count: usize,      // Quantidade total de itens
    pub unique_products: usize, // Quantidade de produtos únicos
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CartItemResponse {
    pub id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub unit_price: i64, // em centavos
    pub quantity: i32,
    pub line_total: i64, // em centavos
    pub attributes_snapshot: serde_json::Value,
    pub dt_created: DateTime<Utc>,
}

// Nova struct que inclui os dados do produto
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CartItemWithProduct {
    // Dados do item do carrinho
    pub id: Uuid,
    pub cart_id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub unit_price: i64, // em centavos
    pub quantity: i32,
    pub line_discount_total: i64, // em centavos
    pub line_tax_total: i64,      // em centavos
    pub line_total: i64,          // em centavos
    pub attributes_snapshot: serde_json::Value,
    pub attributes_hash: String,
    pub dt_created: DateTime<Utc>,
    pub dt_updated: DateTime<Utc>,
    pub dt_deleted: Option<DateTime<Utc>>,

    // Dados do produto
    pub product_name: String,
    pub product_slug: String,
    pub product_short_description: Option<String>,
    pub product_description: Option<String>,
    pub product_price: BigDecimal,
    pub product_stock_quantity: i32,
    pub product_is_active: bool,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CartSummary {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub status: CartStatus,
    pub currency: String,
    #[serde_as(as = "DisplayFromStr")]
    pub subtotal: BigDecimal,
    #[serde_as(as = "DisplayFromStr")]
    pub grand_total: BigDecimal,
    pub item_count: usize,
    pub dt_updated: DateTime<Utc>,
}

// Implementação de métodos úteis para CartWithItems
#[allow(dead_code)]
impl CartWithItems {
    /// Cria um novo CartWithItems a partir de um Cart e uma lista de itens
    pub fn from_cart_and_items(cart: Cart, items: Vec<CartItem>) -> Self {
        let item_count = items.iter().map(|item| item.quantity as usize).sum();
        let unique_products = items.len();

        Self {
            id: cart.id,
            tenant_id: cart.tenant_id,
            user_id: cart.user_id,
            status: cart.status,
            currency: cart.currency,
            subtotal: cart.subtotal,
            discount_total: cart.discount_total,
            tax_total: cart.tax_total,
            shipping_total: cart.shipping_total,
            grand_total: cart.grand_total,
            version: cart.version,
            expires_at: cart.expires_at,
            dt_created: cart.dt_created,
            dt_updated: cart.dt_updated,
            dt_deleted: cart.dt_deleted,
            items: vec![], // Será preenchido depois
            item_count,
            unique_products,
        }
    }

    /// Cria um novo CartWithItems com produtos populados
    pub fn from_cart_and_items_with_products(
        cart: Cart,
        items: Vec<CartItem>,
        products: Vec<crate::apps::product::models::Product>,
    ) -> Self {
        let item_count = items.iter().map(|item| item.quantity as usize).sum();
        let unique_products = items.len();

        // Converter CartItem para CartItemWithProduct
        let items_with_products: Vec<CartItemWithProduct> = items
            .into_iter()
            .filter_map(|item| {
                // Encontrar o produto correspondente
                let product = products.iter().find(|p| p.id == item.product_id)?;

                Some(CartItemWithProduct {
                    // Dados do item
                    id: item.id,
                    cart_id: item.cart_id,
                    product_id: item.product_id,
                    variant_id: item.variant_id,
                    unit_price: item.unit_price,
                    quantity: item.quantity,
                    line_discount_total: item.line_discount_total,
                    line_tax_total: item.line_tax_total,
                    line_total: item.line_total,
                    attributes_snapshot: item.attributes_snapshot,
                    attributes_hash: item.attributes_hash,
                    dt_created: item.dt_created,
                    dt_updated: item.dt_updated,
                    dt_deleted: item.dt_deleted,

                    // Dados do produto
                    product_name: product.name.clone(),
                    product_slug: product.slug.clone(),
                    product_short_description: product.short_description.clone(),
                    product_description: product.description.clone(),
                    product_price: product.price.clone(),
                    product_stock_quantity: product.stock_quantity,
                    product_is_active: product.is_active,
                })
            })
            .collect();

        Self {
            id: cart.id,
            tenant_id: cart.tenant_id,
            user_id: cart.user_id,
            status: cart.status,
            currency: cart.currency,
            subtotal: cart.subtotal,
            discount_total: cart.discount_total,
            tax_total: cart.tax_total,
            shipping_total: cart.shipping_total,
            grand_total: cart.grand_total,
            version: cart.version,
            expires_at: cart.expires_at,
            dt_created: cart.dt_created,
            dt_updated: cart.dt_updated,
            dt_deleted: cart.dt_deleted,
            items: items_with_products,
            item_count,
            unique_products,
        }
    }

    /// Calcula o total de itens no carrinho
    pub fn total_items(&self) -> usize {
        self.items.iter().map(|item| item.quantity as usize).sum()
    }

    /// Verifica se o carrinho está vazio
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Obtém um item específico por ID
    pub fn get_item(&self, item_id: Uuid) -> Option<&CartItemWithProduct> {
        self.items.iter().find(|item| item.id == item_id)
    }

    /// Obtém todos os produtos únicos no carrinho
    pub fn unique_product_ids(&self) -> Vec<Uuid> {
        self.items.iter().map(|item| item.product_id).collect()
    }
}
