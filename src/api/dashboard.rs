use axum::{
    extract::{Extension, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::Claims,
    services::{ActivityItem, DashboardService, DashboardStats},
    AppState,
};

/// Query parameters for activity endpoint
#[derive(Debug, Deserialize)]
pub struct ActivityQuery {
    /// Maximum number of items to return (default: 10)
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    10
}

/// Get dashboard statistics
///
/// # Arguments
///
/// * `state` - Application state
/// * `claims` - Authenticated user claims
///
/// # Returns
///
/// Dashboard statistics for the user
///
/// # Errors
///
/// Returns database error if queries fail
pub async fn get_stats(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<DashboardStats>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user ID in token".to_string()))?;

    let service = DashboardService::new(state.pool.clone());
    let stats = service.get_stats(user_id).await?;

    Ok(Json(stats))
}

/// Get recent activity
///
/// # Arguments
///
/// * `state` - Application state
/// * `query` - Query parameters (limit)
/// * `claims` - Authenticated user claims
///
/// # Returns
///
/// List of recent activity items
///
/// # Errors
///
/// Returns database error if queries fail
pub async fn get_activity(
    State(state): State<AppState>,
    Query(query): Query<ActivityQuery>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<Vec<ActivityItem>>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user ID in token".to_string()))?;

    // Validate limit
    let limit = if query.limit > 0 && query.limit <= 100 {
        query.limit
    } else {
        10
    };

    let service = DashboardService::new(state.pool.clone());
    let activity = service.get_recent_activity(user_id, limit).await?;

    Ok(Json(activity))
}
