use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::error::AppResult;

/// Dashboard statistics
#[derive(Debug, sqlx::FromRow)]
pub struct DashboardStatsQuery {
    pub total: i64,
    pub pending: i64,
    pub in_progress: i64,
    pub completed: i64,
    pub expired: i64,
}

/// Document statistics
#[derive(Debug, sqlx::FromRow)]
pub struct DocumentStatsQuery {
    pub total: i64,
    pub analyzed: i64,
}

/// Activity item from DB
#[derive(Debug, sqlx::FromRow)]
pub struct ActivityItemQuery {
    pub id: Option<Uuid>,
    pub activity_type: Option<String>,
    pub title: Option<String>,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

/// Dashboard repository for database operations
pub struct DashboardRepository {
    pool: PgPool,
}

impl DashboardRepository {
    /// Create a new DashboardRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get compliance statistics
    #[instrument(skip(self))]
    pub async fn get_compliance_stats(&self, user_id: Uuid) -> AppResult<DashboardStatsQuery> {
        let stats = sqlx::query_as!(
            DashboardStatsQuery,
            r#"
            SELECT 
                COUNT(*) as "total!",
                COUNT(*) FILTER (WHERE status = 'pending') as "pending!",
                COUNT(*) FILTER (WHERE status = 'in_progress') as "in_progress!",
                COUNT(*) FILTER (WHERE status = 'completed') as "completed!",
                COUNT(*) FILTER (WHERE status = 'expired') as "expired!"
            FROM compliance_items
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(stats)
    }

    /// Get document statistics
    #[instrument(skip(self))]
    pub async fn get_document_stats(&self, user_id: Uuid) -> AppResult<DocumentStatsQuery> {
        let stats = sqlx::query_as!(
            DocumentStatsQuery,
            r#"
            SELECT 
                COUNT(*) as "total!",
                COUNT(*) FILTER (WHERE ai_analysis IS NOT NULL) as "analyzed!"
            FROM documents
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(stats)
    }

    /// Get recent activity
    #[instrument(skip(self))]
    pub async fn get_recent_activity(&self, user_id: Uuid, limit: i64) -> AppResult<Vec<ActivityItemQuery>> {
        let activities = sqlx::query_as!(
            ActivityItemQuery,
            r#"
            SELECT id, 'compliance_created' as "activity_type!", title, created_at as "timestamp!"
            FROM compliance_items
            WHERE user_id = $1
            UNION ALL
            SELECT id, 'document_uploaded' as "activity_type!", filename as title, uploaded_at as "timestamp!"
            FROM documents
            WHERE user_id = $1
            ORDER BY "timestamp!" DESC
            LIMIT $2
            "#,
            user_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(activities)
    }
}
