use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppResult,
    models::{CreateRiskScoreDto, RiskScore, UpdateRiskScoreDto},
};

/// Repository for risk score database operations
pub struct RiskScoreRepository {
    pool: PgPool,
}

impl RiskScoreRepository {
    /// Create new repository
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    ///
    /// Repository instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find all risk scores for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - User UUID
    ///
    /// # Returns
    ///
    /// List of risk scores
    ///
    /// # Errors
    ///
    /// Returns database error if query fails
    pub async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<RiskScore>> {
        let scores = sqlx::query_as::<_, RiskScore>(
            "SELECT id, compliance_item_id, document_id, user_id, risk_category,
                    risk_score, risk_level, assessment_date, assessed_by, notes,
                    ai_confidence, ai_reasoning, created_at, updated_at
             FROM risk_scores
             WHERE user_id = $1
             ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(scores)
    }

    /// Find risk scores by compliance item
    ///
    /// # Arguments
    ///
    /// * `compliance_item_id` - Compliance item UUID
    /// * `user_id` - User UUID for authorization
    ///
    /// # Returns
    ///
    /// List of risk scores for the compliance item
    ///
    /// # Errors
    ///
    /// Returns database error if query fails
    pub async fn find_by_compliance_item(
        &self,
        compliance_item_id: Uuid,
        user_id: Uuid,
    ) -> AppResult<Vec<RiskScore>> {
        let scores = sqlx::query_as::<_, RiskScore>(
            "SELECT id, compliance_item_id, document_id, user_id, risk_category,
                    risk_score, risk_level, assessment_date, assessed_by, notes,
                    ai_confidence, ai_reasoning, created_at, updated_at
             FROM risk_scores
             WHERE compliance_item_id = $1 AND user_id = $2
             ORDER BY assessment_date DESC"
        )
        .bind(compliance_item_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(scores)
    }

    /// Find risk score by ID
    ///
    /// # Arguments
    ///
    /// * `id` - Risk score UUID
    /// * `user_id` - User UUID for authorization
    ///
    /// # Returns
    ///
    /// Risk score if found and authorized
    ///
    /// # Errors
    ///
    /// Returns database error if query fails
    pub async fn find_by_id(&self, id: Uuid, user_id: Uuid) -> AppResult<Option<RiskScore>> {
        let score = sqlx::query_as::<_, RiskScore>(
            "SELECT id, compliance_item_id, document_id, user_id, risk_category,
                    risk_score, risk_level, assessment_date, assessed_by, notes,
                    ai_confidence, ai_reasoning, created_at, updated_at
             FROM risk_scores
             WHERE id = $1 AND user_id = $2"
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(score)
    }

    /// Create new risk score
    ///
    /// # Arguments
    ///
    /// * `user_id` - User UUID
    /// * `dto` - Risk score data
    ///
    /// # Returns
    ///
    /// Created risk score
    ///
    /// # Errors
    ///
    /// Returns database error if insert fails
    pub async fn create(&self, user_id: Uuid, dto: &CreateRiskScoreDto) -> AppResult<RiskScore> {
        let compliance_item_id = Uuid::parse_str(&dto.compliance_item_id)
            .map_err(|_| crate::error::AppError::Validation("Invalid compliance item ID".to_string()))?;

        let document_id = dto.document_id
            .as_ref()
            .map(|id| Uuid::parse_str(id))
            .transpose()
            .map_err(|_| crate::error::AppError::Validation("Invalid document ID".to_string()))?;

        let score = sqlx::query_as::<_, RiskScore>(
            "INSERT INTO risk_scores 
                (user_id, compliance_item_id, document_id, risk_category, risk_score,
                 risk_level, assessed_by, notes, ai_confidence, ai_reasoning)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             RETURNING id, compliance_item_id, document_id, user_id, risk_category,
                       risk_score, risk_level, assessment_date, assessed_by, notes,
                       ai_confidence, ai_reasoning, created_at, updated_at"
        )
        .bind(user_id)
        .bind(compliance_item_id)
        .bind(document_id)
        .bind(&dto.risk_category)
        .bind(dto.risk_score)
        .bind(&dto.risk_level)
        .bind(&dto.assessed_by)
        .bind(&dto.notes)
        .bind(dto.ai_confidence)
        .bind(&dto.ai_reasoning)
        .fetch_one(&self.pool)
        .await?;

        Ok(score)
    }

    /// Update risk score
    ///
    /// # Arguments
    ///
    /// * `id` - Risk score UUID
    /// * `user_id` - User UUID for authorization
    /// * `dto` - Update data
    ///
    /// # Returns
    ///
    /// Updated risk score
    ///
    /// # Errors
    ///
    /// Returns database error if update fails
    pub async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        dto: &UpdateRiskScoreDto,
    ) -> AppResult<Option<RiskScore>> {
        let score = sqlx::query_as::<_, RiskScore>(
            "UPDATE risk_scores
             SET risk_category = COALESCE($3, risk_category),
                 risk_score = COALESCE($4, risk_score),
                 risk_level = COALESCE($5, risk_level),
                 notes = COALESCE($6, notes),
                 ai_confidence = COALESCE($7, ai_confidence),
                 ai_reasoning = COALESCE($8, ai_reasoning)
             WHERE id = $1 AND user_id = $2
             RETURNING id, compliance_item_id, document_id, user_id, risk_category,
                       risk_score, risk_level, assessment_date, assessed_by, notes,
                       ai_confidence, ai_reasoning, created_at, updated_at"
        )
        .bind(id)
        .bind(user_id)
        .bind(&dto.risk_category)
        .bind(dto.risk_score)
        .bind(&dto.risk_level)
        .bind(&dto.notes)
        .bind(dto.ai_confidence)
        .bind(&dto.ai_reasoning)
        .fetch_optional(&self.pool)
        .await?;

        Ok(score)
    }

    /// Delete risk score
    ///
    /// # Arguments
    ///
    /// * `id` - Risk score UUID
    /// * `user_id` - User UUID for authorization
    ///
    /// # Returns
    ///
    /// True if deleted, false if not found
    ///
    /// # Errors
    ///
    /// Returns database error if delete fails
    pub async fn delete(&self, id: Uuid, user_id: Uuid) -> AppResult<bool> {
        let result = sqlx::query(
            "DELETE FROM risk_scores WHERE id = $1 AND user_id = $2"
        )
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
