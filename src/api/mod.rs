use axum::{routing::post, Router};

mod auth;

use crate::AppState;

/// Create the API router with all endpoints
///
/// # Returns
///
/// Returns an Axum Router with all API routes configured
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
}
