use crate::app_core::settings::Settings;
use once_cell::sync::OnceCell;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static SETTINGS: OnceCell<Settings> = OnceCell::new();

pub fn setup_development_logging() -> Result<(), String> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rust_template=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    Ok(())
}

pub fn init_settings() -> Result<(), String> {
    if SETTINGS.get().is_some() {
        info!("⚠️ Settings já inicializado. Ignorando segunda chamada.");
        return Ok(());
    }

    let settings = Settings::load()?;
    SETTINGS
        .set(settings)
        .map_err(|_| "Configurações já inicializadas".to_string())
}

pub fn get_settings() -> &'static Settings {
    SETTINGS
        .get()
        .expect("Settings ainda não inicializado. Rode init_settings() antes.")
}
