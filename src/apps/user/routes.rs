use crate::app_core::app_error::AppError;
use crate::app_core::app_extensions::RequestUserExt;
use crate::app_core::app_state::AppState;
use crate::apps::user::models::{
    ChangePasswordRequest, ForgotPasswordRequest, LoginRequest, UpdateProfileRequest,
    UpdateUserRequest, UserRequest,
};
use crate::apps::user::services::UserService;
use crate::utils::pagination::PaginationParams;
use actix_web::{HttpRequest, HttpResponse, Responder, web};

// ===== PUBLIC ROUTES (sem autenticação) =====

/// Cadastrar novo usuário
pub async fn create_user(
    app_state: web::Data<AppState>,
    payload: web::Json<UserRequest>,
) -> Result<impl Responder, AppError> {
    let response = UserService::create_user_with_profile(payload.into_inner(), &app_state).await?;

    Ok(HttpResponse::Created().json(response))
}

/// Login do usuário
pub async fn login(
    app_state: web::Data<AppState>,
    payload: web::Json<LoginRequest>,
) -> Result<impl Responder, AppError> {
    let response = UserService::login_user(payload.into_inner(), &app_state).await?;

    Ok(HttpResponse::Ok().json(response))
}

/// Esqueci minha senha
pub async fn forgot_password(
    app_state: web::Data<AppState>,
    payload: web::Json<ForgotPasswordRequest>,
) -> Result<impl Responder, AppError> {
    UserService::forgot_password(payload.into_inner(), &app_state).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Email de recuperação enviado com sucesso"
    })))
}

/// Alterar senha com código
pub async fn change_password(
    app_state: web::Data<AppState>,
    payload: web::Json<ChangePasswordRequest>,
) -> Result<impl Responder, AppError> {
    UserService::change_password(payload.into_inner(), &app_state).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Senha alterada com sucesso"
    })))
}

/// Confirmar email (rota pública)
pub async fn confirm_email(
    app_state: web::Data<AppState>,
    code: web::Path<String>,
) -> Result<impl Responder, AppError> {
    UserService::confirm_email(code.into_inner(), &app_state).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Email confirmado com sucesso"
    })))
}

// ===== PRIVATE ROUTES (com autenticação) =====

/// Buscar dados do usuário logado
pub async fn get_me(
    req: HttpRequest,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    // Extrair user_id do token JWT
    let user_id = req.user_id()?;

    // Extrair token do header Authorization
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::unauthorized("Token não fornecido"))?;

    // Remover "Token " do início do token se existir
    let token = if auth_header.starts_with("Token ") {
        auth_header.strip_prefix("Token ").unwrap().to_string()
    } else {
        auth_header.to_string()
    };

    let response = UserService::get_me(user_id, token, &app_state).await?;

    Ok(HttpResponse::Ok().json(response))
}

/// Listar usuários paginados
pub async fn list_users(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    query: web::Query<PaginationParams>,
) -> Result<impl Responder, AppError> {
    let access_level = req.access_level()?;

    if !["admin", "super_admin"].contains(&access_level.as_str()) {
        return Err(AppError::forbidden(
            "Acesso negado. Apenas admins podem listar users.".to_string(),
        ));
    }

    let PaginationParams { limit, offset } = query.into_inner();

    let response = UserService::list_users_paginated(limit, offset, &app_state).await?;

    Ok(HttpResponse::Ok().json(response))
}

/// Atualizar usuário logado
pub async fn update_user(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    payload: web::Json<UpdateUserRequest>,
) -> Result<impl Responder, AppError> {
    // Extrair user_id do token JWT
    let user_id = req.user_id()?;

    // Extrair token do header Authorization
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::unauthorized("Token não fornecido"))?;

    // Remover "Token " do início do token se existir
    let token = if auth_header.starts_with("Token ") {
        auth_header.strip_prefix("Token ").unwrap().to_string()
    } else {
        auth_header.to_string()
    };

    let response =
        UserService::update_user(user_id, payload.into_inner(), token, &app_state).await?;

    Ok(HttpResponse::Ok().json(response))
}

/// Atualizar perfil do usuário logado
pub async fn update_profile(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    payload: web::Json<UpdateProfileRequest>,
) -> Result<impl Responder, AppError> {
    // Extrair user_id do token JWT
    let user_id = req.user_id()?;

    // Extrair token do header Authorization
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::unauthorized("Token não fornecido"))?;

    // Remover "Token " do início do token se existir
    let token = if auth_header.starts_with("Token ") {
        auth_header.strip_prefix("Token ").unwrap().to_string()
    } else {
        auth_header.to_string()
    };

    let response =
        UserService::update_profile(user_id, payload.into_inner(), token, &app_state).await?;

    Ok(HttpResponse::Ok().json(response))
}

/// Deletar usuário logado (soft delete)
pub async fn delete_user(
    req: HttpRequest,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    // Extrair user_id do token JWT
    let user_id = req.user_id()?;

    UserService::delete_user(user_id, &app_state).await?;

    Ok(HttpResponse::NoContent().finish())
}
