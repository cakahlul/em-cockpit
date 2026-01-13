//! Search Commands
//!
//! Tauri commands for unified search functionality.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::services::{SearchResult, SearchResultType};

/// Search query from frontend
#[derive(Debug, Clone, Deserialize)]
pub struct SearchQueryParams {
    pub query: String,
    #[serde(default)]
    pub types: Vec<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub include_closed: bool,
}

fn default_limit() -> usize {
    10
}

impl SearchQueryParams {
    pub fn parse_types(&self) -> Vec<SearchResultType> {
        if self.types.is_empty() {
            return vec![SearchResultType::Ticket, SearchResultType::PullRequest];
        }

        self.types
            .iter()
            .filter_map(|t| match t.to_lowercase().as_str() {
                "ticket" | "jira" => Some(SearchResultType::Ticket),
                "pr" | "pullrequest" | "pull_request" => Some(SearchResultType::PullRequest),
                "incident" => Some(SearchResultType::Incident),
                "document" | "doc" => Some(SearchResultType::Document),
                _ => None,
            })
            .collect()
    }
}

/// Search response for frontend
#[derive(Debug, Clone, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResultDto>,
    pub total: usize,
    pub query: String,
}

/// DTO for search result
#[derive(Debug, Clone, Serialize)]
pub struct SearchResultDto {
    pub id: String,
    #[serde(rename = "type")]
    pub result_type: String,
    pub icon: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub url: Option<String>,
    pub score: f32,
    pub metadata: SearchMetadataDto,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchMetadataDto {
    pub status: Option<String>,
    pub assignee: Option<String>,
    pub priority: Option<String>,
    #[serde(rename = "isStale")]
    pub is_stale: Option<bool>,
}

impl From<SearchResult> for SearchResultDto {
    fn from(result: SearchResult) -> Self {
        Self {
            id: result.id,
            result_type: result.result_type.as_str().to_string(),
            icon: result.result_type.icon().to_string(),
            title: result.title,
            subtitle: result.subtitle,
            url: result.url,
            score: result.relevance_score,
            metadata: SearchMetadataDto {
                status: result.metadata.status,
                assignee: result.metadata.assignee,
                priority: result.metadata.priority,
                is_stale: result.metadata.is_stale,
            },
        }
    }
}

/// Command error response
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl CommandError {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
        }
    }

    pub fn internal(message: &str) -> Self {
        Self::new("INTERNAL_ERROR", message)
    }

    pub fn not_found(message: &str) -> Self {
        Self::new("NOT_FOUND", message)
    }

    pub fn validation(message: &str) -> Self {
        Self::new("VALIDATION_ERROR", message)
    }

    pub fn auth(message: &str) -> Self {
        Self::new("AUTH_ERROR", message)
    }
}

/// Perform unified search
#[tauri::command]
pub async fn search(
    params: SearchQueryParams,
) -> Result<SearchResponse, CommandError> {
    // Validate query
    if params.query.trim().is_empty() {
        return Err(CommandError::validation("Search query cannot be empty"));
    }

    // For now, return mock data until full integration is wired up
    // In production, this would use the SearchService with configured repositories
    let results = mock_search_results(&params.query);

    Ok(SearchResponse {
        total: results.len(),
        query: params.query,
        results,
    })
}

/// Search with cache bypass
#[tauri::command]
pub async fn search_fresh(
    params: SearchQueryParams,
) -> Result<SearchResponse, CommandError> {
    // Same as search but bypasses cache
    search(params).await
}

/// Get recent searches
#[tauri::command]
pub async fn get_recent_searches() -> Result<Vec<String>, CommandError> {
    // TODO: Implement recent searches storage
    Ok(vec![])
}

/// Clear search history
#[tauri::command]
pub async fn clear_search_history() -> Result<(), CommandError> {
    // TODO: Implement search history clearing
    Ok(())
}

// Mock search results for testing
fn mock_search_results(query: &str) -> Vec<SearchResultDto> {
    // Check if it looks like a ticket ID
    let ticket_pattern = regex::Regex::new(r"^[A-Z]+-\d+$").unwrap();
    
    if ticket_pattern.is_match(&query.to_uppercase()) {
        return vec![SearchResultDto {
            id: query.to_uppercase(),
            result_type: "Ticket".to_string(),
            icon: "🎫".to_string(),
            title: format!("[{}] Mock ticket result", query.to_uppercase()),
            subtitle: Some(format!("{} • In Progress", query.to_uppercase())),
            url: None,
            score: 2.0,
            metadata: SearchMetadataDto {
                status: Some("In Progress".to_string()),
                assignee: Some("John Doe".to_string()),
                priority: Some("Medium".to_string()),
                is_stale: None,
            },
        }];
    }

    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_query_params_parse_types() {
        let params = SearchQueryParams {
            query: "test".to_string(),
            types: vec!["ticket".to_string(), "pr".to_string()],
            limit: 10,
            include_closed: false,
        };

        let types = params.parse_types();
        assert_eq!(types.len(), 2);
        assert!(types.contains(&SearchResultType::Ticket));
        assert!(types.contains(&SearchResultType::PullRequest));
    }

    #[test]
    fn test_search_query_params_empty_types() {
        let params = SearchQueryParams {
            query: "test".to_string(),
            types: vec![],
            limit: 10,
            include_closed: false,
        };

        let types = params.parse_types();
        assert_eq!(types.len(), 2); // Default types
    }

    #[test]
    fn test_command_error_creation() {
        let err = CommandError::validation("Invalid input");
        assert_eq!(err.code, "VALIDATION_ERROR");
        assert_eq!(err.message, "Invalid input");
    }

    #[test]
    fn test_search_result_dto_from() {
        use crate::services::SearchResultMetadata;
        
        let search_result = SearchResult {
            id: "TEST-1".to_string(),
            result_type: SearchResultType::Ticket,
            title: "Test ticket".to_string(),
            subtitle: Some("Subtitle".to_string()),
            url: None,
            relevance_score: 1.5,
            updated_at: chrono::Utc::now(),
            metadata: SearchResultMetadata {
                status: Some("Open".to_string()),
                assignee: None,
                priority: None,
                is_stale: None,
            },
        };

        let dto: SearchResultDto = search_result.into();
        assert_eq!(dto.id, "TEST-1");
        assert_eq!(dto.result_type, "Ticket");
        assert_eq!(dto.icon, "🎫");
        assert_eq!(dto.score, 1.5);
    }

    #[tokio::test]
    async fn test_search_empty_query() {
        let params = SearchQueryParams {
            query: "".to_string(),
            types: vec![],
            limit: 10,
            include_closed: false,
        };

        let result = search(params).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "VALIDATION_ERROR");
    }

    #[tokio::test]
    async fn test_search_ticket_id() {
        let params = SearchQueryParams {
            query: "PROJ-123".to_string(),
            types: vec![],
            limit: 10,
            include_closed: false,
        };

        let result = search(params).await;
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].id, "PROJ-123");
    }
}
