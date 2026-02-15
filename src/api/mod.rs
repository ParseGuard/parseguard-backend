use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

mod auth;
mod compliance;

use crate::{middleware::auth_middleware, AppState};

/// Create the API router with all endpoints
///
/// # Returns
///
/// Returns an Axum Router with all API routes configured
pub fn router() -> Router<AppState> {
    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login));

   // Protected routes (auth required)
    let protected_routes = Router::new()
        .route("/compliance", get(compliance::list_compliance))
        .route("/compliance", post(compliance::create_compliance))
        .route("/compliance/:id", get(compliance::get_compliance))
        .route("/compliance/:id", put(compliance::update_compliance))
        .route("/compliance/:id", delete(compliance::delete_compliance))
        .route_layer(middleware::from_fn(auth_middleware));

    // Combine routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
}
