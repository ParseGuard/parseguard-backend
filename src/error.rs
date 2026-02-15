use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Custom error type for the application
///
/// This type wraps various error types that can occur throughout the application
/// and provides a consistent error response format.
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    /// Database error from SQLx
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    /// Authentication error
    #[error("Authentication error: {0}")]
    Auth(String),
    
    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
    
    /// Not found error
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    /// Internal server error
    #[error("Internal server error: {0}")]
    Internal(String),
    
    /// JWT error
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    
    /// OLLAMA API error
    #[error("OLLAMA API error: {0}")]
    Ollama(String),
    
    ///IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Database migration error
    #[error("Migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

impl IntoResponse for AppError {
    /// Convert error into HTTP response
    ///
    /// # Returns
    ///
    /// Returns an Axum Response with appropriate status code and JSON error message
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(ref e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred")
            }
            AppError::Auth(_) => (StatusCode::UNAUTHORIZED, "Authentication failed"),
            AppError::Validation(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AppError::NotFound(ref msg) => (StatusCode::NOT_FOUND, msg.as_str()),
            AppError::Internal(ref msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AppError::Jwt(_) => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AppError::Ollama(ref msg) => {
                tracing::error!("OLLAMA error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "AI service error")
            }
            AppError::Io(ref e) => {
                tracing::error!("IO error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AppError::Migrate(ref e) => {
                tracing::error!("Migration error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database migration failed")
            }
        };

        let body = Json(json!({
            "error": error_message,
            "message": self.to_string(),
        }));

        (status, body).into_response()
    }
}

/// Result type alias for application errors
pub type AppResult<T> = Result<T, AppError>;
