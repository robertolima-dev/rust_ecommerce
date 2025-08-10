use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // ID do usuário ou email
    pub exp: usize,  // timestamp de expiração
    pub access_level: String,
    pub tenant_id: Uuid,
}
