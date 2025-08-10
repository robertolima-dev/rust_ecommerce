use crate::app_core::app_error::AppError;
use crate::app_core::app_state::AppState;
use crate::apps::sync_app::producer::SyncProducer;
use crate::apps::tenant::repositories::TenantRepository;
use crate::apps::user::models::{
    ChangePasswordRequest, ForgotPasswordRequest, LoginRequest, Profile, UpdateProfileRequest,
    UpdateUserRequest, User, UserRequest, UserResponse, UserWithProfile,
};
use crate::apps::user::repositories::{ProfileRepository, TokenRepository, UserRepository};
use crate::utils::formatter::generate_username_from_email;
use crate::utils::jwt::{calculate_remaining_expiration, generate_jwt};
use crate::utils::pagination::PaginatedResponse;
use bcrypt::{DEFAULT_COST, hash};
use tracing::{error, info};
use uuid::Uuid;
use validator::Validate;

pub struct UserService;

impl UserService {
    // ===== PUBLIC METHODS (sem autenticação) =====

    /// Cria um novo usuário com perfil
    pub async fn create_user_with_profile(
        request: UserRequest,
        app_state: &AppState,
    ) -> Result<UserResponse, AppError> {
        // Validar dados de entrada
        request
            .validate()
            .map_err(|e| AppError::bad_request(format!("Dados inválidos: {}", e)))?;

        let repository = UserRepository::new(app_state);
        let _profile_repo = ProfileRepository::new(app_state);
        let token_repo = TokenRepository::new(app_state);

        // Gerar username a partir do email
        let username = generate_username_from_email(&request.email);

        // Criar usuário
        let user = User::new(
            &username,
            &request.email,
            &request.first_name,
            &request.last_name,
            &request.password,
        )
        .map_err(|_| AppError::internal("Erro ao hashear senha"))?;

        // Criar perfil
        let profile = Profile::from_request(user.id, request.profile);

        // Salvar no banco usando transação
        repository.create_user_with_profile(&user, &profile).await?;

        let repository_tenant = TenantRepository::new(app_state);

        let tenant_id = request.tenant_id;
        let tenant = if let Some(tenant_id) = tenant_id {
            // Se tenant_id foi fornecido, buscar o tenant existente
            repository_tenant
                .find_by_id(tenant_id)
                .await
                .map_err(|e| AppError::database_error(e.to_string()))?
                .ok_or_else(|| AppError::not_found("Tenant não encontrado"))?;

            repository_tenant
                .create_tenant_user(user.id, tenant_id)
                .await
                .map_err(|e| AppError::database_error(e.to_string()))?
        } else {
            // Se não foi fornecido, criar um novo tenant
            repository_tenant
                .create(user.id, "api_template")
                .await
                .map_err(|e| AppError::database_error(e.to_string()))?
        };

        // Criar token de confirmação de email
        let confirm_token = token_repo.create_token(user.id, "confirm_email").await?;

        // TODO: Enviar email com token de confirmação
        // Por enquanto apenas print do token
        println!(
            "Token de confirmação de email para {}: {}",
            request.email, confirm_token.code
        );

        // Gerar token JWT real
        let token = generate_jwt(&user.id.to_string(), &profile.access_level, tenant.id)
            .map_err(|_| AppError::internal("Erro ao gerar token JWT"))?;

        // Calcular tempo restante de expiração
        let expires_in = calculate_remaining_expiration(&token)
            .map_err(|_| AppError::internal("Erro ao calcular expiração do token"))?
            .to_string();

        let user_with_profile =
            UserWithProfile::from_user_and_profile_ref(&user, &profile, &tenant);

        // Sync com SQS após criação do usuário
        if let Err(e) = SyncProducer::sync_user(user_with_profile.clone(), app_state, None).await {
            error!("Erro ao sincronizar usuário criado: {}", e);
            // Não falha a criação do usuário se o sync falhar
        } else {
            info!("Usuário sincronizado com sucesso após criação");
        }

        Ok(UserResponse::from(user_with_profile, token, expires_in))
    }

    /// Login do usuário
    pub async fn login_user(
        request: LoginRequest,
        app_state: &AppState,
    ) -> Result<UserResponse, AppError> {
        // Validar dados de entrada
        request
            .validate()
            .map_err(|e| AppError::bad_request(format!("Dados inválidos: {}", e)))?;

        let repository = UserRepository::new(app_state);
        let profile_repo = ProfileRepository::new(app_state);

        // Buscar usuário por email
        let user = repository
            .find_by_email(&request.email)
            .await?
            .ok_or_else(|| AppError::unauthorized("Credenciais inválidas"))?;

        // Verificar senha
        if !user.verify_password(&request.password) {
            return Err(AppError::unauthorized("Credenciais inválidas"));
        }

        // Buscar perfil
        let profile = profile_repo
            .find_by_user_id(user.id)
            .await?
            .ok_or_else(|| AppError::not_found("Perfil não encontrado"))?;

        let repository_tenant = TenantRepository::new(app_state);
        let tenant = repository_tenant
            .find_by_user_id(user.id)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        // Gerar token JWT real
        let token = generate_jwt(&user.id.to_string(), &profile.access_level, tenant.id)
            .map_err(|_| AppError::internal("Erro ao gerar token JWT"))?;

        // Calcular tempo restante de expiração
        let expires_in = calculate_remaining_expiration(&token)
            .map_err(|_| AppError::internal("Erro ao calcular expiração do token"))?
            .to_string();

        let user_with_profile =
            UserWithProfile::from_user_and_profile_ref(&user, &profile, &tenant);

        Ok(UserResponse::from(user_with_profile, token, expires_in))
    }

