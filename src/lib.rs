pub mod config;
pub mod error;
pub mod api;
pub mod db;
pub mod models;
pub mod middleware;
pub mod services;
pub mod utils;

pub use config::Config;
pub use error::{AppError, AppResult};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool
    pub pool: sqlx::PgPool,
    
    /// Application configuration
    pub config: Config,
}

/// Health check endpoint
///
/// # Returns
///
/// Returns a simple JSON response indicating the service is healthy
pub async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "service": "parseguard-backend",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
