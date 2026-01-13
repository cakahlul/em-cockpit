//! Gemini AI Client
//!
//! Provides spec analysis using Google's Gemini API.

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::integrations::traits::IntegrationError;

/// Gemini configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiConfig {
    pub model: String,
    #[serde(skip)]
    pub api_key: Option<String>,
    pub daily_token_limit: Option<u32>,
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self {
            model: "gemini-pro".to_string(),
            api_key: None,
            daily_token_limit: None,
        }
    }
}

impl GeminiConfig {
    pub fn new(model: &str) -> Self {
        Self {
            model: model.to_string(),
            ..Default::default()
        }
    }

    pub fn with_api_key(mut self, key: &str) -> Self {
        self.api_key = Some(key.to_string());
        self
    }
}

/// Result of spec analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecAnalysis {
    pub clarity_score: u8,
    pub justification: String,
    pub ambiguous_phrases: Vec<AmbiguousPhrase>,
    pub missing_scenarios: Vec<MissingScenario>,
    pub risks: Vec<Risk>,
}

/// An ambiguous phrase found in the spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbiguousPhrase {
    pub text: String,
    pub explanation: String,
    pub suggested_replacement: String,
}

/// A missing scenario/edge case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingScenario {
    pub scenario: String,
    pub impact: String,
}

/// An identified risk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub risk: String,
    pub mitigation: String,
}

/// Gemini API client
#[derive(Debug)]
pub struct GeminiClient {
    config: GeminiConfig,
    http_client: Client,
}

impl GeminiClient {
    pub fn new(config: GeminiConfig) -> Result<Self, IntegrationError> {
        if config.api_key.is_none() {
            return Err(IntegrationError::ConfigError(
                "Gemini API key is required".to_string(),
            ));
        }

        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| IntegrationError::Network(e.to_string()))?;

        Ok(Self { config, http_client })
    }

    /// Analyze a spec/PRD for clarity and completeness
    pub async fn analyze_spec(&self, content: &str) -> Result<SpecAnalysis, IntegrationError> {
        let prompt = self.build_analysis_prompt(content);
        let response = self.generate_content(&prompt).await?;
        self.parse_analysis(&response)
    }

    /// Anonymize content for privacy
    pub fn anonymize_content(&self, content: &str) -> String {
        let mut result = content.to_string();
        
        // Email pattern
        let email_re = regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap();
        result = email_re.replace_all(&result, "[EMAIL]").to_string();
        
        // IP pattern
        let ip_re = regex::Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b").unwrap();
        result = ip_re.replace_all(&result, "[IP_ADDRESS]").to_string();
        
        result
    }

    fn build_analysis_prompt(&self, content: &str) -> String {
        format!(r#"You are a technical requirements analyst. Analyze the following PRD/spec for clarity and completeness.

SPEC CONTENT:
{}

Analyze and respond with a JSON object containing:
1. "clarity_score": number 0-100 (how clear and unambiguous the spec is)
2. "justification": string (brief explanation of the score)
3. "ambiguous_phrases": array of objects with "text", "explanation", "suggested_replacement"
4. "missing_scenarios": array of objects with "scenario", "impact"
5. "risks": array of objects with "risk", "mitigation"

Focus on:
- Ambiguous terms like "fast", "user-friendly", "robust", "simple", "easy"
- Missing edge cases and error scenarios
- Non-testable requirements
- Security and performance considerations

Respond ONLY with valid JSON, no markdown formatting."#, content)
    }

    async fn generate_content(&self, prompt: &str) -> Result<String, IntegrationError> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            self.config.model
        );

        let body = serde_json::json!({
            "contents": [{
                "parts": [{
                    "text": prompt
                }]
            }],
            "generationConfig": {
                "temperature": 0.2,
                "maxOutputTokens": 4096
            }
        });

        let api_key = self.config.api_key.as_ref().unwrap();

        let response = self.http_client
            .post(&url)
            .header("Content-Type", "application/json")
            .query(&[("key", api_key)])
            .json(&body)
            .send()
            .await?;

        match response.status().as_u16() {
            200 => {
                let result: GeminiResponse = response.json().await
                    .map_err(|e| IntegrationError::ParseError(e.to_string()))?;
                
                result.candidates
                    .first()
                    .and_then(|c| c.content.parts.first())
                    .map(|p| p.text.clone())
                    .ok_or_else(|| IntegrationError::ApiError("Empty response".to_string()))
            }
            401 => Err(IntegrationError::Auth("Invalid API key".to_string())),
            429 => Err(IntegrationError::RateLimit),
            status => {
                let body = response.text().await.unwrap_or_default();
                Err(IntegrationError::ApiError(format!("Status {}: {}", status, body)))
            }
        }
    }

    fn parse_analysis(&self, response: &str) -> Result<SpecAnalysis, IntegrationError> {
        // Try to extract JSON from response (handle potential markdown wrapping)
        let json_str = if response.contains("```json") {
            response
                .split("```json")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(response)
        } else if response.contains("```") {
            response
                .split("```")
                .nth(1)
                .unwrap_or(response)
        } else {
            response
        };

        serde_json::from_str(json_str.trim())
            .map_err(|e| IntegrationError::ParseError(format!("Failed to parse analysis: {}", e)))
    }
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Debug, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Deserialize)]
struct GeminiPart {
    text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_config_default() {
        let config = GeminiConfig::default();
        assert_eq!(config.model, "gemini-pro");
        assert!(config.api_key.is_none());
    }

