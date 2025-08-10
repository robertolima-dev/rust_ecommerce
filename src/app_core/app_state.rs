use sqlx::PgPool;

#[allow(dead_code)]
pub struct AppState {
    pub db: PgPool,
}
