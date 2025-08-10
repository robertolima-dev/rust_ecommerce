use crate::apps::tenant::models::Tenant;
use crate::utils::validation::{
    validate_birth_date, validate_document, validate_email, validate_password, validate_phone,
};
use bcrypt::{DEFAULT_COST, hash, verify};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ===== AUTH MODELS =====
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(custom = "validate_email")]
    pub email: String,

    #[validate(custom = "validate_password")]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ForgotPasswordRequest {
    #[validate(custom = "validate_email")]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    pub code: String,
    #[validate(custom = "validate_password")]
    pub password: String,
}

// ===== USER TOKEN MODELS =====
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub code: String,
    pub token_type: String,
    pub expires_at: DateTime<Utc>,
    pub consumed: bool,
    pub dt_created: DateTime<Utc>,
}

impl UserToken {
    pub fn new(user_id: Uuid, code: String, token_type: String) -> Self {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::hours(1); // 1 hora de expiração

        Self {
            id: Uuid::new_v4(),
            user_id,
            code,
            token_type,
            expires_at,
            consumed: false,
            dt_created: now,
        }
    }
}

// ===== PROFILE MODELS =====
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub bio: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub phone: Option<String>,
    pub document: Option<String>,
    pub profession: Option<String>,
    pub avatar: Option<String>,
    pub confirm_email: bool,
    pub unsubscribe: bool,
    pub access_level: String,
    pub dt_updated: DateTime<Utc>,
    pub dt_created: DateTime<Utc>,
}

impl Profile {
    pub fn new(user_id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            bio: None,
            birth_date: None,
            phone: None,
            document: None,
            profession: None,
            avatar: None,
            confirm_email: false,
            unsubscribe: false,
            access_level: "user".to_string(),
            dt_created: now,
            dt_updated: now,
        }
    }

    pub fn from_request(user_id: Uuid, req: Option<ProfileRequest>) -> Self {
        let now = Utc::now();

        if let Some(profile) = req {
            Self {
                id: Uuid::new_v4(),
                user_id,
                bio: profile.bio,
                birth_date: profile
                    .birth_date
                    .and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
                phone: profile.phone,
                document: profile.document,
                profession: profile.profession,
                avatar: profile.avatar,
                confirm_email: profile.confirm_email.unwrap_or(false),
                unsubscribe: profile.unsubscribe.unwrap_or(false),
                access_level: profile.access_level.unwrap_or_else(|| "user".to_string()),
                dt_created: now,
                dt_updated: now,
            }
        } else {
            Self::new(user_id)
        }
    }
}

#[derive(Debug, Deserialize, Validate, Serialize)]
pub struct ProfileRequest {
    #[validate(length(
        min = 3,
        max = 500,
        message = "A bio deve ter entre 3 e 500 caracteres"
    ))]
    pub bio: Option<String>,

    #[validate(custom = "validate_birth_date")]
    pub birth_date: Option<String>,

    #[validate(custom = "validate_phone")]
    pub phone: Option<String>,

    #[validate(custom = "validate_document")]
    pub document: Option<String>,

    #[validate(length(
        min = 2,
        max = 100,
        message = "A profissão deve ter entre 2 e 100 caracteres"
    ))]
    pub profession: Option<String>,

    #[validate(url(message = "URL do avatar inválida"))]
    pub avatar: Option<String>,

    pub confirm_email: Option<bool>,
    pub unsubscribe: Option<bool>,

    #[validate(length(
        min = 2,
        max = 50,
        message = "O nível de acesso deve ter entre 2 e 50 caracteres"
    ))]
    pub access_level: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(
        min = 3,
        max = 500,
        message = "A bio deve ter entre 3 e 500 caracteres"
    ))]
    pub bio: Option<String>,

    #[validate(custom = "validate_phone")]
    pub phone: Option<String>,

    pub birth_date: Option<NaiveDate>,

    #[validate(length(
        min = 2,
        max = 100,
        message = "A profissão deve ter entre 2 e 100 caracteres"
    ))]
    pub profession: Option<String>,

    #[validate(custom = "validate_document")]
    pub document: Option<String>,

    #[validate(url(message = "URL do avatar inválida"))]
    pub avatar: Option<String>,
}

// ===== USER MODELS =====
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub dt_created: DateTime<Utc>,
    pub dt_updated: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub dt_deleted: Option<DateTime<Utc>>,
}

impl User {
    pub fn new(
        username: &str,
        email: &str,
        first_name: &str,
        last_name: &str,
        password: &str,
    ) -> Result<Self, bcrypt::BcryptError> {
        let hashed = hash(password, DEFAULT_COST)?;
        let now = Utc::now();

        Ok(Self {
            id: Uuid::new_v4(),
            username: username.to_string(),
            email: email.to_string(),
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            password: hashed,
            dt_created: now,
            dt_updated: now,
            dt_deleted: None,
        })
    }

    pub fn verify_password(&self, input: &str) -> bool {
        verify(input, &self.password).unwrap_or(false)
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UserRequest {
    #[validate(custom = "validate_email")]
    pub email: String,

    #[validate(length(min = 2, max = 50, message = "O nome deve ter entre 2 e 50 caracteres"))]
    pub first_name: String,

    #[validate(length(
        min = 2,
        max = 50,
        message = "O sobrenome deve ter entre 2 e 50 caracteres"
    ))]
    pub last_name: String,

    #[validate(custom = "validate_password")]
    pub password: String,

    #[validate]
    pub profile: Option<ProfileRequest>,

    pub tenant_id: Option<Uuid>,
}

#[derive(Serialize, Clone, Debug)]
pub struct UserWithProfile {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub profile: Profile,
    pub tenant: Tenant,
}

impl UserWithProfile {
    pub fn _from_user_and_profile(user: User, profile: Profile, tenant: Tenant) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            profile,
            tenant,
        }
    }

    pub fn from_user_and_profile_ref(user: &User, profile: &Profile, tenant: &Tenant) -> Self {
        Self {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            profile: profile.clone(),
            tenant: tenant.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct UserResponse {
    pub user: UserWithProfile,
    pub expires_in: String,
    pub token: String,
}

impl UserResponse {
    pub fn from(user_with_profile: UserWithProfile, token: String, expires_in: String) -> Self {
        Self {
            user: user_with_profile,
            token,
            expires_in,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 2, max = 50, message = "O nome deve ter entre 2 e 50 caracteres"))]
    pub first_name: Option<String>,

    #[validate(length(
        min = 2,
        max = 50,
        message = "O sobrenome deve ter entre 2 e 50 caracteres"
    ))]
    pub last_name: Option<String>,
}
