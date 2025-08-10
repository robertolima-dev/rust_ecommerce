use crate::apps::sync_app::producer::SyncProducer;
use crate::apps::user::models::UserWithProfile;
use tracing::info;

#[allow(dead_code)]
pub struct SyncConsumer;

#[allow(dead_code)]
impl SyncConsumer {
    /// Exemplo de como usar a função sync_user
    ///
    /// Esta função demonstra como chamar o sync para um usuário
    pub async fn process_user_sync(user_with_profile: UserWithProfile, app_state: &crate::app_core::app_state::AppState) -> Result<(), String> {
        info!("Processando sync do usuário: {}", user_with_profile.email);

        // Chama a função de sync
        SyncProducer::sync_user(user_with_profile, app_state, None).await?;

        info!("Processamento concluído com sucesso");
        Ok(())
    }

    /// Exemplo de como processar múltiplos usuários
    pub async fn process_batch_sync(users: Vec<UserWithProfile>, app_state: &crate::app_core::app_state::AppState) -> Result<(), String> {
        info!("Processando sync em lote de {} usuários", users.len());

        // Chama a função de sync em lote
        SyncProducer::sync_users(users, app_state, None).await?;

        info!("Processamento em lote concluído");
        Ok(())
    }
}
