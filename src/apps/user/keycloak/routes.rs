use crate::app_core::app_error::AppError;
use crate::app_core::app_state::AppState;
use crate::apps::user::keycloak::models::KeycloakLoginRequest;
use crate::apps::user::keycloak::services::KeycloakService;
use actix_web::{HttpResponse, Responder, web};

/// Endpoint para login via Keycloak
pub async fn login_keycloak(
    app_state: web::Data<AppState>,
    payload: web::Json<KeycloakLoginRequest>,
) -> Result<impl Responder, AppError> {
    let auth_data =
        KeycloakService::login_with_provider_token(payload.into_inner(), &app_state).await?;
    Ok(HttpResponse::Ok().json(auth_data))
}