    #[test]
    fn test_gemini_config_builder() {
        let config = GeminiConfig::new("gemini-1.5-pro")
            .with_api_key("test-key");

        assert_eq!(config.model, "gemini-1.5-pro");
        assert_eq!(config.api_key, Some("test-key".to_string()));
    }

    #[test]
    fn test_client_requires_api_key() {
        let config = GeminiConfig::default();
        let result = GeminiClient::new(config);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IntegrationError::ConfigError(_)));
    }

    #[test]
    fn test_anonymize_content_emails() {
        let config = GeminiConfig::default().with_api_key("key");
        let client = GeminiClient::new(config).unwrap();

        let content = "Contact john.doe@example.com for details";
        let anonymized = client.anonymize_content(content);

        assert!(!anonymized.contains("john.doe@example.com"));
        assert!(anonymized.contains("[EMAIL]"));
    }

    #[test]
    fn test_anonymize_content_ips() {
        let config = GeminiConfig::default().with_api_key("key");
        let client = GeminiClient::new(config).unwrap();

        let content = "Server at 192.168.1.100";
        let anonymized = client.anonymize_content(content);

        assert!(!anonymized.contains("192.168.1.100"));
        assert!(anonymized.contains("[IP_ADDRESS]"));
    }

    #[test]
    fn test_parse_analysis_json() {
        let config = GeminiConfig::default().with_api_key("key");
        let client = GeminiClient::new(config).unwrap();

        let json = r#"{
            "clarity_score": 75,
            "justification": "Good overall",
            "ambiguous_phrases": [],
            "missing_scenarios": [],
            "risks": []
        }"#;

        let result = client.parse_analysis(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().clarity_score, 75);
    }

    #[test]
    fn test_parse_analysis_with_markdown() {
        let config = GeminiConfig::default().with_api_key("key");
        let client = GeminiClient::new(config).unwrap();

        let json = r#"```json
{
    "clarity_score": 80,
    "justification": "Clear spec",
    "ambiguous_phrases": [],
    "missing_scenarios": [],
    "risks": []
}
```"#;

        let result = client.parse_analysis(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().clarity_score, 80);
    }

    #[test]
    fn test_spec_analysis_struct() {
        let analysis = SpecAnalysis {
            clarity_score: 85,
            justification: "Well written".to_string(),
            ambiguous_phrases: vec![AmbiguousPhrase {
                text: "fast".to_string(),
                explanation: "Not measurable".to_string(),
                suggested_replacement: "< 200ms response time".to_string(),
            }],
            missing_scenarios: vec![MissingScenario {
                scenario: "Network failure".to_string(),
                impact: "User may see errors".to_string(),
            }],
            risks: vec![Risk {
                risk: "Performance".to_string(),
                mitigation: "Add caching".to_string(),
            }],
        };

        assert_eq!(analysis.clarity_score, 85);
        assert_eq!(analysis.ambiguous_phrases.len(), 1);
    }
}
