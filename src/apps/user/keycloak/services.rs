use crate::app_core::app_error::AppError;
use crate::app_core::app_state::AppState;
use crate::apps::sync_app::producer::SyncProducer;
use crate::apps::tenant::repositories::TenantRepository;
use crate::apps::user::keycloak::{
    config::KeycloakConfig,
    models::{
        KeycloakAuthData, KeycloakLoginRequest, KeycloakTokenIntrospectResponse, KeycloakUserInfo,
    },
};
use crate::apps::user::models::{Profile, User, UserWithProfile};
use crate::apps::user::repositories::{ProfileRepository, UserRepository};
use crate::utils::jwt::{calculate_remaining_expiration, generate_jwt};
use reqwest::Client;
use tracing::{error, info};
use uuid::Uuid;
use validator::Validate;

pub struct KeycloakService;

impl KeycloakService {
    /// Login via Keycloak usando provider token
    pub async fn login_with_provider_token(
        request: KeycloakLoginRequest,
        app_state: &AppState,
    ) -> Result<KeycloakAuthData, AppError> {
        info!("Iniciando login via Keycloak");

        // Validar request
        request
            .validate()
            .map_err(|e| AppError::bad_request(format!("Dados inválidos: {}", e)))?;

        let config = KeycloakConfig::new();
        let client = Client::new();

        // 1. Validar token com Keycloak
        let token_info =
            Self::validate_provider_token(&client, &config, &request.provider_token).await?;

        if !token_info.active {
            return Err(AppError::unauthorized("Token inválido ou expirado"));
        }

        // 2. Extrair informações do usuário
        let user_info = Self::get_user_info_from_token(&token_info)?;

        // 3. Buscar ou criar usuário no banco local
        let user_with_profile = Self::find_or_create_user(&user_info, app_state).await?;

        // 4. Gerar JWT customizado
        let access_level = Self::map_keycloak_roles_to_access_level(&user_info);
        let token = generate_jwt(
            &user_with_profile.id.to_string(),
            &access_level,
            Uuid::new_v4(),
        )
        .map_err(|_| AppError::internal("Erro ao gerar token JWT"))?;

        // 5. Calcular tempo de expiração
        let expires_in = calculate_remaining_expiration(&token)
            .map_err(|_| AppError::internal("Erro ao calcular expiração do token"))?
            .to_string();

        // 6. Mapear para resposta
        let email = user_with_profile.email.clone();
        let auth_data = KeycloakAuthData {
            user: user_with_profile,
            token,
            expires_in,
        };

        info!("Login via Keycloak realizado com sucesso para: {}", email);

        Ok(auth_data)
    }

