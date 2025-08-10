use crate::app_core::app_error::AppError;
use crate::app_core::app_extensions::RequestUserExt;
use crate::app_core::app_state::AppState;
use crate::apps::orchestrator::models::{CreateOrchestratorRequest, SyncAllUsersRequest};
use crate::apps::orchestrator::services::OrchestratorService;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use uuid::Uuid;
use validator::Validate;

pub async fn list_orchestrators(
    app_state: web::Data<AppState>,
    req: HttpRequest,
) -> Result<impl Responder, AppError> {
    // Verificar se o usuário está autenticado e tem access_level = super_admin
    let access_level = req.access_level()?;

    if access_level != "super_admin" {
        return Err(AppError::forbidden(
            "Acesso negado. Apenas super_admin pode listar os apps.".to_string(),
        ));
    }

    let result = OrchestratorService::list_orchestrators(&app_state).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Lista de apps cadastrados",
        "data": result
    })))
}

pub async fn get_orchestrator(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    // Verificar se o usuário está autenticado e tem access_level = super_admin
    let access_level = req.access_level()?;

    if access_level != "super_admin" {
        return Err(AppError::forbidden(
            "Acesso negado. Apenas super_admin pode visualizar detalhes dos apps.".to_string(),
        ));
    }

    let id_str = path.into_inner();
    let id =
        Uuid::parse_str(&id_str).map_err(|_| AppError::bad_request("ID inválido".to_string()))?;

    let orchestrator = OrchestratorService::get_orchestrator(&app_state, id).await?;

    match orchestrator {
        Some(app) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Detalhes do app",
            "data": app
        }))),
        None => Err(AppError::not_found("App não encontrado".to_string())),
    }
}

pub async fn create_orchestrator(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<CreateOrchestratorRequest>,
) -> Result<impl Responder, AppError> {
    // Verificar se o usuário está autenticado e tem access_level = super_admin
    let access_level = req.access_level()?;

    if access_level != "super_admin" {
        return Err(AppError::forbidden(
            "Acesso negado. Apenas super_admin pode cadastrar novos apps.".to_string(),
        ));
    }

    // Validar o payload
    payload
        .validate()
        .map_err(|e| AppError::bad_request(format!("Dados inválidos: {}", e)))?;

    let orchestrator =
        OrchestratorService::create_orchestrator(&app_state, payload.into_inner()).await?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "message": "App cadastrado com sucesso",
        "data": orchestrator
    })))
}



// Endpoint para autorizar app pelo app_token
pub async fn authorize_app(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let app_token_str = path.into_inner();

    // Converter string para UUID
    let app_token = Uuid::parse_str(&app_token_str)
        .map_err(|_| AppError::bad_request("Token inválido".to_string()))?;

    let auth_response = OrchestratorService::authorize_app(&app_state, app_token).await?;

    Ok(HttpResponse::Ok().json(auth_response))
}

// Endpoint para sincronizar todos os usuários com um app específico
pub async fn sync_all_users_with_app(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    payload: web::Json<SyncAllUsersRequest>,
) -> Result<impl Responder, AppError> {
    // Verificar se o usuário está autenticado e tem access_level = super_admin
    let access_level = req.access_level()?;

    if access_level != "super_admin" {
        return Err(AppError::forbidden(
            "Acesso negado. Apenas super_admin pode sincronizar usuários.".to_string(),
        ));
    }

    // Validar o payload
    payload
        .validate()
        .map_err(|e| AppError::bad_request(format!("Dados inválidos: {}", e)))?;

    OrchestratorService::sync_all_users_with_app(&app_state, payload.into_inner()).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Sincronização de usuários iniciada com sucesso"
    })))
}

// Endpoint para deletar um orchestrator
pub async fn delete_orchestrator(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    // Verificar se o usuário está autenticado e tem access_level = super_admin
    let access_level = req.access_level()?;

    if access_level != "super_admin" {
        return Err(AppError::forbidden(
            "Acesso negado. Apenas super_admin pode deletar apps.".to_string(),
        ));
    }

    let id_str = path.into_inner();
    let id = Uuid::parse_str(&id_str)
        .map_err(|_| AppError::bad_request("ID inválido".to_string()))?;

    let deleted = OrchestratorService::delete_orchestrator(&app_state, id).await?;

    if deleted {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "App deletado com sucesso"
        })))
    } else {
        Err(AppError::not_found("App não encontrado".to_string()))
    }
}
