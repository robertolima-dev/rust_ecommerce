use crate::app_core::app_state::AppState;
use crate::app_core::app_error::AppError;
use crate::utils::pagination::PaginatedResponse;
use crate::apps::cart::models::{Cart, CreateCartRequest, UpdateCartRequest};
use crate::apps::cart::repositories::CartRepository;
use uuid::Uuid;

pub struct CartService;

impl CartService {
    pub async fn list_carts(app_state: &AppState) -> Result<PaginatedResponse<Cart>, AppError> {
        let repository = CartRepository::new(app_state);
        let carts = repository.find_all().await
            .map_err(|e| AppError::database_error(e.to_string()))?;
        
        Ok(PaginatedResponse {
            count: carts.len() as i64,
            results: carts,
            limit: 10,
            offset: 0,
        })
    }

    pub async fn get_cart(app_state: &AppState, id: Uuid) -> Result<Option<Cart>, AppError> {
        let repository = CartRepository::new(app_state);
        repository.find_by_id(id).await
            .map_err(|e| AppError::database_error(e.to_string()))
    }

    pub async fn create_cart(app_state: &AppState, request: CreateCartRequest) -> Result<Cart, AppError> {
        let repository = CartRepository::new(app_state);
        repository.create(request).await
            .map_err(|e| AppError::database_error(e.to_string()))
    }

    pub async fn update_cart(app_state: &AppState, id: Uuid, request: UpdateCartRequest) -> Result<Option<Cart>, AppError> {
        let repository = CartRepository::new(app_state);
        repository.update(id, request).await
            .map_err(|e| AppError::database_error(e.to_string()))
    }

    pub async fn delete_cart(app_state: &AppState, id: Uuid) -> Result<bool, AppError> {
        let repository = CartRepository::new(app_state);
        repository.delete(id).await
            .map_err(|e| AppError::database_error(e.to_string()))
    }
}
