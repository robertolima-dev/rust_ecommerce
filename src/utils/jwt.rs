use crate::app_core::app_model::Claims;
use crate::app_core::init_settings::get_settings;
use chrono::{Duration, Utc};
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode,
};
use uuid::Uuid;

pub fn generate_jwt(
    user_id: &str,
    access_level: &str,
    tenant_id: Uuid,
) -> Result<String, jsonwebtoken::errors::Error> {
    let settings = get_settings();
    let expires_in = settings.jwt.expires_in;

    let exp = Utc::now()
        .checked_add_signed(Duration::seconds(expires_in as i64))
        .expect("Erro ao calcular expiração do token")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp,
        access_level: access_level.to_string(),
        tenant_id,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(settings.jwt.secret.as_bytes()),
    )
}

#[allow(dead_code)]
pub fn verify_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let settings = get_settings();

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(settings.jwt.secret.as_ref()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

#[allow(dead_code)]
pub fn decode_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let settings = get_settings();
    let token_data: TokenData<Claims> = decode::<Claims>(
        token,
        &DecodingKey::from_secret(settings.jwt.secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )?;

    Ok(token_data.claims)
}

pub fn calculate_remaining_expiration(token: &str) -> Result<i64, String> {
    let settings = get_settings();

    let token_data: TokenData<Claims> = decode::<Claims>(
        token,
        &DecodingKey::from_secret(settings.jwt.secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|e| format!("Erro ao decodificar token: {}", e))?;

    let exp_timestamp = token_data.claims.exp as i64;
    let now_timestamp = Utc::now().timestamp();

    Ok(exp_timestamp.saturating_sub(now_timestamp))
}
