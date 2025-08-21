use crate::app_core::app_error::AppError;
use crate::app_core::app_extensions::RequestUserExt;
use crate::app_core::app_state::AppState;
use crate::apps::cart::services::CartService;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use uuid::Uuid;

pub async fn get_cards(
    app_state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let tenant_id = req.tenant_id()?;

    let result = CartService::list_cards(&app_state, tenant_id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!(result)))
}

pub async fn get_card_by_tenant(
    app_state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let tenant_id = req.tenant_id()?;

    let result = CartService::get_cart(&app_state, tenant_id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!(result)))
}

pub async fn create_cart(
    app_state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let user_id = req.user_id()?;
    let tenant_id = req.tenant_id()?;

    let result = CartService::create_cart(&app_state, user_id, tenant_id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!(result)))
}

pub async fn delete_cart(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let tenant_id = req.tenant_id()?;
    let id = path.into_inner();

    CartService::delete_cart(&app_state, id, tenant_id).await?;

    Ok(HttpResponse::NoContent().finish())
}
