use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;

/// Dashboard statistics
///
/// Contains key metrics for the dashboard view
#[derive(Debug, Serialize)]
pub struct DashboardStats {
    /// Total number of compliance items
    pub total_compliance_items: i64,
    
    /// Number of pending compliance items
    pub pending_items: i64,
    
    /// Number of in-progress compliance items
    pub in_progress_items: i64,
    
    /// Number of completed compliance items
    pub completed_items: i64,
    
    /// Number of expired compliance items
    pub expired_items: i64,
    
    /// Total number of documents
    pub total_documents: i64,
    
    /// Number of documents with AI analysis
    pub analyzed_documents: i64,
    
    /// Average compliance score (0-100)
    pub compliance_score: f64,
}

/// Recent activity item
#[derive(Debug, Serialize)]
pub struct ActivityItem {
    /// Activity ID
    pub id: Uuid,
    
    /// Activity type (e.g., "compliance_created", "document_uploaded")
    pub activity_type: String,
    
    /// Title/description of the activity
    pub title: String,
    
    /// Timestamp of the activity
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Dashboard service for aggregating statistics
///
/// Provides dashboard data and analytics
pub struct DashboardService {
    /// Database connection pool
    pool: PgPool,
}

impl DashboardService {
    /// Create a new DashboardService
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    ///
    /// # Returns
    ///
    /// New DashboardService instance
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get dashboard statistics for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - User UUID
    ///
    /// # Returns
    ///
    /// Dashboard statistics
    ///
    /// # Errors
    ///
    /// Returns database error if queries fail
    pub async fn get_stats(&self, user_id: Uuid) -> AppResult<DashboardStats> {
        // Get compliance item counts
        let compliance_stats: (i64, i64, i64, i64, i64) = sqlx::query_as(
            "SELECT 
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'pending') as pending,
                COUNT(*) FILTER (WHERE status = 'in_progress') as in_progress,
                COUNT(*) FILTER (WHERE status = 'completed') as completed,
                COUNT(*) FILTER (WHERE status = 'expired') as expired
             FROM compliance_items
             WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        // Get document counts
        let document_stats: (i64, i64) = sqlx::query_as(
            "SELECT 
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE ai_analysis IS NOT NULL) as analyzed
             FROM documents
             WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        // Calculate compliance score (percentage of completed items)
        let compliance_score = if compliance_stats.0 > 0 {
            (compliance_stats.3 as f64 / compliance_stats.0 as f64) * 100.0
        } else {
            0.0
        };

        Ok(DashboardStats {
            total_compliance_items: compliance_stats.0,
            pending_items: compliance_stats.1,
            in_progress_items: compliance_stats.2,
            completed_items: compliance_stats.3,
            expired_items: compliance_stats.4,
            total_documents: document_stats.0,
            analyzed_documents: document_stats.1,
            compliance_score,
        })
    }

    /// Get recent activity for a user
    ///
    /// # Arguments
    ///
    /// * `user_id` - User UUID
    /// * `limit` - Maximum number of items to return
    ///
    /// # Returns
    ///
    /// List of recent activity items
    ///
    /// # Errors
    ///
    /// Returns database error if queries fail
    pub async fn get_recent_activity(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> AppResult<Vec<ActivityItem>> {
        // Combine compliance and document activities
        let activities = sqlx::query_as::<_, (Uuid, String, String, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, 'compliance_created' as activity_type, title, created_at as timestamp
             FROM compliance_items
             WHERE user_id = $1
             UNION ALL
             SELECT id, 'document_uploaded' as activity_type, filename as title, uploaded_at as timestamp
             FROM documents
             WHERE user_id = $1
             ORDER BY timestamp DESC
             LIMIT $2"
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(activities
            .into_iter()
            .map(|(id, activity_type, title, timestamp)| ActivityItem {
                id,
                activity_type,
                title,
                timestamp,
            })
            .collect())
    }
}
