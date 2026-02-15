use axum::Router;

use crate::AppState;

/// Create the API router with all endpoints
///
/// # Returns
///
/// Returns an Axum Router with all API routes configured
pub fn router() -> Router<AppState> {
    Router::new()
        // Routes will be added here
}
