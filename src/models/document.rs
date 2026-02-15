use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Document model from database
///
/// Represents an uploaded document with AI analysis
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Document {
    /// Unique identifier
    pub id: Uuid,
    
    /// User who uploaded the document
    pub user_id: Uuid,
    
    /// Original filename
    pub filename: String,
    
    /// File path on server
    pub file_path: String,
    
    /// File size in bytes
    pub file_size: i64,
    
    /// MIME type
    pub mime_type: String,
    
    /// Extracted text from document
    pub extracted_text: Option<String>,
    
    /// AI analysis results (JSON)
    pub ai_analysis: Option<sqlx::types::Json<serde_json::Value>>,
    
    /// Upload timestamp
    pub uploaded_at: DateTime<Utc>,
}

/// DTO for creating document records
///
/// Used when registering a new uploaded document
#[derive(Debug, Deserialize, Validate)]
pub struct CreateDocumentDto {
    /// Original filename
    #[validate(length(min = 1, message = "Filename is required"))]
    pub filename: String,
    
    /// File path on server
    #[validate(length(min = 1, message = "File path is required"))]
    pub file_path: String,
    
    /// File size in bytes
    #[validate(range(min = 1, message = "File size must be positive"))]
    pub file_size: i64,
    
    /// MIME type
    #[validate(length(min = 1, message = "MIME type is required"))]
    pub mime_type: String,
    
    /// Extracted text (optional, can be added later)
    pub extracted_text: Option<String>,
}

/// DTO for updating document analysis
///
/// Used to add AI analysis results after processing
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateDocumentDto {
    /// Extracted text from document
    pub extracted_text: Option<String>,
    
    /// AI analysis results (JSON)
    pub ai_analysis: Option<sqlx::types::Json<serde_json::Value>>,
}

/// Document response with metadata
///
/// Used in API responses
#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    /// Document ID
    pub id: Uuid,
    
    /// Filename
    pub filename: String,
    
    /// File size in bytes
    pub file_size: i64,
    
    /// MIME type
    pub mime_type: String,
    
    /// Whether text has been extracted
    pub has_extracted_text: bool,
    
    /// Whether AI analysis is available
    pub has_ai_analysis: bool,
    
    /// Upload timestamp
    pub uploaded_at: DateTime<Utc>,
}

impl From<Document> for DocumentResponse {
    /// Convert Document to DocumentResponse
    ///
    /// # Arguments
    ///
    /// * `doc` - Document model from database
    ///
    /// # Returns
    ///
    /// Safe document representation for API responses
    fn from(doc: Document) -> Self {
        Self {
            id: doc.id,
            filename: doc.filename,
            file_size: doc.file_size,
            mime_type: doc.mime_type,
            has_extracted_text: doc.extracted_text.is_some(),
            has_ai_analysis: doc.ai_analysis.is_some(),
            uploaded_at: doc.uploaded_at,
        }
    }
}
