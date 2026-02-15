use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    db::repository::ComplianceRepository,
    error::{AppError, AppResult},
    models::{Claims, ComplianceItem, CreateComplianceDto, UpdateComplianceDto},
    AppState,
};

/// Get all compliance items for authenticated user
///
/// # Arguments
///
/// * `state` - Application state
/// * `claims` - Authenticated user claims from middleware
///
/// # Returns
///
/// List of compliance items
///
/// # Errors
///
/// Returns database error if query fails
pub async fn list_compliance(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<Vec<ComplianceItem>>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user ID in token".to_string()))?;

    let repo = ComplianceRepository::new(state.pool.clone());
    let items = repo.find_by_user(user_id).await?;

    Ok(Json(items))
}

/// Get single compliance item by ID
///
/// # Arguments
///
/// * `state` - Application state
/// * `id` - Compliance item UUID
/// * `claims` - Authenticated user claims
///
/// # Returns
///
/// Compliance item if found and owned by user
///
/// # Errors
///
/// Returns 404 if not found or not authorized
pub async fn get_compliance(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<ComplianceItem>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user ID in tokenToken".to_string()))?;

    let repo = ComplianceRepository::new(state.pool.clone());
    let item = repo.find_by_id(id, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Compliance item not found".to_string()))?;

    Ok(Json(item))
}

/// Create new compliance item
///
/// # Arguments
///
/// * `state` - Application state
/// * `dto` - Compliance item creation data
/// * `claims` - Authenticated user claims
///
/// # Returns
///
/// Created compliance item
///
/// # Errors
///
/// Returns validation or database error
pub async fn create_compliance(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(dto): Json<CreateComplianceDto>,
) -> AppResult<(StatusCode, Json<ComplianceItem>)> {
    // Validate input
    dto.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user ID in token".to_string()))?;

    let repo = ComplianceRepository::new(state.pool.clone());
    let item = repo.create(user_id, &dto).await?;

    Ok((StatusCode::CREATED, Json(item)))
}

/// Update compliance item
///
/// # Arguments
///
/// * `state` - Application state
/// * `id` - Compliance item UUID
/// * `dto` - Update data
/// * `claims` - Authenticated user claims
///
/// # Returns
///
/// Updated compliance item
///
/// # Errors
///
/// Returns 404 if not found or validation error
pub async fn update_compliance(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
    Json(dto): Json<UpdateComplianceDto>,
) -> AppResult<Json<ComplianceItem>> {
    // Validate input
    dto.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user ID in token".to_string()))?;

    let repo = ComplianceRepository::new(state.pool.clone());
    let item = repo.update(id, user_id, &dto)
        .await?
        .ok_or_else(|| AppError::NotFound("Compliance item not found".to_string()))?;

    Ok(Json(item))
}

/// Delete compliance item
///
/// # Arguments
///
/// * `state` - Application state
/// * `id` - Compliance item UUID
/// * `claims` - Authenticated user claims
///
/// # Returns
///
/// 204 No Content on success
///
/// # Errors
///
/// Returns 404 if not found or not authorized
pub async fn delete_compliance(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
) -> AppResult<StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user ID in token".to_string()))?;

    let repo = ComplianceRepository::new(state.pool.clone());
    let deleted = repo.delete(id, user_id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Compliance item not found".to_string()))
    }
}
