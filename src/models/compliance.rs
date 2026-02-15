use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Risk levels for compliance items
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum RiskLevel {
    #[serde(rename = "low")]
    Low,
    
    #[serde(rename = "medium")]
    Medium,
    
    #[serde(rename = "high")]
    High,
    
    #[serde(rename = "critical")]
    Critical,
}

impl RiskLevel {
    /// Convert RiskLevel to database string
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Low => "low",
            RiskLevel::Medium => "medium",
            RiskLevel::High => "high",
            RiskLevel::Critical => "critical",
        }
    }
}

/// Status of compliance item
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum ComplianceStatus {
    #[serde(rename = "pending")]
    Pending,
    
    #[serde(rename = "in_progress")]
    InProgress,
    
    #[serde(rename = "completed")]
    Completed,
    
    #[serde(rename = "expired")]
    Expired,
}

impl ComplianceStatus {
    /// Convert ComplianceStatus to database string
    pub fn as_str(&self) -> &'static str {
        match self {
            ComplianceStatus::Pending => "pending",
            ComplianceStatus::InProgress => "in_progress",
            ComplianceStatus::Completed => "completed",
            ComplianceStatus::Expired => "expired",
        }
    }
}

/// Compliance item model from database
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ComplianceItem {
    /// Unique identifier
    pub id: Uuid,
    
    /// User who owns this compliance item
    pub user_id: Uuid,
    
    /// Title/name of the compliance requirement
    pub title: String,
    
    /// Detailed description
    pub description: Option<String>,
    
    /// Risk level
    #[sqlx(try_from = "String")]
    pub risk_level: String,
    
    /// Current status
    #[sqlx(try_from = "String")]
    pub status: String,
    
    /// Due date for completion
    pub due_date: Option<DateTime<Utc>>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// DTO for creating complian ce items
#[derive(Debug, Deserialize, Validate)]
pub struct CreateComplianceDto {
    /// Title (required, min 3 chars)
    #[validate(length(min = 3, message = "Title must be at least 3 characters"))]
    pub title: String,
    
    /// Description (optional)
    pub description: Option<String>,
    
    /// Risk level (required)
    pub risk_level: String,
    
    /// Status (required)
    pub status: String,
    
    /// Due date (optional)
    pub due_date: Option<DateTime<Utc>>,
}

/// DTO for updating compliance items
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateComplianceDto {
    /// Title (optional)
    #[validate(length(min = 3, message = "Title must be at least 3 characters"))]
    pub title: Option<String>,
    
    /// Description (optional)
    pub description: Option<String>,
    
    /// Risk level (optional)
    pub risk_level: Option<String>,
    
    /// Status (optional)
    pub status: Option<String>,
    
    /// Due date (optional)
    pub due_date: Option<DateTime<Utc>>,
}
