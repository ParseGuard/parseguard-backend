use serde::Serialize;
use sqlx::PgPool;
use tracing::{info, instrument};
use uuid::Uuid;

use crate::{db::repository::DashboardRepository, error::AppResult};

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
    /// Dashboard repository
    repository: DashboardRepository,
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
        info!("📊 DashboardService started");
        Self { 
            repository: DashboardRepository::new(pool) 
        }
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
    #[instrument(skip(self))]
    pub async fn get_stats(&self, user_id: Uuid) -> AppResult<DashboardStats> {
        info!("Fetching stats for user: {}", user_id);
        
        let compliance_stats = self.repository.get_compliance_stats(user_id).await?;
        let document_stats = self.repository.get_document_stats(user_id).await?;

        // Calculate compliance score (percentage of completed items)
        let compliance_score = if compliance_stats.total > 0 {
            (compliance_stats.completed as f64 / compliance_stats.total as f64) * 100.0
        } else {
            0.0
        };

        Ok(DashboardStats {
            total_compliance_items: compliance_stats.total,
            pending_items: compliance_stats.pending,
            in_progress_items: compliance_stats.in_progress,
            completed_items: compliance_stats.completed,
            expired_items: compliance_stats.expired,
            total_documents: document_stats.total,
            analyzed_documents: document_stats.analyzed,
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
    #[instrument(skip(self))]
    pub async fn get_recent_activity(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> AppResult<Vec<ActivityItem>> {
        info!("Fetching recent activity for user: {} (limit: {})", user_id, limit);
        
        let activities = self.repository.get_recent_activity(user_id, limit).await?;

        Ok(activities
            .into_iter()
            .filter_map(|item| {
                if let (Some(id), Some(activity_type), Some(title), Some(timestamp)) = 
                    (item.id, item.activity_type, item.title, item.timestamp) 
                {
                    Some(ActivityItem {
                        id,
                        activity_type,
                        title,
                        timestamp,
                    })
                } else {
                    None
                }
            })
            .collect())
    }
}
