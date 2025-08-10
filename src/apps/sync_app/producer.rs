use crate::app_core::app_state::AppState;
use crate::apps::orchestrator::repositories::OrchestratorRepository;
use crate::apps::user::models::UserWithProfile;
use reqwest::Client;
use serde_json;
use tracing::{error, info, warn};

pub struct SyncProducer;

impl SyncProducer {
    /// Função para sincronizar um usuário com SQS
    pub async fn sync_user(
        user_with_profile: UserWithProfile,
        app_state: &AppState,
        app_name: Option<&str>,
    ) -> Result<(), String> {
        info!("Iniciando sync do usuário: {}", user_with_profile.email);

        // Criar JSON no formato especificado
        let json_data = serde_json::json!({
            "user": {
                "id": user_with_profile.id,
                "username": user_with_profile.username,
                "email": user_with_profile.email,
                "first_name": user_with_profile.first_name,
                "last_name": user_with_profile.last_name,
                "profile": {
                    "id": user_with_profile.profile.id,
                    "user_id": user_with_profile.profile.user_id,
                    "bio": user_with_profile.profile.bio,
                    "birth_date": user_with_profile.profile.birth_date,
                    "phone": user_with_profile.profile.phone,
                    "document": user_with_profile.profile.document,
                    "profession": user_with_profile.profile.profession,
                    "avatar": user_with_profile.profile.avatar,
                    "confirm_email": user_with_profile.profile.confirm_email,
                    "unsubscribe": user_with_profile.profile.unsubscribe,
                    "access_level": user_with_profile.profile.access_level,
                    "dt_updated": user_with_profile.profile.dt_updated,
                    "dt_created": user_with_profile.profile.dt_created
                }
            }
        });

        let msg_data = serde_json::json!({
            "obj_type": "sync_user",
            "obj_data": json_data,
            "obj_cmd": "put"
        });
        println!("=== SYNC USER JSON ===");
        println!("{}", serde_json::to_string_pretty(&msg_data).unwrap());
        println!("=====================\n");

        // Listar todos os orchestrators disponíveis e dar um post para cada um deles
        let orchestrator_repo = OrchestratorRepository::new(app_state);
        let orchestrators = orchestrator_repo
            .find_all()
            .await
            .map_err(|e| format!("Erro ao buscar orchestrators: {}", e))?;

        if orchestrators.is_empty() {
            warn!("Nenhum orchestrator encontrado para sincronização");
            return Ok(());
        }

        info!(
            "Encontrados {} orchestrators para sincronização",
            orchestrators.len()
        );

        // Criar cliente HTTP
        let client = Client::new();

        // Enviar dados para cada orchestrator
        for orchestrator in orchestrators {
            if app_name.is_some() && orchestrator.app_name != app_name.unwrap() {
                continue;
            }

            if orchestrator.app_name == "app_auth" {
                info!(
                    "Orchestrator {} não é suportado para sync de usuários",
                    orchestrator.app_name
                );
                continue;
            }

            let url = format!(
                "{}/v1/event-sync/",
                orchestrator.app_url.trim_end_matches('/')
            );

            info!(
                "Enviando dados para orchestrator: {} ({})",
                orchestrator.app_name, url
            );

            let response = client
                .post(&url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Token {}", orchestrator.app_token))
                .json(&msg_data)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        info!("Dados enviados com sucesso para {}", orchestrator.app_name);
                    } else {
                        warn!(
                            "Erro ao enviar dados para {}: Status {}",
                            orchestrator.app_name,
                            resp.status()
                        );
                    }
                }
                Err(e) => {
                    error!(
                        "Erro de conexão ao enviar dados para {}: {}",
                        orchestrator.app_name, e
                    );
                }
            }
        }

        // TODO: Implementar integração com SQS
        // Exemplo de como seria a implementação:
        // let message = serde_json::to_string(&json_data)
        //     .map_err(|e| format!("Erro ao serializar dados: {}", e))?;
        //
        // sqs_client.send_message()
        //     .queue_url("https://sqs.region.amazonaws.com/account/queue-name")
        //     .message_body(message)
        //     .send()
        //     .await
        //     .map_err(|e| format!("Erro ao enviar para SQS: {}", e))?;

        info!(
            "Sync do usuário {} concluído com sucesso",
            user_with_profile.email
        );

        Ok(())
    }

    /// Função para sincronizar múltiplos usuários
    #[allow(dead_code)]
    pub async fn sync_users(
        users: Vec<UserWithProfile>,
        app_state: &AppState,
        app_name: Option<&str>,
    ) -> Result<(), String> {
        info!("Iniciando sync de {} usuários", users.len());

        let total_users = users.len();
        for user in users {
            match Self::sync_user(user, app_state, app_name).await {
                Ok(_) => info!("Usuário sincronizado com sucesso"),
                Err(e) => {
                    error!("Erro ao sincronizar usuário: {}", e);
                    // Dependendo da estratégia, você pode querer continuar ou parar aqui
                    // return Err(e);
                }
            }
        }

        info!("Sync de {} usuários concluído", total_users);
        Ok(())
    }
}
