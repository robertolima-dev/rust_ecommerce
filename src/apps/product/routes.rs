use crate::app_core::app_extensions::RequestUserExt;
use crate::app_core::app_state::AppState;
use crate::apps::product::models::{CreateProductRequest, ProductListParams, UpdateProductRequest};
use crate::{app_core::app_error::AppError, apps::product::services::ProductService};
use actix_web::web::Json;
use actix_web::{HttpRequest, HttpResponse, Responder, web};

use uuid::Uuid;

pub async fn list_products(
    app_state: web::Data<AppState>,
    query: web::Query<ProductListParams>,
) -> Result<impl Responder, AppError> {
    let params = query.into_inner();
    let result = ProductService::list_products(&app_state, params).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!(result)))
}

pub async fn get_product(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<impl Responder, AppError> {
    let id = path.into_inner();
    let result = ProductService::get_product(&app_state, id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!(result)))
}

pub async fn create_product(
    app_state: web::Data<AppState>,
    payload: Json<CreateProductRequest>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let tenant_id = req.tenant_id()?;
    let dto = payload.into_inner();
    let result = ProductService::create_product(&app_state, dto, tenant_id).await?;

    Ok(HttpResponse::Created().json(serde_json::json!(result)))
}

pub async fn update_product(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    payload: Json<UpdateProductRequest>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let id = path.into_inner();
    let tenant_id = req.tenant_id()?;
    let dto = payload.into_inner();
    let result = ProductService::update_product(&app_state, id, tenant_id, dto).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!(result)))
}

pub async fn delete_product(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let id = path.into_inner();
    let tenant_id = req.tenant_id()?;
    ProductService::delete_product(&app_state, id, tenant_id)
        .await
        .map_err(AppError::from)?;

    Ok(HttpResponse::NoContent().finish())
}
