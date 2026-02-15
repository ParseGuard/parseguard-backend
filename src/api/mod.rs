use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

mod auth;
mod compliance;
mod dashboard;
mod documents;

use crate::{middleware::auth_middleware, AppState};

/// Create API router with all endpoints
///
/// # Arguments
///
/// * `state` - Application state
///
/// # Returns
///
/// Configured Axum router
pub fn create_router(state: AppState) -> Router {
    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        // Auth refresh
        .route("/auth/refresh", post(auth::refresh))
        // Compliance
        .route("/compliance", get(compliance::list_compliance))
        .route("/compliance", post(compliance::create_compliance))
        .route("/compliance/:id", get(compliance::get_compliance))
        .route("/compliance/:id", put(compliance::update_compliance))
        .route("/compliance/:id", delete(compliance::delete_compliance))
        // Documents
        .route("/documents", get(documents::list_documents))
        .route("/documents", post(documents::create_document))
        .route("/documents/:id", get(documents::get_document))
        .route("/documents/:id", put(documents::update_document))
        .route("/documents/:id", delete(documents::delete_document))
        // Dashboard
        .route("/dashboard/stats", get(dashboard::get_stats))
        .route("/dashboard/activity", get(dashboard::get_activity))
        .route_layer(middleware::from_fn(auth_middleware));

    // Combine routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state)
}
