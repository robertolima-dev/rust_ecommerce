use crate::app_core::app_error::AppError;
use crate::app_core::app_extensions::RequestUserExt;
use crate::app_core::app_state::AppState;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use uuid::Uuid;

pub async fn list_carts(
    app_state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let user_id = req.user_id()?;
    let tenant_id = req.tenant_id()?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Lista de Cart",
        "data": []
    })))
}

pub async fn get_cart(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<impl Responder, AppError> {
    let user_id = req.user_id()?;
    let tenant_id = req.tenant_id()?;
    let id = path.into_inner();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Detalhes do Cart",
        "id": id,
        "data": {}
    })))
}

pub async fn create_cart(
    app_state: web::Data<AppState>,
    payload: web::Json<serde_json::Value>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let user_id = req.user_id()?;
    let tenant_id = req.tenant_id()?;
    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": "Cart criado com sucesso",
        "data": payload.into_inner()
    })))
}

pub async fn update_cart(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    payload: web::Json<serde_json::Value>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let user_id = req.user_id()?;
    let tenant_id = req.tenant_id()?;
    let id = path.into_inner();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Cart atualizado com sucesso",
        "id": id,
        "data": payload.into_inner()
    })))
}

pub async fn delete_cart(
    app_state: web::Data<AppState>,
    path: web::Path<Uuid>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    let user_id = req.user_id()?;
    let tenant_id = req.tenant_id()?;
    let id = path.into_inner();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Cart deletado com sucesso",
        "id": id
    })))
}