    /// Esqueci minha senha
    pub async fn forgot_password(
        request: ForgotPasswordRequest,
        app_state: &AppState,
    ) -> Result<(), AppError> {
        request
            .validate()
            .map_err(|e| AppError::bad_request(format!("Dados inválidos: {}", e)))?;

        let repository = UserRepository::new(app_state);
        let token_repo = TokenRepository::new(app_state);

        // Verificar se usuário existe
        let user = repository
            .find_by_email(&request.email)
            .await?
            .ok_or_else(|| AppError::not_found("Usuário não encontrado"))?;

        // Criar token de reset de senha
        let reset_token = token_repo.create_token(user.id, "reset_password").await?;

        // TODO: Enviar email com token de reset
        // Por enquanto apenas print do token
        println!(
            "Token de reset de senha para {}: {}",
            request.email, reset_token.code
        );

        Ok(())
    }

    /// Alterar senha com código
    pub async fn change_password(
        request: ChangePasswordRequest,
        app_state: &AppState,
    ) -> Result<(), AppError> {
        request
            .validate()
            .map_err(|e| AppError::bad_request(format!("Dados inválidos: {}", e)))?;

        let repository = UserRepository::new(app_state);
        let token_repo = TokenRepository::new(app_state);

        // Buscar token válido por código
        let token = token_repo
            .find_valid_token_by_code(&request.code, "reset_password")
            .await?
            .ok_or_else(|| AppError::bad_request("Token inválido ou expirado"))?;

        // Hash da nova senha
        let hashed_password = hash(&request.password, DEFAULT_COST)
            .map_err(|_| AppError::internal("Erro ao hashear senha"))?;

        // Atualizar senha
        repository
            .update_password(token.user_id, &hashed_password)
            .await?;

        // Marcar token como consumido
        token_repo.mark_as_consumed(token.id).await?;

        Ok(())
    }

    /// Confirmar email com código
    pub async fn confirm_email(code: String, app_state: &AppState) -> Result<(), AppError> {
        let token_repo = TokenRepository::new(app_state);
        let profile_repo = ProfileRepository::new(app_state);
        let user_repo = UserRepository::new(app_state);

        // Buscar token válido
        let token = token_repo
            .find_valid_token_by_code(&code, "confirm_email")
            .await?
            .ok_or_else(|| AppError::bad_request("Token inválido ou expirado"))?;

        // Confirmar email no perfil
        profile_repo.confirm_email(token.user_id).await?;

        // Marcar token como consumido
        token_repo.mark_as_consumed(token.id).await?;

        // Buscar usuário e perfil atualizados para sincronização
        let user = user_repo.find_by_id(token.user_id).await?;
        let profile = profile_repo
            .find_by_user_id(token.user_id)
            .await?
            .ok_or_else(|| AppError::not_found("Perfil não encontrado"))?;

        let repository_tenant = TenantRepository::new(app_state);
        let tenant = repository_tenant
            .find_by_user_id(user.id)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        let user_with_profile =
            UserWithProfile::from_user_and_profile_ref(&user, &profile, &tenant);

        // Sync com orchestrators após confirmação de email
        if let Err(e) = SyncProducer::sync_user(user_with_profile, app_state, None).await {
            error!(
                "Erro ao sincronizar usuário após confirmação de email: {}",
                e
            );
            // Não falha a confirmação se o sync falhar
        } else {
            info!("Usuário sincronizado com sucesso após confirmação de email");
        }

        Ok(())
    }

    // ===== PRIVATE METHODS (com autenticação) =====

    /// Buscar dados do usuário logado
    pub async fn get_me(
        user_id: Uuid,
        token: String,
        app_state: &AppState,
    ) -> Result<UserResponse, AppError> {
        let repository = UserRepository::new(app_state);
        let profile_repo = ProfileRepository::new(app_state);

        let user = repository.find_by_id(user_id).await?;
        let profile = profile_repo
            .find_by_user_id(user.id)
            .await?
            .ok_or_else(|| AppError::not_found("Perfil não encontrado"))?;

        let repository_tenant = TenantRepository::new(app_state);
        let tenant = repository_tenant
            .find_by_user_id(user.id)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        let user_with_profile =
            UserWithProfile::from_user_and_profile_ref(&user, &profile, &tenant);

        // Calcular tempo restante de expiração do token atual
        let expires_in = calculate_remaining_expiration(&token)
            .map_err(|_| AppError::internal("Erro ao calcular expiração do token"))?
            .to_string();

        Ok(UserResponse::from(user_with_profile, token, expires_in))
    }

