use crate::error::{AppError, AppResult};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// File upload result
#[derive(Debug)]
pub struct UploadedFile {
    /// Original filename
    pub original_name: String,
    
    /// Stored filename (UUID-based)
    pub stored_name: String,
    
    /// Full file path
    pub file_path: PathBuf,
    
    /// File size in bytes
    pub size: u64,
    
    /// MIME type
    pub mime_type: String,
}

/// Validate file size
///
/// # Arguments
///
/// * `size` - File size in bytes
/// * `max_size` - Maximum allowed size
///
/// # Returns
///
/// Ok if valid, Err otherwise
///
/// # Errors
///
/// Returns validation error if file is too large
pub fn validate_file_size(size: u64, max_size: usize) -> AppResult<()> {
    if size as usize > max_size {
        return Err(AppError::Validation(format!(
            "File size {} bytes exceeds maximum {} bytes",
            size, max_size
        )));
    }
    Ok(())
}

/// Validate MIME type
///
/// # Arguments
///
/// * `mime_type` - File MIME type
///
/// # Returns
///
/// Ok if valid, Err otherwise
///
/// # Errors
///
/// Returns validation error if MIME type not allowed
pub fn validate_mime_type(mime_type: &str) -> AppResult<()> {
    const ALLOWED_TYPES: &[&str] = &[
        "application/pdf",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "application/msword",
        "text/plain",
        "text/csv",
        "application/json",
    ];

    if !ALLOWED_TYPES.contains(&mime_type) {
        return Err(AppError::Validation(format!(
            "File type '{}' not allowed. Allowed types: PDF, DOCX, DOC, TXT, CSV, JSON",
            mime_type
        )));
    }
    Ok(())
}

/// Generate secure file path
///
/// # Arguments
///
/// * `upload_dir` - Base upload directory
/// * `original_filename` - Original filename
///
/// # Returns
///
/// Tuple of (stored_filename, full_path)
///
/// # Errors
///
/// Returns error if path operations fail
pub fn generate_file_path(upload_dir: &str, original_filename: &str) -> AppResult<(String, PathBuf)> {
    // Create upload directory if it doesn't exist
    std::fs::create_dir_all(upload_dir)
        .map_err(|e| AppError::Internal(format!("Failed to create upload directory: {}", e)))?;

    // Extract extension
    let extension = Path::new(original_filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");

    // Generate unique filename
    let stored_filename = format!("{}. {}", Uuid::new_v4(), extension);
    let full_path = Path::new(upload_dir).join(&stored_filename);

    Ok((stored_filename, full_path))
}

/// Save file bytes to disk
///
/// # Arguments
///
/// * `file_path` - Path to save file
/// * `bytes` - File contents
///
/// # Returns
///
/// Number of bytes written
///
/// # Errors
///
/// Returns error if file write fails
pub async fn save_file(file_path: &Path, bytes: &[u8]) -> AppResult<u64> {
    tokio::fs::write(file_path, bytes)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to save file: {}", e)))?;
    
    Ok(bytes.len() as u64)
}

/// Delete file from disk
///
/// # Arguments
///
/// * `file_path` - Path to file to delete
///
/// # Returns
///
/// Ok if successful
///
/// # Errors
///
/// Returns error if deletion fails
pub async fn delete_file(file_path: &Path) -> AppResult<()> {
    if file_path.exists() {
        tokio::fs::remove_file(file_path)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to delete file: {}", e)))?;
    }
    Ok(())
}
