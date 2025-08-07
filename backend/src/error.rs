use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Document not found: {0}")]
    DocumentNotFound(String),
    
    #[error("Invalid input: {0}")]
    ValidationError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Migration error: {0}")]
    MigrationError(#[from] sqlx::migrate::MigrateError),
    
    #[error("Internal server error: {0}")]
    InternalError(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    #[error("Authorization error: {0}")]
    AuthorizationError(String),
    
    #[error("User not found: {0}")]
    UserNotFound(String),
    
    #[error("User already exists: {0}")]
    UserAlreadyExists(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::DocumentNotFound(id) => {
                (StatusCode::NOT_FOUND, format!("Document not found: {}", id))
            }
            AppError::ValidationError(msg) => {
                (StatusCode::BAD_REQUEST, format!("Validation error: {}", msg))
            }
            AppError::DatabaseError(e) => {
                tracing::error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::MigrationError(e) => {
                tracing::error!("Migration error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::InternalError(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            AppError::RateLimitExceeded => {
                (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".to_string())
            }
            AppError::AuthenticationError(msg) => {
                (StatusCode::UNAUTHORIZED, format!("Authentication error: {}", msg))
            }
            AppError::AuthorizationError(msg) => {
                (StatusCode::FORBIDDEN, format!("Authorization error: {}", msg))
            }
            AppError::UserNotFound(email) => {
                (StatusCode::NOT_FOUND, format!("User not found: {}", email))
            }
            AppError::UserAlreadyExists(email) => {
                (StatusCode::CONFLICT, format!("User already exists: {}", email))
            }
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>; 