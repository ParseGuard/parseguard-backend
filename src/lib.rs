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
