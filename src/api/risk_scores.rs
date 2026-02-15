use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    db::repository::RiskScoreRepository,
    error::{AppError, AppResult},
    models::{Claims, CreateRiskScoreDto, RiskScore, UpdateRiskScoreDto},
    AppState,
};

/// List all risk scores for authenticated user
///
/// # Arguments
///
/// * `state` - Application state
/// * `claims` - Authenticated user claims
///
/// # Returns
///
/// List of risk scores
///
/// # Errors
///
/// Returns database error if query fails
pub async fn list_scores(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<Vec<RiskScore>>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user ID in token".to_string()))?;

    let repo = RiskScoreRepository::new(state.pool.clone());
    let scores = repo.find_by_user(user_id).await?;

    Ok(Json(scores))
}

/// Get risk scores for specific compliance item
///
/// # Arguments
///
/// * `state` - Application state
/// * `id` - Compliance item UUID
/// * `claims` - Authenticated user claims
///
/// # Returns
///
/// List of risk scores for the item
///
/// # Errors
///
/// Returns database error if query fails
pub async fn list_by_compliance(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<Vec<RiskScore>>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user ID in token".to_string()))?;

    let repo = RiskScoreRepository::new(state.pool.clone());
    let scores = repo.find_by_compliance_item(id, user_id).await?;

    Ok(Json(scores))
}

/// Get single risk score by ID
///
/// # Arguments
///
/// * `state` - Application state
/// * `id` - Risk score UUID
/// * `claims` - Authenticated user claims
///
/// # Returns
///
/// Risk score with full details
///
/// # Errors
///
/// Returns 404 if not found or not authorized
pub async fn get_score(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<RiskScore>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user ID in token".to_string()))?;

    let repo = RiskScoreRepository::new(state.pool.clone());
    let score = repo.find_by_id(id, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Risk score not found".to_string()))?;

    Ok(Json(score))
}

/// Create new risk score
///
/// # Arguments
///
/// * `state` - Application state
/// * `dto` - Risk score creation data
/// * `claims` - Authenticated user claims
///
/// # Returns
///
/// Created risk score
///
/// # Errors
///
/// Returns validation or database error
pub async fn create_score(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(dto): Json<CreateRiskScoreDto>,
) -> AppResult<(StatusCode, Json<RiskScore>)> {
    // Validate input
    dto.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user ID in token".to_string()))?;

    let repo = RiskScoreRepository::new(state.pool.clone());
    let score = repo.create(user_id, &dto).await?;

    Ok((StatusCode::CREATED, Json(score)))
}

/// Update risk score
///
/// # Arguments
///
/// * `state` - Application state
/// * `id` - Risk score UUID
/// * `dto` - Update data
/// * `claims` - Authenticated user claims
///
/// # Returns
///
/// Updated risk score
///
/// # Errors
///
/// Returns 404 if not found or validation error
pub async fn update_score(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
    Json(dto): Json<UpdateRiskScoreDto>,
) -> AppResult<Json<RiskScore>> {
    // Validate input
    dto.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user ID in token".to_string()))?;

    let repo = RiskScoreRepository::new(state.pool.clone());
    let score = repo.update(id, user_id, &dto)
        .await?
        .ok_or_else(|| AppError::NotFound("Risk score not found".to_string()))?;

    Ok(Json(score))
}

/// Delete risk score
///
/// # Arguments
///
/// * `state` - Application state
/// * `id` - Risk score UUID
/// * `claims` - Authenticated user claims
///
/// # Returns
///
/// 204 No Content on success
///
/// # Errors
///
/// Returns 404 if not found or not authorized
pub async fn delete_score(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
) -> AppResult<StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user ID in token".to_string()))?;

    let repo = RiskScoreRepository::new(state.pool.clone());
    let deleted = repo.delete(id, user_id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Risk score not found".to_string()))
    }
}
