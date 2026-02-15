use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Risk score model from database
///
/// Represents a risk assessment for a compliance item
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct RiskScore {
    /// Unique identifier
    pub id: Uuid,
    
    /// Related compliance item
    pub compliance_item_id: Uuid,
    
    /// Related document (optional)
    pub document_id: Option<Uuid>,
    
    /// User who created this assessment
    pub user_id: Uuid,
    
    /// Risk category (e.g., "Data Privacy", "Security")
    pub risk_category: String,
    
    /// Numeric risk score (0-100)
    pub risk_score: i32,
    
    /// Risk level classification
    pub risk_level: String,
    
    /// Assessment date
    pub assessment_date: DateTime<Utc>,
    
    /// Who performed the assessment
    pub assessed_by: Option<String>,
    
    /// Additional notes
    pub notes: Option<String>,
    
    /// AI confidence score (0.0-1.0)
    pub ai_confidence: Option<f32>,
    
    /// AI reasoning/explanation
    pub ai_reasoning: Option<String>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// DTO for creating risk assessment
#[derive(Debug, Deserialize, Validate)]
pub struct CreateRiskScoreDto {
    /// Compliance item ID
    #[validate(length(min = 1, message = "Compliance item ID is required"))]
    pub compliance_item_id: String,
    
    /// Document ID (optional)
    pub document_id: Option<String>,
    
    /// Risk category
    #[validate(length(min = 1, max = 100, message = "Risk category must be 1-100 characters"))]
    pub risk_category: String,
    
    /// Risk score (0-100)
    #[validate(range(min = 0, max = 100, message = "Risk score must be 0-100"))]
    pub risk_score: i32,
    
    /// Risk level
    #[validate(custom(function = "validate_risk_level"))]
    pub risk_level: String,
    
    /// Who assessed
    pub assessed_by: Option<String>,
    
    /// Notes
    pub notes: Option<String>,
    
    /// AI confidence
    pub ai_confidence: Option<f32>,
    
    /// AI reasoning
    pub ai_reasoning: Option<String>,
}

/// Validate risk level enum
fn validate_risk_level(level: &str) -> Result<(), validator::ValidationError> {
    match level {
        "low" | "medium" | "high" | "critical" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_risk_level")),
    }
}

/// DTO for updating risk assessment
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateRiskScoreDto {
    /// Risk category
    #[validate(length(min = 1, max = 100))]
    pub risk_category: Option<String>,
    
    /// Risk score
    #[validate(range(min = 0, max = 100))]
    pub risk_score: Option<i32>,
    
    /// Risk level (validated if present)
    pub risk_level: Option<String>,
    
    /// Notes
    pub notes: Option<String>,
    
    /// AI confidence
    pub ai_confidence: Option<f32>,
    
    /// AI reasoning
    pub ai_reasoning: Option<String>,
}

impl UpdateRiskScoreDto {
    /// Validate risk level if present
    ///
    /// # Returns
    ///
    /// Ok if valid or None, Err otherwise
    pub fn validate_risk_level(&self) -> Result<(), validator::ValidationError> {
        if let Some(ref level) = self.risk_level {
            validate_risk_level(level)?;
        }
        Ok(())
    }
}
