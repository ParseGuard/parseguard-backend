use axum::{
    extract::{State, Json},
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use crate::{
    error::AppResult,
    services::ai_service::AiService,
    AppState,
};

/// Request dto for document analysis
#[derive(Deserialize)]
pub struct AnalyzeDocumentDto {
    pub text: String,
}

/// Request dto for risk assessment
#[derive(Deserialize)]
pub struct AssessRiskDto {
    pub title: String,
    pub description: Option<String>,
}

/// Response for risk assessment
#[derive(Serialize)]
pub struct RiskAssessmentResponse {
    pub score: i32,
    pub level: String,
    pub confidence: f32,
}

/// Analyze document content
pub async fn analyze_document(
    State(state): State<AppState>,
    Json(dto): Json<AnalyzeDocumentDto>,
) -> AppResult<impl IntoResponse> {
    let ai_service = AiService::new(state.config.ollama_url.clone());
    let analysis = ai_service.analyze_document(&dto.text).await?;
    
    Ok((StatusCode::OK, Json(analysis)))
}

/// Assess compliance risk
pub async fn assess_risk(
    State(state): State<AppState>,
    Json(dto): Json<AssessRiskDto>,
) -> AppResult<impl IntoResponse> {
    let ai_service = AiService::new(state.config.ollama_url.clone());
    let (score, level, confidence) = ai_service.assess_risk(&dto.title, dto.description.as_deref()).await?;
    
    let response = RiskAssessmentResponse {
        score,
        level,
        confidence,
    };
    
    Ok((StatusCode::OK, Json(response)))
}
