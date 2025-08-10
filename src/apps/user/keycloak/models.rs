use serde::{Deserialize, Serialize};
use validator::Validate;
use crate::apps::user::models::UserWithProfile;

#[derive(Debug, Deserialize, Validate)]
pub struct KeycloakLoginRequest {
    #[validate(length(min = 1, message = "Provider token é obrigatório"))]
    pub provider_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeycloakTokenIntrospectResponse {
    pub active: bool,
    pub exp: Option<i64>,
    pub iat: Option<i64>,
    pub jti: Option<String>,
    pub iss: Option<String>,
    pub aud: Option<String>,
    pub sub: Option<String>,
    pub typ: Option<String>,
    pub azp: Option<String>,
    pub scope: Option<String>,
    pub sid: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub preferred_username: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub email: Option<String>,
    pub realm_access: Option<RealmAccess>,
    pub resource_access: Option<ResourceAccess>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RealmAccess {
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResourceAccess {
    #[serde(rename = "rust-template-client")]
    pub rust_template_client: Option<ClientAccess>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientAccess {
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeycloakUserInfo {
    pub sub: String,
    pub email_verified: bool,
    pub name: String,
    pub preferred_username: String,
    pub given_name: String,
    pub family_name: String,
    pub email: String,
    pub realm_access: Option<RealmAccess>,
    pub resource_access: Option<ResourceAccess>,
}

#[derive(Debug, Serialize)]
pub struct KeycloakLoginResponse {
    pub message: String,
    pub data: KeycloakAuthData,
}

#[derive(Debug, Serialize)]
pub struct KeycloakAuthData {
    pub user: UserWithProfile,
    pub token: String,
    pub expires_in: String,
} 