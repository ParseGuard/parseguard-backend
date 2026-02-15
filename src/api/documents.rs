use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    db::repository::DocumentRepository,
    error::{AppError, AppResult},
    models::{Claims, CreateDocumentDto, Document, DocumentResponse, UpdateDocumentDto},
    AppState,
};

/// DTO for creating document from text
#[derive(serde::Deserialize)]
pub struct CreateTextDocumentDto {
    pub title: String,
    pub content: String,
}

/// Create document from text content
pub async fn create_from_text(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(dto): Json<CreateTextDocumentDto>,
) -> AppResult<(StatusCode, Json<Document>)> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user ID in token".to_string()))?;

    // valid filename
    let safe_filename = dto.title
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>();
    let filename = format!("{}.txt", safe_filename);
    let unique_id = Uuid::new_v4();
    let stored_filename = format!("{}_{}", unique_id, filename);
    let file_path = std::path::Path::new(&state.config.upload_dir).join(&stored_filename);

    // Ensure upload directory exists
    tokio::fs::create_dir_all(&state.config.upload_dir)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create upload dir: {}", e)))?;

    // Write file
    tokio::fs::write(&file_path, &dto.content)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to save file: {}", e)))?;

    let file_size = dto.content.len() as i64;

    let create_dto = CreateDocumentDto {
        filename,
        file_path: file_path.to_string_lossy().to_string(),
        file_size,
        mime_type: "text/plain".to_string(),
        extracted_text: Some(dto.content),
    };

    let repo = DocumentRepository::new(state.pool.clone());
    let document = repo.create(user_id, &create_dto).await?;

    Ok((StatusCode::CREATED, Json(document)))
}

/// Get all documents for authenticated user
///
/// # Arguments
///
/// * `state` - Application state
/// * `claims` - Authenticated user claims from middleware
///
/// # Returns
///
/// List of documents
///
/// # Errors
///
/// Returns database error if query fails
pub async fn list_documents(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<Vec<DocumentResponse>>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user ID in token".to_string()))?;

    let repo = DocumentRepository::new(state.pool.clone());
    let documents = repo.find_by_user(user_id).await?;

    let responses: Vec<DocumentResponse> = documents
        .into_iter()
        .map(DocumentResponse::from)
        .collect();

    Ok(Json(responses))
}

/// Get single document by ID
///
/// # Arguments
///
/// * `state` - Application state
/// * `id` - Document UUID
/// * `claims` - Authenticated user claims
///
/// # Returns
///
/// Document with full details including AI analysis
///
/// # Errors
///
/// Returns 404 if not found or not authorized
pub async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
) -> AppResult<Json<Document>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user ID in token".to_string()))?;

    let repo = DocumentRepository::new(state.pool.clone());
    let document = repo.find_by_id(id, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

    Ok(Json(document))
}

/// Create document record (after file upload)
///
/// # Arguments
///
/// * `state` - Application state
/// * `dto` - Document creation data
/// * `claims` - Authenticated user claims
///
/// # Returns
///
/// Created document
///
/// # Errors
///
/// Returns validation or database error
pub async fn create_document(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(dto): Json<CreateDocumentDto>,
) -> AppResult<(StatusCode, Json<Document>)> {
    // Validate input
    dto.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user ID in token".to_string()))?;

    let repo = DocumentRepository::new(state.pool.clone());
    let document = repo.create(user_id, &dto).await?;

    Ok((StatusCode::CREATED, Json(document)))
}

/// Update document with AI analysis
///
/// # Arguments
///
/// * `state` - Application state
/// * `id` - Document UUID
/// * `dto` - Update data
/// * `claims` - Authenticated user claims
///
/// # Returns
///
/// Updated document
///
/// # Errors
///
/// Returns 404 if not found or validation error
pub async fn update_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
    Json(dto): Json<UpdateDocumentDto>,
) -> AppResult<Json<Document>> {
    // Validate input
    dto.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user ID in token".to_string()))?;

    let repo = DocumentRepository::new(state.pool.clone());
    let document = repo.update(id, user_id, &dto)
        .await?
        .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

    Ok(Json(document))
}

/// Delete document
///
/// # Arguments
///
/// * `state` - Application state
/// * `id` - Document UUID
/// * `claims` - Authenticated user claims
///
/// # Returns
///
/// 204 No Content on success
///
/// # Errors
///
/// Returns 404 if not found or not authorized
pub async fn delete_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
) -> AppResult<StatusCode> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user ID in token".to_string()))?;

    let repo = DocumentRepository::new(state.pool.clone());
    let deleted = repo.delete(id, user_id).await?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Document not found".to_string()))
    }
}
