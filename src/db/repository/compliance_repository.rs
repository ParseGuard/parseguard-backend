use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppResult,
    models::{ComplianceItem, CreateComplianceDto, UpdateComplianceDto},
};

/// Compliance repository for database operations
///
/// Handles all compliance-related database queries
pub struct ComplianceRepository {
    /// Database connection pool
    pool: PgPool,
}

impl ComplianceRepository {
    /// Create a new ComplianceRepository
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    ///
    /// New ComplianceRepository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find all compliance items for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - User UUID
    ///
    /// # Returns
    ///
    /// List of compliance items
    ///
    /// # Errors
    ///
    /// Returns database error if query fails
    pub async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<ComplianceItem>> {
        let items = sqlx::query_as::<_, ComplianceItem>(
            "SELECT id, user_id, title, description, risk_level, status, due_date, created_at, updated_at
             FROM compliance_items
             WHERE user_id = $1
             ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }

    /// Find compliance item by ID
    ///
    /// # Arguments
    ///
    /// * `id` - Compliance item UUID
    /// * `user_id` - User UUID (for authorization)
    ///
    /// # Returns
    ///
    /// Optional ComplianceItem if found and owned by user
    ///
    /// # Errors
    ///
    /// Returns database error if query fails
    pub async fn find_by_id(&self, id: Uuid, user_id: Uuid) -> AppResult<Option<ComplianceItem>> {
        let item = sqlx::query_as::<_, ComplianceItem>(
            "SELECT id, user_id, title, description, risk_level, status, due_date, created_at, updated_at
             FROM compliance_items
             WHERE id = $1 AND user_id = $2"
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(item)
    }

    /// Create a new compliance item
    ///
    /// # Arguments
    ///
    /// * `user_id` - User UUID who owns this item
    /// * `dto` - Compliance item creation data
    ///
    /// # Returns
    ///
    /// Created ComplianceItem
    ///
    /// # Errors
    ///
    /// Returns database error if insertion fails
    pub async fn create(&self, user_id: Uuid, dto: &CreateComplianceDto) -> AppResult<ComplianceItem> {
        let item = sqlx::query_as::<_, ComplianceItem>(
            "INSERT INTO compliance_items (user_id, title, description, risk_level, status, due_date)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, user_id, title, description, risk_level, status, due_date, created_at, updated_at"
        )
        .bind(user_id)
        .bind(&dto.title)
        .bind(&dto.description)
        .bind(&dto.risk_level)
        .bind(&dto.status)
        .bind(dto.due_date)
        .fetch_one(&self.pool)
        .await?;

        Ok(item)
    }

    /// Update a compliance item
    ///
    /// # Arguments
    ///
    /// * `id` - Compliance item UUID
    /// * `user_id` - User UUID (for authorization)
    /// * `dto` - Update data
    ///
    /// # Returns
    ///
    /// Updated ComplianceItem or None if not found/unauthorized
    ///
    /// # Errors
    ///
    /// Returns database error if update fails
    pub async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        dto: &UpdateComplianceDto,
    ) -> AppResult<Option<ComplianceItem>> {
        // Build dynamic query based on what fields are provided
        let mut query = String::from("UPDATE compliance_items SET ");
        let mut updates = Vec::new();
        let mut param_count = 3; // Starting from $3 since $1 is id, $2 is user_id

        if dto.title.is_some() {
            updates.push(format!("title = ${}", param_count));
            param_count += 1;
        }
        if dto.description.is_some() {
            updates.push(format!("description = ${}", param_count));
            param_count += 1;
        }
        if dto.risk_level.is_some() {
            updates.push(format!("risk_level = ${}", param_count));
            param_count += 1;
        }
        if dto.status.is_some() {
            updates.push(format!("status = ${}", param_count));
            param_count += 1;
        }
        if dto.due_date.is_some() {
            updates.push(format!("due_date = ${}", param_count));
            param_count += 1;
        }

        if updates.is_empty() {
            // No updates provided, just return existing item
            return self.find_by_id(id, user_id).await;
        }

        updates.push("updated_at = NOW()".to_string());
        query.push_str(&updates.join(", "));
        query.push_str(" WHERE id = $1 AND user_id = $2 RETURNING id, user_id, title, description, risk_level, status, due_date, created_at, updated_at");

        let mut query_builder = sqlx::query_as::<_, ComplianceItem>(&query)
            .bind(id)
            .bind(user_id);

        if let Some(ref title) = dto.title {
            query_builder = query_builder.bind(title);
        }
        if let Some(ref description) = dto.description {
            query_builder = query_builder.bind(description);
        }
        if let Some(ref risk_level) = dto.risk_level {
            query_builder = query_builder.bind(risk_level);
        }
        if let Some(ref status) = dto.status {
            query_builder = query_builder.bind(status);
        }
        if let Some(due_date) = dto.due_date {
            query_builder = query_builder.bind(due_date);
        }

        let item = query_builder.fetch_optional(&self.pool).await?;

        Ok(item)
    }

    /// Delete a compliance item
    ///
    /// # Arguments
    ///
    /// * `id` - Compliance item UUID
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
            "DELETE FROM compliance_items WHERE id = $1 AND user_id = $2"
        )
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