    /// Validar provider token com Keycloak
    async fn validate_provider_token(
        client: &Client,
        config: &KeycloakConfig,
        token: &str,
    ) -> Result<KeycloakTokenIntrospectResponse, AppError> {
        let params = [
            ("token", token),
            ("client_id", &config.client_id),
            ("client_secret", &config.client_secret),
        ];

        let response = client
            .post(&config.token_introspect_url())
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                error!("Erro ao validar token com Keycloak: {}", e);
                AppError::internal("Erro ao validar token com Keycloak")
            })?;

        if !response.status().is_success() {
            error!("Keycloak retornou erro: {}", response.status());
            return Err(AppError::unauthorized("Token inválido"));
        }

        let token_info: KeycloakTokenIntrospectResponse = response.json().await.map_err(|e| {
            error!("Erro ao deserializar resposta do Keycloak: {}", e);
            AppError::internal("Erro ao processar resposta do Keycloak")
        })?;

        Ok(token_info)
    }

    /// Extrair informações do usuário do token
    fn get_user_info_from_token(
        token_info: &KeycloakTokenIntrospectResponse,
    ) -> Result<KeycloakUserInfo, AppError> {
        let email = token_info
            .email
            .as_ref()
            .ok_or_else(|| AppError::bad_request("Email não encontrado no token"))?;

        let given_name = token_info
            .given_name
            .as_ref()
            .unwrap_or(&"".to_string())
            .clone();

        let family_name = token_info
            .family_name
            .as_ref()
            .unwrap_or(&"".to_string())
            .clone();

        let preferred_username = token_info
            .preferred_username
            .as_ref()
            .unwrap_or(&"".to_string())
            .clone();

        let name = token_info.name.as_ref().unwrap_or(&"".to_string()).clone();

        let email_verified = token_info.email_verified.unwrap_or(false);

        let realm_access = token_info.realm_access.clone();
        let resource_access = token_info.resource_access.clone();

        let sub = token_info
            .sub
            .as_ref()
            .ok_or_else(|| AppError::bad_request("Subject não encontrado no token"))?
            .clone();

        Ok(KeycloakUserInfo {
            sub,
            email_verified,
            name,
            preferred_username,
            given_name,
            family_name,
            email: email.clone(),
            realm_access,
            resource_access,
        })
    }

    /// Buscar ou criar usuário no banco local
    async fn find_or_create_user(
        user_info: &KeycloakUserInfo,
        app_state: &AppState,
    ) -> Result<UserWithProfile, AppError> {
        let user_repo = UserRepository::new(app_state);
        let profile_repo = ProfileRepository::new(app_state);

        // Tentar buscar usuário por email
        let existing_user = user_repo.find_by_email(&user_info.email).await?;

        match existing_user {
            Some(user) => {
                info!("Usuário encontrado: {}", user.email);
                let profile = profile_repo
                    .find_by_user_id(user.id)
                    .await?
                    .ok_or_else(|| AppError::not_found("Perfil não encontrado"))?;

                let repository_tenant = TenantRepository::new(app_state);
                let tenant = repository_tenant
                    .find_by_user_id(user.id)
                    .await
                    .map_err(|e| AppError::database_error(e.to_string()))?;

                Ok(UserWithProfile::from_user_and_profile_ref(
                    &user, &profile, &tenant,
                ))
            }
            None => {
                info!("Criando novo usuário: {}", user_info.email);
                Self::create_user_from_keycloak(user_info, app_state).await
            }
        }
    }

    /// Criar usuário a partir dos dados do Keycloak
    async fn create_user_from_keycloak(
        user_info: &KeycloakUserInfo,
        app_state: &AppState,
    ) -> Result<UserWithProfile, AppError> {
        let user_repo = UserRepository::new(app_state);

        // Gerar username a partir do email
        let username = crate::utils::formatter::generate_username_from_email(&user_info.email);

        // Criar usuário (sem senha, pois autenticação é via Keycloak)
        let user = User {
            id: Uuid::new_v4(),
            username: username.clone(),
            email: user_info.email.clone(),
            first_name: user_info.given_name.clone(),
            last_name: user_info.family_name.clone(),
            password: "".to_string(), // Senha vazia para usuários Keycloak
            dt_created: chrono::Utc::now(),
            dt_updated: chrono::Utc::now(),
            dt_deleted: None,
        };

        // Criar perfil
        let access_level = Self::map_keycloak_roles_to_access_level(user_info);
        let profile = Profile {
            id: Uuid::new_v4(),
            user_id: user.id,
            bio: None,
            birth_date: None,
            phone: None,
            document: None,
            profession: None,
            avatar: None,
            confirm_email: user_info.email_verified,
            unsubscribe: false,
            access_level,
            dt_created: chrono::Utc::now(),
            dt_updated: chrono::Utc::now(),
        };

        // Salvar no banco
        user_repo.create_user_with_profile(&user, &profile).await?;

        let repository_tenant = TenantRepository::new(app_state);
        let tenant = repository_tenant
            .create(user.id, "api_template")
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        // Criar UserWithProfile para retorno
        let user_with_profile =
            UserWithProfile::from_user_and_profile_ref(&user, &profile, &tenant);

        // Sincronizar usuário com orquestradores (apenas se foi criado)
        if let Err(sync_error) =
            SyncProducer::sync_user(user_with_profile.clone(), app_state, None).await
        {
            error!(
                "Erro ao sincronizar usuário com orquestradores: {}",
                sync_error
            );
            // Não falha o login se a sincronização falhar
        }

        Ok(user_with_profile)
    }

    /// Mapear roles do Keycloak para access_level
    fn map_keycloak_roles_to_access_level(user_info: &KeycloakUserInfo) -> String {
        // Verificar roles do realm
        if let Some(realm_access) = &user_info.realm_access {
            if realm_access.roles.contains(&"super_admin".to_string()) {
                return "super_admin".to_string();
            }
            if realm_access.roles.contains(&"admin".to_string()) {
                return "admin".to_string();
            }
        }

        // Verificar roles do client
        if let Some(resource_access) = &user_info.resource_access {
            if let Some(client_access) = &resource_access.rust_template_client {
                if client_access.roles.contains(&"super_admin".to_string()) {
                    return "super_admin".to_string();
                }
                if client_access.roles.contains(&"admin".to_string()) {
                    return "admin".to_string();
                }
            }
        }

        // Default para user
        "user".to_string()
    }
}
