use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppResult,
    models::{CreateDocumentDto, Document, UpdateDocumentDto},
};

/// Document repository for database operations
///
/// Handles all document-related database queries
pub struct DocumentRepository {
    /// Database connection pool
    pool: PgPool,
}

impl DocumentRepository {
    /// Create a new DocumentRepository
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    ///
    /// New DocumentRepository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find all documents for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - User UUID
    ///
    /// # Returns
    ///
    /// List of documents
    ///
    /// # Errors
    ///
    /// Returns database error if query fails
    pub async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<Document>> {
        let documents = sqlx::query_as::<_, Document>(
            "SELECT id, user_id, filename, file_path, file_size, mime_type, 
                    extracted_text, ai_analysis, uploaded_at
             FROM documents
             WHERE user_id = $1
             ORDER BY uploaded_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(documents)
    }

    /// Find document by ID
    ///
    /// # Arguments
    ///
    /// * `id` - Document UUID
    /// * `user_id` - User UUID (for authorization)
    ///
    /// # Returns
    ///
    /// Optional Document if found and owned by user
    ///
    /// # Errors
    ///
    /// Returns database error if query fails
    pub async fn find_by_id(&self, id: Uuid, user_id: Uuid) -> AppResult<Option<Document>> {
        let document = sqlx::query_as::<_, Document>(
            "SELECT id, user_id, filename, file_path, file_size, mime_type, 
                    extracted_text, ai_analysis, uploaded_at
             FROM documents
             WHERE id = $1 AND user_id = $2"
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(document)
    }

    /// Create a new document record
    ///
    /// # Arguments
    ///
    /// * `user_id` - User UUID who uploaded this document
    /// * `dto` - Document creation data
    ///
    /// # Returns
    ///
    /// Created Document
    ///
    /// # Errors
    ///
    /// Returns database error if insertion fails
    pub async fn create(&self, user_id: Uuid, dto: &CreateDocumentDto) -> AppResult<Document> {
        let document = sqlx::query_as::<_, Document>(
            "INSERT INTO documents (user_id, filename, file_path, file_size, mime_type, extracted_text)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, user_id, filename, file_path, file_size, mime_type, 
                       extracted_text, ai_analysis, uploaded_at"
        )
        .bind(user_id)
        .bind(&dto.filename)
        .bind(&dto.file_path)
        .bind(dto.file_size as i32)
        .bind(&dto.mime_type)
        .bind(&dto.extracted_text)
        .fetch_one(&self.pool)
        .await?;

        Ok(document)
    }

    /// Update document with AI analysis
    ///
    /// # Arguments
    ///
    /// * `id` - Document UUID
    /// * `user_id` - User UUID (for authorization)
    /// * `dto` - Update data
    ///
    /// # Returns
    ///
    /// Updated Document or None if not found/unauthorized
    ///
    /// # Errors
    ///
    /// Returns database error if update fails
    pub async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        dto: &UpdateDocumentDto,
    ) -> AppResult<Option<Document>> {
        let document = sqlx::query_as::<_, Document>(
            "UPDATE documents 
             SET extracted_text = COALESCE($3, extracted_text),
                 ai_analysis = COALESCE($4, ai_analysis)
             WHERE id = $1 AND user_id = $2
             RETURNING id, user_id, filename, file_path, file_size, mime_type, 
                       extracted_text, ai_analysis, uploaded_at"
        )
        .bind(id)
        .bind(user_id)
        .bind(&dto.extracted_text)
        .bind(&dto.ai_analysis)
        .fetch_optional(&self.pool)
        .await?;

        Ok(document)
    }

    /// Delete a document
    ///
    /// # Arguments
    ///
    /// * `id` - Document UUID
    /// * `user_id` - User UUID (for authorization)
    ///
    /// # Returns
    ///
    /// true if deleted, false if not found/unauthorized
    ///
    /// # Errors
    ///
    /// Returns database error if deletion fails
    pub async fn delete(&self, id: Uuid, user_id: Uuid) -> AppResult<bool> {
        let result = sqlx::query(
            "DELETE FROM documents WHERE id = $1 AND user_id = $2"
        )
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
