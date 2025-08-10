use crate::app_core::app_state::AppState;
use crate::apps::user::models::{Profile, UpdateUserRequest, User, UserToken};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ===== USER REPOSITORY =====
pub struct UserRepository<'a> {
    app_state: &'a AppState,
}

impl<'a> UserRepository<'a> {
    pub fn new(app_state: &'a AppState) -> Self {
        Self { app_state }
    }

    /// Buscar usuário por ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<User, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT 
                id, username, email, first_name, last_name, password,
                dt_created, dt_updated, dt_deleted
            FROM users
            WHERE dt_deleted IS NULL AND id = $1
            "#,
            id
        )
        .fetch_one(&self.app_state.db)
        .await?;

        Ok(User {
            id: row.id,
            username: row.username,
            email: row.email,
            first_name: row.first_name,
            last_name: row.last_name,
            password: row.password,
            dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
            dt_updated: DateTime::from_naive_utc_and_offset(row.dt_updated, Utc),
            dt_deleted: row
                .dt_deleted
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        })
    }

    /// Buscar usuário por email
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT 
                id, username, email, first_name, last_name, password,
                dt_created, dt_updated, dt_deleted
            FROM users
            WHERE dt_deleted IS NULL AND email = $1
            "#,
            email
        )
        .fetch_optional(&self.app_state.db)
        .await?;

        Ok(row.map(|row| User {
            id: row.id,
            username: row.username,
            email: row.email,
            first_name: row.first_name,
            last_name: row.last_name,
            password: row.password,
            dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
            dt_updated: DateTime::from_naive_utc_and_offset(row.dt_updated, Utc),
            dt_deleted: row
                .dt_deleted
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        }))
    }

    /// Contar total de usuários
    pub async fn count_all(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE dt_deleted IS NULL")
            .fetch_one(&self.app_state.db)
            .await?;
        Ok(row.0)
    }

    /// Listar usuários paginados
    pub async fn find_all_paginated(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<User>, sqlx::Error> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id, username, email, first_name, last_name, password,
                dt_created, dt_updated, dt_deleted
            FROM users
            WHERE dt_deleted IS NULL
            ORDER BY dt_created DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.app_state.db)
        .await?;

        let users = rows
            .into_iter()
            .map(|row| User {
                id: row.id,
                username: row.username,
                email: row.email,
                first_name: row.first_name,
                last_name: row.last_name,
                password: row.password,
                dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
                dt_updated: DateTime::from_naive_utc_and_offset(row.dt_updated, Utc),
                dt_deleted: row
                    .dt_deleted
                    .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
            })
            .collect();

        Ok(users)
    }

    /// Criar usuário e perfil em transação
    pub async fn create_user_with_profile(
        &self,
        user: &User,
        profile: &Profile,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.app_state.db.begin().await?;

        // Criar usuário
        sqlx::query!(
            r#"
            INSERT INTO users (id, username, email, first_name, last_name, password, dt_created, dt_updated, dt_deleted)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            user.id,
            user.username,
            user.email,
            user.first_name,
            user.last_name,
            user.password,
            user.dt_created.naive_utc(),
            user.dt_updated.naive_utc(),
            user.dt_deleted.map(|dt| dt.naive_utc())
        )
        .execute(&mut *tx)
        .await?;

        // Criar perfil
        sqlx::query!(
            r#"
            INSERT INTO profiles (
                id, user_id, bio, birth_date, phone, document, profession, avatar,
                confirm_email, unsubscribe, access_level, dt_created, dt_updated
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
            profile.id,
            profile.user_id,
            profile.bio,
            profile.birth_date,
            profile.phone,
            profile.document,
            profile.profession,
            profile.avatar,
            profile.confirm_email,
            profile.unsubscribe,
            profile.access_level,
            profile.dt_created.naive_utc(),
            profile.dt_updated.naive_utc()
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    /// Atualizar campos do usuário
    pub async fn update_user_fields(
        &self,
        user_id: Uuid,
        request: UpdateUserRequest,
    ) -> Result<User, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            UPDATE users
            SET 
                first_name = COALESCE($1, first_name),
                last_name = COALESCE($2, last_name),
                dt_updated = $3
            WHERE id = $4 AND dt_deleted IS NULL
            RETURNING id, username, email, first_name, last_name, password,
                      dt_created, dt_updated, dt_deleted
            "#,
            request.first_name,
            request.last_name,
            Utc::now().naive_utc(),
            user_id
        )
        .fetch_one(&self.app_state.db)
        .await?;

        Ok(User {
            id: row.id,
            username: row.username,
            email: row.email,
            first_name: row.first_name,
            last_name: row.last_name,
            password: row.password,
            dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
            dt_updated: DateTime::from_naive_utc_and_offset(row.dt_updated, Utc),
            dt_deleted: row
                .dt_deleted
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        })
    }

    /// Soft delete do usuário
    pub async fn soft_delete(&self, user_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            UPDATE users
            SET dt_deleted = $1
            WHERE id = $2 AND dt_deleted IS NULL
            "#,
            Utc::now().naive_utc(),
            user_id
        )
        .execute(&self.app_state.db)
        .await?;

        Ok(result.rows_affected())
    }

    /// Atualizar senha do usuário
    pub async fn update_password(
        &self,
        user_id: Uuid,
        hashed_password: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE users
            SET password = $1, dt_updated = $2
            WHERE id = $3 AND dt_deleted IS NULL
            "#,
            hashed_password,
            Utc::now().naive_utc(),
            user_id
        )
        .execute(&self.app_state.db)
        .await?;

        Ok(())
    }
}

