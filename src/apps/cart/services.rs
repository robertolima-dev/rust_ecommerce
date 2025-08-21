use crate::app_core::app_error::AppError;
use crate::app_core::app_state::AppState;
use crate::apps::cart::models::Cart;
use crate::apps::cart::repositories::CartRepository;
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

    pub async fn get_cart(app_state: &AppState, tenant_id: Uuid) -> Result<Cart, AppError> {
        let repository = CartRepository::new(app_state);
        let cart = repository
            .find_by_tenant_id(tenant_id)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        match cart {
            Some(cart) => Ok(cart),
            None => Err(AppError::not_found("Carrinho não encontrado")),
        }
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
}
