use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};

use crate::error::{AppError, AppResult};

/// OLLAMA AI Service for document analysis
pub struct AiService {
    /// HTTP client
    client: Client,
    
    /// OLLAMA API base URL
    base_url: String,
}

/// AI analysis request
#[derive(Debug, Serialize)]
struct AnalysisRequest {
    /// Model name (default: llama2)
    model: String,
    
    /// Prompt for analysis
    prompt: String,
    
    /// Stream response (false for single response)
    stream: bool,
}

/// AI analysis response
#[derive(Debug, Deserialize)]
struct AnalysisResponse {
    /// Generated response text
    response: String,
    
    /// Whether generation is complete
    done: bool,
}

/// Document analysis result
#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentAnalysis {
    /// Summary of document
    pub summary: String,
    
    /// Detected compliance topics
    pub compliance_topics: Vec<String>,
    
    /// Risk indicators found
    pub risk_indicators: Vec<String>,
    
    /// Suggested compliance items
    pub suggested_items: Vec<SuggestedComplianceItem>,
    
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
}

/// Suggested compliance item from AI
#[derive(Debug, Serialize, Deserialize)]
pub struct SuggestedComplianceItem {
    /// Suggested title
    pub title: String,
    
    /// Description
    pub description: String,
    
    /// Suggested risk level
    pub risk_level: String,
    
    /// Confidence in suggestion (0.0-1.0)
    pub confidence: f32,
}

