use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeycloakConfig {
    pub base_url: String,
    pub realm: String,
    pub client_id: String,
    pub client_secret: String,
    pub admin_username: String,
    pub admin_password: String,
}

impl KeycloakConfig {
    pub fn new() -> Self {
        Self {
            base_url: std::env::var("KEYCLOAK_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            realm: std::env::var("KEYCLOAK_REALM").unwrap_or_else(|_| "rust-template".to_string()),
            client_id: std::env::var("KEYCLOAK_CLIENT_ID")
                .unwrap_or_else(|_| "rust-template-client".to_string()),
            client_secret: std::env::var("KEYCLOAK_CLIENT_SECRET")
                .unwrap_or_else(|_| "".to_string()),
            admin_username: std::env::var("KEYCLOAK_ADMIN_USERNAME")
                .unwrap_or_else(|_| "admin".to_string()),
            admin_password: std::env::var("KEYCLOAK_ADMIN_PASSWORD")
                .unwrap_or_else(|_| "admin".to_string()),
        }
    }

    pub fn auth_url(&self) -> String {
        format!(
            "{}/realms/{}/protocol/openid-connect",
            self.base_url, self.realm
        )
    }

    pub fn _admin_url(&self) -> String {
        format!("{}/admin/realms/{}", self.base_url, self.realm)
    }

    pub fn token_introspect_url(&self) -> String {
        format!("{}/token/introspect", self.auth_url())
    }

    pub fn _userinfo_url(&self) -> String {
        format!("{}/userinfo", self.auth_url())
    }
}
