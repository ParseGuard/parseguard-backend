mod config;
mod error;
mod api;
mod db;
mod models;
mod middleware;
mod services;

use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use config::Config;
use error::AppResult;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "parseguard_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env();
    tracing::info!("Configuration loaded");

    // Create database connection pool
    let pool = db::create_pool(&config.database_url).await?;
    tracing::info!("Database connection pool created");

    // Run migrations
    db::run_migrations(&pool).await?;
    tracing::info!("Database migrations completed");

    // Create CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build application router
    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/api", api::router())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(AppState {
            pool: pool.clone(),
            config: config.clone(),
        });

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Health check endpoint
///
/// # Returns
///
/// Returns a simple JSON response indicating the service is healthy
async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "service": "parseguard-backend",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool
    pub pool: sqlx::PgPool,
    
    /// Application configuration
    pub config: Config,
}