    /// Listar usuários paginados
    pub async fn list_users_paginated(
        limit: i64,
        offset: i64,
        app_state: &AppState,
    ) -> Result<PaginatedResponse<User>, AppError> {
        let repository = UserRepository::new(app_state);

        let count = repository.count_all().await?;
        let users = repository.find_all_paginated(limit, offset).await?;

        Ok(PaginatedResponse {
            count,
            results: users,
            limit,
            offset,
        })
    }

    /// Atualizar usuário logado
    pub async fn update_user(
        user_id: Uuid,
        request: UpdateUserRequest,
        token: String,
        app_state: &AppState,
    ) -> Result<UserResponse, AppError> {
        request
            .validate()
            .map_err(|e| AppError::bad_request(format!("Dados inválidos: {}", e)))?;

        let repository = UserRepository::new(app_state);
        let profile_repo = ProfileRepository::new(app_state);

        let user = repository.update_user_fields(user_id, request).await?;
        let profile = profile_repo
            .find_by_user_id(user.id)
            .await?
            .ok_or_else(|| AppError::not_found("Perfil não encontrado"))?;

        let repository_tenant = TenantRepository::new(app_state);
        let tenant = repository_tenant
            .find_by_user_id(user.id)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        let user_with_profile =
            UserWithProfile::from_user_and_profile_ref(&user, &profile, &tenant);

        // Sync com SQS após atualização do usuário
        if let Err(e) = SyncProducer::sync_user(user_with_profile.clone(), app_state, None).await {
            error!("Erro ao sincronizar usuário atualizado: {}", e);
        } else {
            info!("Usuário sincronizado com sucesso após atualização");
        }

        // Calcular tempo restante de expiração do token atual
        let expires_in = calculate_remaining_expiration(&token)
            .map_err(|_| AppError::internal("Erro ao calcular expiração do token"))?
            .to_string();

        Ok(UserResponse::from(user_with_profile, token, expires_in))
    }

    /// Atualizar perfil do usuário logado
    pub async fn update_profile(
        user_id: Uuid,
        request: UpdateProfileRequest,
        token: String,
        app_state: &AppState,
    ) -> Result<UserResponse, AppError> {
        request
            .validate()
            .map_err(|e| AppError::bad_request(format!("Dados inválidos: {}", e)))?;

        let repository = UserRepository::new(app_state);
        let profile_repo = ProfileRepository::new(app_state);

        // Buscar usuário e perfil atual
        let user = repository.find_by_id(user_id).await?;
        let mut profile = profile_repo
            .find_by_user_id(user.id)
            .await?
            .ok_or_else(|| AppError::not_found("Perfil não encontrado"))?;

        // Atualizar campos do perfil
        if let Some(bio) = request.bio {
            profile.bio = Some(bio);
        }
        if let Some(phone) = request.phone {
            profile.phone = Some(phone);
        }
        if let Some(birth_date) = request.birth_date {
            profile.birth_date = Some(birth_date);
        }
        if let Some(profession) = request.profession {
            profile.profession = Some(profession);
        }
        if let Some(document) = request.document {
            profile.document = Some(document);
        }
        if let Some(avatar) = request.avatar {
            profile.avatar = Some(avatar);
        }

        // Salvar perfil atualizado
        profile_repo.update(user_id, &profile).await?;

        let repository_tenant = TenantRepository::new(app_state);
        let tenant = repository_tenant
            .find_by_user_id(user_id)
            .await
            .map_err(|e| AppError::database_error(e.to_string()))?;

        let user_with_profile =
            UserWithProfile::from_user_and_profile_ref(&user, &profile, &tenant);

        // Sync com SQS após atualização do perfil
        if let Err(e) = SyncProducer::sync_user(user_with_profile.clone(), app_state, None).await {
            error!("Erro ao sincronizar perfil atualizado: {}", e);
        } else {
            info!("Usuário sincronizado com sucesso após atualização de perfil");
        }

        // Calcular tempo restante de expiração do token atual
        let expires_in = calculate_remaining_expiration(&token)
            .map_err(|_| AppError::internal("Erro ao calcular expiração do token"))?
            .to_string();

        Ok(UserResponse::from(user_with_profile, token, expires_in))
    }

    /// Deletar usuário (soft delete)
    pub async fn delete_user(user_id: Uuid, app_state: &AppState) -> Result<(), AppError> {
        let repository = UserRepository::new(app_state);
        repository.soft_delete(user_id).await?;
        Ok(())
    }
}