// ===== PROFILE REPOSITORY =====
pub struct ProfileRepository<'a> {
    app_state: &'a AppState,
}

impl<'a> ProfileRepository<'a> {
    pub fn new(app_state: &'a AppState) -> Self {
        Self { app_state }
    }

    /// Buscar perfil por user_id
    pub async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<Profile>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT 
                id, user_id, bio, birth_date, phone, document, profession, avatar,
                confirm_email, unsubscribe, access_level, dt_created, dt_updated
            FROM profiles
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.app_state.db)
        .await?;

        Ok(row.map(|row| Profile {
            id: row.id,
            user_id: row.user_id,
            bio: row.bio,
            birth_date: row.birth_date,
            phone: row.phone,
            document: row.document,
            profession: row.profession,
            avatar: row.avatar,
            confirm_email: row.confirm_email.unwrap_or(false),
            unsubscribe: row.unsubscribe.unwrap_or(false),
            access_level: row.access_level.unwrap_or_else(|| "user".to_string()),
            dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
            dt_updated: DateTime::from_naive_utc_and_offset(row.dt_updated, Utc),
        }))
    }

    /// Atualizar perfil
    pub async fn update(&self, user_id: Uuid, profile: &Profile) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE profiles
            SET 
                bio = $1, birth_date = $2, phone = $3, document = $4,
                profession = $5, avatar = $6, dt_updated = $7
            WHERE user_id = $8
            "#,
            profile.bio,
            profile.birth_date,
            profile.phone,
            profile.document,
            profile.profession,
            profile.avatar,
            Utc::now().naive_utc(),
            user_id
        )
        .execute(&self.app_state.db)
        .await?;

        Ok(())
    }

    /// Confirmar email
    pub async fn confirm_email(&self, user_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE profiles
            SET confirm_email = true, dt_updated = $1
            WHERE user_id = $2
            "#,
            Utc::now().naive_utc(),
            user_id
        )
        .execute(&self.app_state.db)
        .await?;

        Ok(())
    }
}

// ===== TOKEN REPOSITORY =====
pub struct TokenRepository<'a> {
    app_state: &'a AppState,
}

impl<'a> TokenRepository<'a> {
    pub fn new(app_state: &'a AppState) -> Self {
        Self { app_state }
    }

    /// Criar token para usuário
    pub async fn create_token(
        &self,
        user_id: Uuid,
        token_type: &str,
    ) -> Result<UserToken, sqlx::Error> {
        let code = Uuid::new_v4().to_string();
        let token = UserToken::new(user_id, code.clone(), token_type.to_string());

        sqlx::query!(
            r#"
            INSERT INTO user_tokens (id, user_id, code, token_type, expires_at, consumed, dt_created)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            token.id,
            token.user_id,
            token.code,
            token.token_type,
            token.expires_at.naive_utc(),
            token.consumed,
            token.dt_created.naive_utc()
        )
        .execute(&self.app_state.db)
        .await?;

        Ok(token)
    }

    /// Buscar token válido por user_id e token_type
    pub async fn _find_valid_token(
        &self,
        user_id: Uuid,
        token_type: &str,
    ) -> Result<Option<UserToken>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id, user_id, code, token_type, expires_at, consumed, dt_created
            FROM user_tokens
            WHERE user_id = $1 
                AND token_type = $2 
                AND expires_at > $3 
                AND consumed = FALSE
            ORDER BY dt_created DESC
            LIMIT 1
            "#,
            user_id,
            token_type,
            Utc::now().naive_utc()
        )
        .fetch_optional(&self.app_state.db)
        .await?;

        Ok(row.map(|row| UserToken {
            id: row.id,
            user_id: row.user_id,
            code: row.code,
            token_type: row
                .token_type
                .unwrap_or_else(|| "reset_password".to_string()),
            expires_at: DateTime::from_naive_utc_and_offset(row.expires_at, Utc),
            consumed: row.consumed.unwrap_or(false),
            dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
        }))
    }

    /// Buscar token válido por código
    pub async fn find_valid_token_by_code(
        &self,
        code: &str,
        token_type: &str,
    ) -> Result<Option<UserToken>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id, user_id, code, token_type, expires_at, consumed, dt_created
            FROM user_tokens
            WHERE code = $1 
                AND token_type = $2 
                AND expires_at > $3 
                AND consumed = FALSE
            ORDER BY dt_created DESC
            LIMIT 1
            "#,
            code,
            token_type,
            Utc::now().naive_utc()
        )
        .fetch_optional(&self.app_state.db)
        .await?;

        Ok(row.map(|row| UserToken {
            id: row.id,
            user_id: row.user_id,
            code: row.code,
            token_type: row
                .token_type
                .unwrap_or_else(|| "reset_password".to_string()),
            expires_at: DateTime::from_naive_utc_and_offset(row.expires_at, Utc),
            consumed: row.consumed.unwrap_or(false),
            dt_created: DateTime::from_naive_utc_and_offset(row.dt_created, Utc),
        }))
    }

    /// Marcar token como consumido
    pub async fn mark_as_consumed(&self, token_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE user_tokens
            SET consumed = TRUE
            WHERE id = $1
            "#,
            token_id
        )
        .execute(&self.app_state.db)
        .await?;

        Ok(())
    }
}
