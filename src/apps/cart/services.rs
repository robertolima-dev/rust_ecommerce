use crate::app_core::app_error::AppError;
use crate::app_core::app_state::AppState;
use crate::apps::cart::models::{AddProductCart, Cart, CartWithItems, DeleteProductCart};
use crate::apps::cart::repositories::CartRepository;
use crate::apps::product::repositories::ProductRepository;
use bigdecimal::{BigDecimal, ToPrimitive};
use sqlx::types::Json;
use uuid::Uuid;

pub struct CartService;

impl CartService {
    pub async fn list_cards(app_state: &AppState, tenant_id: Uuid) -> Result<Vec<Cart>, AppError> {
        let repository = CartRepository::new(app_state);
        let carts = repository
            .find_all(tenant_id)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        Ok(carts)
    }

    pub async fn get_cart(
        app_state: &AppState,
        tenant_id: Uuid,
    ) -> Result<CartWithItems, AppError> {
        let repository = CartRepository::new(app_state);
        let cart = repository
            .find_by_tenant_id(tenant_id)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        let cart = match cart {
            Some(cart) => cart,
            None => return Err(AppError::not_found("Carrinho não encontrado")),
        };

        let list_cart_items = repository.list_cart_items(cart.id).await?;

        // Buscar os produtos dos itens do carrinho
        let product_ids: Vec<Uuid> = list_cart_items.iter().map(|item| item.product_id).collect();

        let products = if !product_ids.is_empty() {
            let repo_product = ProductRepository::new(app_state);
            repo_product
                .find_by_ids(&product_ids)
                .await
                .map_err(|e| AppError::database_error(e.to_string()))?
        } else {
            vec![]
        };

        let cart_with_items =
            CartWithItems::from_cart_and_items_with_products(cart, list_cart_items, products);

        Ok(cart_with_items)
    }