impl AiService {
    /// Create new AI service
    ///
    /// # Arguments
    ///
    /// * `ollama_url` - OLLAMA API base URL
    ///
    /// # Returns
    ///
    /// AI service instance
    pub fn new(ollama_url: String) -> Self {
        info!("🤖 AiService started");
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            base_url: ollama_url,
        }
    }

    /// Analyze document text with AI
    ///
    /// # Arguments
    ///
    /// * `text` - Document text to analyze
    ///
    /// # Returns
    ///
    /// Analysis results
    ///
    /// # Errors
    ///
    /// Returns error if AI request fails
    #[instrument(skip(self, text))]
    pub async fn analyze_document(&self, text: &str) -> AppResult<DocumentAnalysis> {
        info!("Analyzing document (length: {})", text.len());
        let prompt = self.create_analysis_prompt(text);

        let response = self.generate(&prompt, "llama2").await?;

        // Parse AI response into structured format
        self.parse_analysis_response(&response)
    }

    /// Generate risk assessment for compliance item
    ///
    /// # Arguments
    ///
    /// * `title` - Compliance item title
    /// * `description` - Compliance item description
    ///
    /// # Returns
    ///
    /// Risk assessment with score and reasoning
    ///
    /// # Errors
    ///
    /// Returns error if AI request fails
    #[instrument(skip(self))]
    pub async fn assess_risk(
        &self,
        title: &str,
        description: Option<&str>,
    ) -> AppResult<(i32, String, f32)> {
        info!("Assessing risk for: {}", title);
        let prompt = format!(
            "Analyze the following compliance item and provide a risk assessment:\n\
             Title: {}\n\
             Description: {}\n\n\
             Provide:\n\
             1. Risk score (0-100)\n\
             2. Risk level (low/medium/high/critical)\n\
             3. Brief reasoning\n\n\
             Format your response as:\n\
             SCORE: <number>\n\
             LEVEL: <level>\n\
             REASONING: <explanation>",
            title,
            description.unwrap_or("N/A")
        );

        let response = self.generate(&prompt, "llama2").await?;

        self.parse_risk_response(&response)
    }

    /// Generate text with OLLAMA
    ///
    /// # Arguments
    ///
    /// * `prompt` - Input prompt
    /// * `model` - Model name
    ///
    /// # Returns
    ///
    /// Generated text
    ///
    /// # Errors
    ///
    /// Returns error if request fails
    #[instrument(skip(self, prompt))]
    async fn generate(&self, prompt: &str, model: &str) -> AppResult<String> {
        info!("Generating with model: {}", model);
        let url = format!("{}/api/generate", self.base_url);

        let request = AnalysisRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::Ollama(format!("Failed to connect to OLLAMA: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Ollama(format!(
                "OLLAMA request failed with status: {}",
                response.status()
            )));
        }

        let result: AnalysisResponse = response
            .json()
            .await
            .map_err(|e| AppError::Ollama(format!("Failed to parse OLLAMA response: {}", e)))?;

        Ok(result.response)
    }

    /// Create analysis prompt
    fn create_analysis_prompt(&self, text: &str) -> String {
        // Truncate text if too long (max 4000 chars)
        let truncated_text = if text.len() > 4000 {
            &text[..4000]
        } else {
            text
        };

        format!(
            "Analyze the following document for compliance and risk management:\n\n\
             {}\n\n\
             Provide:\n\
             1. A brief summary (2-3 sentences)\n\
             2. List of compliance topics mentioned\n\
             3. Risk indicators or concerns\n\
             4. Suggested compliance items to track\n\n\
             Format your response clearly with labeled sections.",
            truncated_text
        )
    }

    /// Parse analysis response
    fn parse_analysis_response(&self, response: &str) -> AppResult<DocumentAnalysis> {
        // Simple parsing (in production, use more robust parsing)
        Ok(DocumentAnalysis {
            summary: self.extract_section(response, "summary").unwrap_or_else(|| {
                response.chars().take(200).collect::<String>() + "..."
            }),
            compliance_topics: self.extract_list(response, "compliance topics"),
            risk_indicators: self.extract_list(response, "risk"),
            suggested_items: self.extract_suggested_items(response),
            confidence: 0.7, // Default confidence
        })
    }

    /// Extract suggested items from response
    fn extract_suggested_items(&self, text: &str) -> Vec<SuggestedComplianceItem> {
        let items_section = self.extract_list(text, "suggested");
        
        items_section.into_iter().map(|item| {
            // Simple parsing to split title/desc if possible, otherwise use whole line as title
            let parts: Vec<&str> = item.splitn(2, ':').collect();
            let title = parts[0].trim().to_string();
            let description = parts.get(1).map(|s| s.trim().to_string()).unwrap_or_else(|| "No description provided".to_string());
            
            SuggestedComplianceItem {
                title,
                description,
                risk_level: "medium".to_string(), // Default risk
                confidence: 0.8,
            }
        }).collect()
    }

    fn parse_risk_response(&self, response: &str) -> AppResult<(i32, String, f32)> {
        let score = self.extract_number(response, "SCORE:").unwrap_or(50);
        let level = self
            .extract_value(response, "LEVEL:")
            .unwrap_or_else(|| "medium".to_string());
        let _reasoning = self
            .extract_section(response, "REASONING:")
            .unwrap_or_else(|| response.to_string());

        Ok((score, level, 0.7))
    }

    /// Extract section from response
    fn extract_section(&self, text: &str, marker: &str) -> Option<String> {
        text.to_lowercase()
            .split(marker)
            .nth(1)?
            .split('\n')
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
            .into()
    }

    /// Extract list from response
    fn extract_list(&self, text: &str, marker: &str) -> Vec<String> {
        text.to_lowercase()
            .split(marker)
            .nth(1)
            .map(|section| {
                section
                    .lines()
                    .take(10) // Increase lookahead
                    .filter(|line| {
                        let trimmed = line.trim();
                        !trimmed.is_empty() && (
                            trimmed.starts_with('-') || 
                            trimmed.starts_with('*') || 
                            trimmed.chars().next().map_or(false, |c| c.is_numeric()) // Allow numbered lists
                        )
                    })
                    .map(|line| {
                        // Remove bullets or numbers (e.g., "1. " or "- ")
                        line.trim()
                            .trim_start_matches(|c: char| c == '-' || c == '*' || c.is_numeric() || c == '.')
                            .trim()
                            .to_string()
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Extract value after marker
    fn extract_value(&self, text: &str, marker: &str) -> Option<String> {
        text.lines()
            .find(|line| line.contains(marker))?
            .split(marker)
            .nth(1)?
            .trim()
            .to_string()
            .into()
    }

    /// Extract number from text
    fn extract_number(&self, text: &str, marker: &str) -> Option<i32> {
        self.extract_value(text, marker)?
            .chars()
            .filter(|c| c.is_numeric())
            .collect::<String>()
            .parse()
            .ok()
    }
}
