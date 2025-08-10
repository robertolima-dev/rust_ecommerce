use crate::app_core::app_error::AppError;
use crate::app_core::app_state::AppState;
use crate::apps::product::models::{
    CreateProductRequest, Product, ProductListParams, UpdateProductRequest,
};
use crate::apps::product::repositories::ProductRepository;
use crate::utils::pagination::PaginatedResponse;
use uuid::Uuid;

pub struct ProductService;

impl ProductService {
    pub async fn list_products(
        app_state: &AppState,
        params: ProductListParams,
    ) -> Result<PaginatedResponse<Product>, AppError> {
        let repository = ProductRepository::new(app_state);
        let products_paginated = repository
            .find_all(params)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        Ok(products_paginated)
    }

    pub async fn get_product(app_state: &AppState, id: Uuid) -> Result<Option<Product>, AppError> {
        let repository = ProductRepository::new(app_state);
        repository
            .find_by_id(id)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))
    }

    pub async fn create_product(
        app_state: &AppState,
        request: CreateProductRequest,
        tenant_id: Uuid,
    ) -> Result<Product, AppError> {
        let repository = ProductRepository::new(app_state);
        repository
            .create(request, tenant_id)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))
    }

    pub async fn update_product(
        app_state: &AppState,
        id: Uuid,
        tenant_id: Uuid,
        request: UpdateProductRequest,
    ) -> Result<Option<Product>, AppError> {
        let repository = ProductRepository::new(app_state);
        repository
            .update(id, tenant_id, request)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))
    }

    pub async fn delete_product(
        app_state: &AppState,
        id: Uuid,
        tenant_id: Uuid,
    ) -> Result<bool, AppError> {
        let repository = ProductRepository::new(app_state);
        repository
            .delete(id, tenant_id)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))
    }
}