    pub async fn create_cart(
        app_state: &AppState,
        user_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<Cart, AppError> {
        let repository = CartRepository::new(app_state);
        repository
            .create(tenant_id, user_id)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))
    }

    pub async fn delete_cart(
        app_state: &AppState,
        id: Uuid,
        tenant_id: Uuid,
    ) -> Result<bool, AppError> {
        let repository = CartRepository::new(app_state);
        let deleted = repository
            .delete(id, tenant_id)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        match deleted {
            true => Ok(deleted),
            false => Err(AppError::not_found("Carrinho não encontrado")),
        }
    }

    pub async fn add_product_cart_by_tenant(
        app_state: &AppState,
        request: AddProductCart,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<CartWithItems, AppError> {
        let repo_cart = CartRepository::new(app_state);
        let repo_product = ProductRepository::new(app_state);

        let product_id = request.product_id;
        let quantity = request.quantity;

        let product = repo_product
            .find_by_id(product_id)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        let product = match product {
            Some(product) => product,
            None => return Err(AppError::not_found("Produto não encontrado")),
        };

        if product.stock_quantity < quantity {
            return Err(AppError::bad_request("Estoque insuficiente"));
        }

        if product.tenant_id == tenant_id {
            return Err(AppError::bad_request("Produto pertence a sua conta"));
        }

        let cart = Self::get_or_create_cart(app_state, tenant_id, user_id).await?;

        // Verificar se o produto já existe no carrinho
        let existing_items = repo_cart.list_cart_items(cart.id).await?;
        let existing_item = existing_items
            .iter()
            .find(|item| item.product_id == product_id);

        match existing_item {
            Some(existing_item) => {
                // PRODUTO JÁ EXISTE: Atualizar quantidade
                println!("🔄 Produto já existe no carrinho. Atualizando quantidade...");

                let new_quantity = existing_item.quantity + quantity;

                // Verificar estoque novamente com a nova quantidade total
                if product.stock_quantity < new_quantity {
                    return Err(AppError::bad_request(
                        "Estoque insuficiente para a quantidade solicitada",
                    ));
                }

                // Atualizar quantidade do item existente
                repo_cart
                    .update_cart_item_quantity(
                        existing_item.id,
                        new_quantity,
                        (product.price * BigDecimal::from(100))
                            .to_i64()
                            .unwrap_or(0),
                    )
                    .await?;

                println!(
                    "✅ Quantidade atualizada: {} -> {}",
                    existing_item.quantity, new_quantity
                );
            }
            None => {
                // PRODUTO NÃO EXISTE: Criar novo item
                println!("🆕 Produto não existe no carrinho. Criando novo item...");

                repo_cart
                    .create_cart_item(
                        cart.id,
                        product.id,
                        Uuid::nil(),
                        (product.price * BigDecimal::from(100))
                            .to_i64()
                            .unwrap_or(0),
                        quantity,
                        0,
                        0,
                        Json(serde_json::json!({})),
                    )
                    .await?;

                println!("✅ Novo item criado com quantidade: {}", quantity);
            }
        }

        // Recalcular totais do carrinho
        let updated_items = repo_cart.list_cart_items(cart.id).await?;
        let new_subtotal: BigDecimal = updated_items
            .iter()
            .map(|item| {
                let item_price = BigDecimal::from(item.unit_price) / BigDecimal::from(100);
                item_price * BigDecimal::from(item.quantity)
            })
            .sum();

        // Atualizar dados do carrinho no banco
        repo_cart
            .update_cart_data(
                cart.id,
                &new_subtotal,
                &cart.discount_total,
                &cart.tax_total,
                &cart.shipping_total,
            )
            .await?;

        let cart = repo_cart.find_by_tenant_id(tenant_id).await?;

        let cart = match cart {
            Some(cart) => cart,
            None => return Err(AppError::not_found("Produto não encontrado")),
        };

        let list_cart_items = repo_cart.list_cart_items(cart.id).await?;

        // Buscar os produtos dos itens do carrinho
        let product_ids: Vec<Uuid> = list_cart_items.iter().map(|item| item.product_id).collect();

        let products = if !product_ids.is_empty() {
            repo_product
                .find_by_ids(&product_ids)
                .await
                .map_err(|e| AppError::database_error(e.to_string()))?
        } else {
            vec![]
        };

        let cart_with_items =
            CartWithItems::from_cart_and_items_with_products(cart, list_cart_items, products);

        Ok(cart_with_items)
    }

    pub async fn delete_product_cart_by_tenant(
        app_state: &AppState,
        request: DeleteProductCart,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<CartWithItems, AppError> {
        let repo_cart = CartRepository::new(app_state);

        let product_id = request.product_id;

        let cart = Self::get_or_create_cart(app_state, tenant_id, user_id).await?;

        // Verificar se o produto existe no carrinho
        let existing_items = repo_cart.list_cart_items(cart.id).await?;
        let existing_item = existing_items
            .iter()
            .find(|item| item.product_id == product_id);

        match existing_item {
            Some(existing_item) => {
                // PRODUTO EXISTE: Remover do carrinho
                println!("🗑️ Removendo produto do carrinho: {}", product_id);

                repo_cart.delete_cart_item(existing_item.id).await?;

                println!("✅ Produto removido com sucesso");
            }
            None => {
                // PRODUTO NÃO EXISTE: Retornar erro
                return Err(AppError::not_found("Produto não encontrado no carrinho"));
            }
        }

        // Recalcular totais do carrinho
        let updated_items = repo_cart.list_cart_items(cart.id).await?;
        let new_subtotal: BigDecimal = updated_items
            .iter()
            .map(|item| {
                let item_price = BigDecimal::from(item.unit_price) / BigDecimal::from(100);
                item_price * BigDecimal::from(item.quantity)
            })
            .sum();

        // Atualizar dados do carrinho no banco
        repo_cart
            .update_cart_data(
                cart.id,
                &new_subtotal,
                &cart.discount_total,
                &cart.tax_total,
                &cart.shipping_total,
            )
            .await?;

        // Buscar carrinho atualizado e retornar com produtos populados
        let updated_cart = repo_cart.find_by_tenant_id(tenant_id).await?;
        let cart = match updated_cart {
            Some(cart) => cart,
            None => return Err(AppError::not_found("Carrinho não encontrado")),
        };

        let list_cart_items = repo_cart.list_cart_items(cart.id).await?;

        // Buscar os produtos dos itens restantes
        let product_ids: Vec<Uuid> = list_cart_items.iter().map(|item| item.product_id).collect();
        let products = if !product_ids.is_empty() {
            let repo_product = ProductRepository::new(app_state);
            repo_product
                .find_by_ids(&product_ids)
                .await
                .map_err(|e| AppError::database_error(e.to_string()))?
        } else {
            vec![]
        };

        let cart_with_items =
            CartWithItems::from_cart_and_items_with_products(cart, list_cart_items, products);

        Ok(cart_with_items)
    }

    async fn get_or_create_cart(
        app_state: &AppState,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<Cart, AppError> {
        let repository = CartRepository::new(app_state);

        let cart = repository
            .find_by_tenant_id(tenant_id)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        match cart {
            Some(cart) => Ok(cart),
            None => {
                // Criar novo carrinho
                repository
                    .create(tenant_id, user_id)
                    .await
                    .map_err(|e| AppError::database_error(e.to_string()))
            }
        }
    }
}
