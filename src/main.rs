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

use parseguard_backend::{
    api,
    config::Config,
    db,
    error::AppResult,
    middleware,
    AppState,
};

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
    let cors_layer = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods(
            vec![
                axum::http::Method::GET,
                axum::http::Method::POST,
                axum::http::Method::PUT,
                axum::http::Method::DELETE,
                axum::http::Method::OPTIONS,
            ]
        )
        .allow_headers(
            vec![
                axum::http::header::AUTHORIZATION,
                axum::http::header::CONTENT_TYPE,
                axum::http::header::ACCEPT,
                axum::http::header::COOKIE,
            ]
        )
        .allow_credentials(true);

    // Create AppState
    let state = AppState {
        pool: pool.clone(),
        config: config.clone(),
    };

    // Build application router
    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/api", api::create_router(state))
        .layer(cors_layer)
        .layer(axum::middleware::from_fn(middleware::logger_middleware))
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
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
