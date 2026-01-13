//! PR Commands
//!
//! Tauri commands for pull request aggregation and monitoring.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::commands::search::CommandError;

/// PR list request parameters
#[derive(Debug, Clone, Deserialize)]
pub struct PrListParams {
    #[serde(default)]
    pub repositories: Vec<String>,
    #[serde(default)]
    pub stale_only: bool,
    #[serde(default)]
    pub pending_review_only: bool,
    #[serde(default = "default_pr_limit")]
    pub limit: usize,
}

fn default_pr_limit() -> usize {
    20
}

/// PR summary response
#[derive(Debug, Clone, Serialize)]
pub struct PrSummaryResponse {
    #[serde(rename = "totalOpen")]
    pub total_open: usize,
    #[serde(rename = "pendingReview")]
    pub pending_review: usize,
    #[serde(rename = "staleCount")]
    pub stale_count: usize,
    #[serde(rename = "byRepository")]
    pub by_repository: HashMap<String, usize>,
    #[serde(rename = "trayState")]
    pub tray_state: String,
}

/// PR item for list response
#[derive(Debug, Clone, Serialize)]
pub struct PrItemDto {
    pub id: String,
    pub repository: String,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub author: UserDto,
    pub reviewers: Vec<ReviewerDto>,
    #[serde(rename = "sourceBranch")]
    pub source_branch: String,
    #[serde(rename = "targetBranch")]
    pub target_branch: String,
    #[serde(rename = "checksStatus")]
    pub checks_status: String,
    #[serde(rename = "isStale")]
    pub is_stale: bool,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub url: String,
    #[serde(rename = "ageHours")]
    pub age_hours: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserDto {
    pub id: String,
    pub name: String,
    pub avatar: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReviewerDto {
    pub user: UserDto,
    pub approved: bool,
}

/// Grouped PRs response
#[derive(Debug, Clone, Serialize)]
pub struct GroupedPrsResponse {
    pub groups: Vec<PrGroupDto>,
    #[serde(rename = "totalCount")]
    pub total_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct PrGroupDto {
    pub label: String,
    pub prs: Vec<PrItemDto>,
    #[serde(rename = "staleCount")]
    pub stale_count: usize,
}

/// Get PR summary
#[tauri::command]
pub async fn get_pr_summary() -> Result<PrSummaryResponse, CommandError> {
    // TODO: Wire up to actual PrAggregator service
    Ok(PrSummaryResponse {
        total_open: 0,
        pending_review: 0,
        stale_count: 0,
        by_repository: HashMap::new(),
        tray_state: "neutral".to_string(),
    })
}

/// Get list of PRs
#[tauri::command]
pub async fn get_prs(params: PrListParams) -> Result<Vec<PrItemDto>, CommandError> {
    // TODO: Wire up to actual PrAggregator service
    Ok(vec![])
}

/// Get PRs pending user review
#[tauri::command]
pub async fn get_pending_review_prs() -> Result<Vec<PrItemDto>, CommandError> {
    // TODO: Wire up to actual PrAggregator service
    Ok(vec![])
}

/// Get stale PRs
#[tauri::command]
pub async fn get_stale_prs() -> Result<Vec<PrItemDto>, CommandError> {
    // TODO: Wire up to actual PrAggregator service
    Ok(vec![])
}

/// Get PRs grouped by repository
#[tauri::command]
pub async fn get_prs_grouped_by_repo() -> Result<GroupedPrsResponse, CommandError> {
    Ok(GroupedPrsResponse {
        groups: vec![],
        total_count: 0,
    })
}

/// Get PRs grouped by age
#[tauri::command]
pub async fn get_prs_grouped_by_age() -> Result<GroupedPrsResponse, CommandError> {
    Ok(GroupedPrsResponse {
        groups: vec![],
        total_count: 0,
    })
}

/// Refresh PR data (bypass cache)
#[tauri::command]
pub async fn refresh_prs() -> Result<PrSummaryResponse, CommandError> {
    get_pr_summary().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pr_list_params_defaults() {
        let json = r#"{"repositories": []}"#;
        let params: PrListParams = serde_json::from_str(json).unwrap();
        
        assert_eq!(params.limit, 20);
        assert!(!params.stale_only);
        assert!(!params.pending_review_only);
    }

    #[test]
    fn test_pr_summary_serialization() {
        let summary = PrSummaryResponse {
            total_open: 5,
            pending_review: 2,
            stale_count: 1,
            by_repository: HashMap::from([("repo1".to_string(), 3)]),
            tray_state: "amber".to_string(),
        };

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"totalOpen\":5"));
        assert!(json.contains("\"staleCount\":1"));
    }

    #[test]
    fn test_pr_item_dto_serialization() {
        let pr = PrItemDto {
            id: "123".to_string(),
            repository: "repo1".to_string(),
            title: "Fix bug".to_string(),
            description: None,
            state: "open".to_string(),
            author: UserDto {
                id: "u1".to_string(),
                name: "John".to_string(),
                avatar: None,
            },
            reviewers: vec![],
            source_branch: "feature".to_string(),
            target_branch: "main".to_string(),
            checks_status: "pass".to_string(),
            is_stale: false,
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            url: "https://example.com".to_string(),
            age_hours: 24,
        };

        let json = serde_json::to_string(&pr).unwrap();
        assert!(json.contains("\"sourceBranch\":\"feature\""));
        assert!(json.contains("\"isStale\":false"));
    }

    #[tokio::test]
    async fn test_get_pr_summary() {
        let result = get_pr_summary().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_prs() {
        let params = PrListParams {
            repositories: vec![],
            stale_only: false,
            pending_review_only: false,
            limit: 10,
        };
        let result = get_prs(params).await;
        assert!(result.is_ok());
    }
}
