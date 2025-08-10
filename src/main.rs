mod app_core;
mod apps;
mod utils;

use actix_web::{App, HttpServer, web};
use tracing::info;
use tracing_actix_web::TracingLogger;

use crate::app_core::app_routes::api_v1_scope;
use crate::app_core::databases::postgres::get_db_pool;
use crate::app_core::{app_state::AppState, init_settings};
use dotenvy::dotenv;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 Iniciando Actix Web Server...");
    dotenv().ok();

    // Configurar logging
    init_settings::setup_development_logging()?;

    // Inicializar configurações primeiro
    init_settings::init_settings()?;
    let settings = init_settings::get_settings();

    // Inicializar conexões com bancos de dados
    let pool = get_db_pool().await;

    info!(
        host = %settings.server.host,
        port = %settings.server.port,
        environment = ?settings.environment,
        "🚀 Servidor Actix iniciado"
    );

    let app_state = web::Data::new(AppState { db: pool });

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(app_state.clone())
            .service(api_v1_scope())
    })
    .bind((settings.server.host.clone(), settings.server.port))?
    .run()
    .await?;

    Ok(())
}
