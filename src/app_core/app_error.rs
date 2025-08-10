use actix_web::{HttpResponse, ResponseError};
use derive_more::Display; // essa macro implementa Display por você!
// use std::fmt::{self, Display, Formatter};

#[derive(Debug, Display)]
#[allow(dead_code)]
pub enum AppError {
    #[display(fmt = "Conflito de dados")]
    Conflict(Option<String>),

    #[display(fmt = "{_0:?}")]
    DatabaseError(Option<String>),

    #[display(fmt = "Recurso não encontrado")]
    NotFound(Option<String>),

    #[display(fmt = "Não autorizado")]
    Unauthorized(Option<String>),

    #[display(fmt = "Acesso negado")]
    Forbidden(Option<String>),

    #[display(fmt = "Requisição inválida")]
    BadRequest(Option<String>),

    #[display(fmt = "Erro interno do servidor")]
    InternalError(Option<String>),
}

#[allow(dead_code)]
impl AppError {
    pub fn database_error<S: Into<String>>(msg: S) -> Self {
        AppError::DatabaseError(Some(msg.into()))
    }

    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        AppError::NotFound(Some(msg.into()))
    }

    pub fn unauthorized<S: Into<String>>(msg: S) -> Self {
        AppError::Unauthorized(Some(msg.into()))
    }

    pub fn forbidden<S: Into<String>>(msg: S) -> Self {
        AppError::Forbidden(Some(msg.into()))
    }

    pub fn bad_request<S: Into<String>>(msg: S) -> Self {
        AppError::BadRequest(Some(msg.into()))
    }

    pub fn internal<S: Into<String>>(msg: S) -> Self {
        AppError::InternalError(Some(msg.into()))
    }
}

// Implementação para conversão automática de sqlx::Error
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::not_found("Registro não encontrado"),
            sqlx::Error::Database(db_err) => {
                if let Some(code) = db_err.code() {
                    match code.as_ref() {
                        "23505" => AppError::Conflict(Some("Registro já existe".into())), // Unique violation
                        "23503" => AppError::BadRequest(Some("Referência inválida".into())), // Foreign key violation
                        _ => {
                            AppError::database_error(format!("Erro no banco: {}", db_err.message()))
                        }
                    }
                } else {
                    AppError::database_error(db_err.message())
                }
            }
            _ => AppError::database_error(err.to_string()),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::Conflict(msg) => {
                HttpResponse::Conflict().json(msg.as_deref().unwrap_or("Conflito de dados"))
            }
            AppError::DatabaseError(msg) => HttpResponse::InternalServerError()
                .json(msg.as_deref().unwrap_or("Erro no banco de dados")),
            AppError::NotFound(msg) => {
                HttpResponse::NotFound().json(msg.as_deref().unwrap_or("Recurso não encontrado"))
            }
            AppError::Unauthorized(msg) => {
                HttpResponse::Unauthorized().json(msg.as_deref().unwrap_or("Não autorizado"))
            }
            AppError::Forbidden(msg) => {
                HttpResponse::Forbidden().json(msg.as_deref().unwrap_or("Acesso negado"))
            }
            AppError::BadRequest(msg) => {
                HttpResponse::BadRequest().json(msg.as_deref().unwrap_or("Requisição inválida"))
            }
            AppError::InternalError(msg) => HttpResponse::InternalServerError()
                .json(msg.as_deref().unwrap_or("Erro interno do servidor")),
        }
    }
}
