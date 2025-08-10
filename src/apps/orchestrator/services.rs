use crate::app_core::app_error::AppError;
use crate::app_core::app_state::AppState;
use crate::apps::orchestrator::models::{
    AppAuthorizationResponse, CreateOrchestratorRequest, OrchestratorResponse, SyncAllUsersRequest,
    UpdateOrchestratorRequest,
};
use crate::apps::orchestrator::repositories::OrchestratorRepository;
use crate::apps::sync_app::producer::SyncProducer;
use crate::apps::tenant::repositories::TenantRepository;
use crate::apps::user::models::UserWithProfile;
use crate::apps::user::repositories::{ProfileRepository, UserRepository};
use crate::utils::pagination::PaginatedResponse;
use tracing::{error, info};
use uuid::Uuid;

pub struct OrchestratorService;

#[allow(dead_code)]
impl OrchestratorService {
    pub async fn list_orchestrators(
        app_state: &AppState,
    ) -> Result<PaginatedResponse<OrchestratorResponse>, AppError> {
        let repository = OrchestratorRepository::new(app_state);
        let orchestrators = repository
            .find_all()
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        let responses: Vec<OrchestratorResponse> =
            orchestrators.into_iter().map(|o| o.into()).collect();

        Ok(PaginatedResponse {
            count: responses.len() as i64,
            results: responses,
            limit: 10,
            offset: 0,
        })
    }

    pub async fn get_orchestrator(
        app_state: &AppState,
        id: Uuid,
    ) -> Result<Option<OrchestratorResponse>, AppError> {
        let repository = OrchestratorRepository::new(app_state);
        let orchestrator = repository
            .find_by_id(id)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        Ok(orchestrator.map(|o| o.into()))
    }

    pub async fn create_orchestrator(
        app_state: &AppState,
        request: CreateOrchestratorRequest,
    ) -> Result<OrchestratorResponse, AppError> {
        let repository = OrchestratorRepository::new(app_state);
        let orchestrator = repository
            .create(request)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        Ok(orchestrator.into())
    }

    pub async fn update_orchestrator(
        app_state: &AppState,
        id: Uuid,
        request: UpdateOrchestratorRequest,
    ) -> Result<Option<OrchestratorResponse>, AppError> {
        let repository = OrchestratorRepository::new(app_state);
        let orchestrator = repository
            .update(id, request)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        Ok(orchestrator.map(|o| o.into()))
    }

    pub async fn delete_orchestrator(app_state: &AppState, id: Uuid) -> Result<bool, AppError> {
        let repository = OrchestratorRepository::new(app_state);
        repository
            .delete(id)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))
    }

    pub async fn authorize_app(
        app_state: &AppState,
        app_token: Uuid,
    ) -> Result<AppAuthorizationResponse, AppError> {
        let repository = OrchestratorRepository::new(app_state);
        let orchestrator = repository
            .find_by_app_token(app_token)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        match orchestrator {
            Some(app) => Ok(AppAuthorizationResponse {
                app_name: app.app_name,
                status: "authorized".to_string(),
            }),
            None => Ok(AppAuthorizationResponse {
                app_name: "unknown".to_string(),
                status: "unauthorized".to_string(),
            }),
        }
    }

    pub async fn sync_all_users_with_app(
        app_state: &AppState,
        request: SyncAllUsersRequest,
    ) -> Result<(), AppError> {
        info!(
            "Iniciando sync de todos os usuários com o app: {}",
            request.app_name
        );

        // Buscar todos os usuários
        let user_repo = UserRepository::new(app_state);
        let profile_repo = ProfileRepository::new(app_state);

        let users = user_repo
            .find_all_paginated(1000, 0) // Buscar até 1000 usuários
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        info!("Encontrados {} usuários para sincronizar", users.len());

        let mut success_count = 0;
        let mut error_count = 0;

        // Sincronizar cada usuário com o app específico
        for user in users {
            let profile = match profile_repo.find_by_user_id(user.id).await {
                Ok(Some(profile)) => profile,
                Ok(None) => {
                    error!("Perfil não encontrado para o usuário: {}", user.email);
                    error_count += 1;
                    continue;
                }
                Err(e) => {
                    error!("Erro ao buscar perfil do usuário {}: {}", user.email, e);
                    error_count += 1;
                    continue;
                }
            };

            let repository_tenant = TenantRepository::new(app_state);
            let tenant = repository_tenant
                .find_by_user_id(user.id)
                .await
                .map_err(|e| AppError::database_error(e.to_string()))?;

            let user_with_profile =
                UserWithProfile::from_user_and_profile_ref(&user, &profile, &tenant);

            match SyncProducer::sync_user(user_with_profile, app_state, Some(&request.app_name))
                .await
            {
                Ok(_) => {
                    success_count += 1;
                    info!("Usuário {} sincronizado com sucesso", user.email);
                }
                Err(e) => {
                    error_count += 1;
                    error!("Erro ao sincronizar usuário {}: {}", user.email, e);
                }
            }
        }

        info!(
            "Sync concluído. Sucessos: {}, Erros: {}",
            success_count, error_count
        );

        if error_count > 0 {
            return Err(AppError::internal(format!(
                "Sync concluído com {} erros de {} usuários",
                error_count,
                success_count + error_count
            )));
        }

        Ok(())
    }
}
