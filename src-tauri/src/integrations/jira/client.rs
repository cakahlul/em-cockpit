//! Jira API Client
//!
//! Implements TicketRepository for Jira REST API.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::integrations::traits::{
    IntegrationError, Priority, StatusCategory, Ticket, TicketRepository, TicketSearchQuery,
    TicketStatus, User,
};

/// Jira client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraConfig {
    /// Base URL (e.g., "https://company.atlassian.net")
    pub base_url: String,
    /// Username (email)
    pub username: String,
    /// API token (not stored here, retrieved from credential manager)
    #[serde(skip)]
    pub token: Option<String>,
    /// Default project
    pub default_project: Option<String>,
}

impl JiraConfig {
    pub fn new(base_url: &str, username: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            username: username.to_string(),
            token: None,
            default_project: None,
        }
    }

    pub fn with_token(mut self, token: &str) -> Self {
        self.token = Some(token.to_string());
        self
    }

    pub fn with_default_project(mut self, project: &str) -> Self {
        self.default_project = Some(project.to_string());
        self
    }
}

/// Jira API client
#[derive(Debug)]
pub struct JiraClient {
    config: JiraConfig,
    http_client: Client,
}

impl JiraClient {
    /// Create a new Jira client
    pub fn new(config: JiraConfig) -> Result<Self, IntegrationError> {
        if config.token.is_none() {
            return Err(IntegrationError::ConfigError(
                "Jira token is required".to_string(),
            ));
        }

        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| IntegrationError::Network(e.to_string()))?;

        Ok(Self {
            config,
            http_client,
        })
    }

    /// Create for testing with mock capabilities
    #[cfg(test)]
    pub fn new_for_test(config: JiraConfig, client: Client) -> Self {
        Self {
            config,
            http_client: client,
        }
    }

    /// Build authorization header value
    fn auth_header(&self) -> String {
        use base64::Engine;
        let credentials = format!(
            "{}:{}",
            self.config.username,
            self.config.token.as_deref().unwrap_or("")
        );
        format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode(credentials)
        )
    }

    /// Build JQL query from search parameters
    fn build_jql(&self, query: &TicketSearchQuery) -> String {
        let mut conditions = Vec::new();

        if let Some(ref project) = query.project {
            conditions.push(format!("project = {}", project));
        } else if let Some(ref default_project) = self.config.default_project {
            conditions.push(format!("project = {}", default_project));
        }

        if let Some(ref text) = query.text {
            conditions.push(format!("text ~ \"{}\"", text));
        }

        if let Some(ref assignee) = query.assignee {
            conditions.push(format!("assignee = \"{}\"", assignee));
        }

        if let Some(ref status) = query.status {
            conditions.push(format!("status = \"{}\"", status));
        }

        if conditions.is_empty() {
            "ORDER BY updated DESC".to_string()
        } else {
            format!("{} ORDER BY updated DESC", conditions.join(" AND "))
        }
    }

    /// Map Jira issue to domain Ticket
    fn map_issue(&self, issue: &JiraIssue) -> Ticket {
        let fields = &issue.fields;

        Ticket {
            id: issue.id.clone(),
            key: issue.key.clone(),
            summary: fields.summary.clone(),
            description: fields.description.clone(),
            status: TicketStatus {
                name: fields.status.name.clone(),
                category: self.map_status_category(&fields.status.status_category),
            },
            assignee: fields.assignee.as_ref().map(|a| User {
                id: a.account_id.clone(),
                name: a.display_name.clone(),
                email: a.email_address.clone(),
                avatar_url: a.avatar_urls.as_ref().and_then(|av| av.x48.clone()),
            }),
            reporter: fields.reporter.as_ref().map(|r| User {
                id: r.account_id.clone(),
                name: r.display_name.clone(),
                email: r.email_address.clone(),
                avatar_url: r.avatar_urls.as_ref().and_then(|av| av.x48.clone()),
            }),
            priority: fields.priority.as_ref().map(|p| self.map_priority(&p.name)),
            sprint: fields.sprint.as_ref().map(|s| s.name.clone()),
            labels: fields.labels.clone().unwrap_or_default(),
            updated_at: fields.updated.unwrap_or_else(Utc::now),
            created_at: fields.created.unwrap_or_else(Utc::now),
        }
    }

    fn map_status_category(&self, category: &JiraStatusCategory) -> StatusCategory {
        match category.key.as_str() {
            "new" | "undefined" => StatusCategory::Todo,
            "indeterminate" => StatusCategory::InProgress,
            "done" => StatusCategory::Done,
            _ => StatusCategory::Todo,
        }
    }

    fn map_priority(&self, name: &str) -> Priority {
        match name.to_lowercase().as_str() {
            "highest" | "blocker" => Priority::Highest,
            "high" | "critical" => Priority::High,
            "medium" | "major" => Priority::Medium,
            "low" | "minor" => Priority::Low,
            "lowest" | "trivial" => Priority::Lowest,
            _ => Priority::Medium,
        }
    }
}

#[async_trait]
impl TicketRepository for JiraClient {
    async fn find_by_id(&self, id: &str) -> Result<Ticket, IntegrationError> {
        let url = format!(
            "{}/rest/api/3/issue/{}",
            self.config.base_url, id
        );

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/json")
            .send()
            .await?;

        match response.status().as_u16() {
            200 => {
                let issue: JiraIssue = response
                    .json()
                    .await
                    .map_err(|e| IntegrationError::ParseError(e.to_string()))?;
                Ok(self.map_issue(&issue))
            }
            401 => Err(IntegrationError::Auth("Invalid credentials".to_string())),
            404 => Err(IntegrationError::NotFound(format!("Issue {} not found", id))),
            429 => Err(IntegrationError::RateLimit),
            status => {
                let body = response.text().await.unwrap_or_default();
                Err(IntegrationError::ApiError(format!(
                    "Status {}: {}",
                    status, body
                )))
            }
        }
    }

    async fn search(&self, query: &TicketSearchQuery) -> Result<Vec<Ticket>, IntegrationError> {
        let url = format!("{}/rest/api/3/search", self.config.base_url);
        let jql = self.build_jql(query);

        let body = serde_json::json!({
            "jql": jql,
            "maxResults": query.limit,
            "fields": [
                "summary", "description", "status", "assignee", "reporter",
                "priority", "labels", "updated", "created"
            ]
        });

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        match response.status().as_u16() {
            200 => {
                let result: JiraSearchResult = response
                    .json()
                    .await
                    .map_err(|e| IntegrationError::ParseError(e.to_string()))?;
                Ok(result.issues.iter().map(|i| self.map_issue(i)).collect())
            }
            401 => Err(IntegrationError::Auth("Invalid credentials".to_string())),
            429 => Err(IntegrationError::RateLimit),
            status => {
                let body = response.text().await.unwrap_or_default();
                Err(IntegrationError::ApiError(format!(
                    "Status {}: {}",
                    status, body
                )))
            }
        }
    }
}

// ===== Jira API Response Types =====

#[derive(Debug, Deserialize)]
struct JiraSearchResult {
    issues: Vec<JiraIssue>,
    #[allow(dead_code)]
    total: i32,
}

#[derive(Debug, Deserialize)]
struct JiraIssue {
    id: String,
    key: String,
    fields: JiraFields,
}

#[derive(Debug, Deserialize)]
struct JiraFields {
    summary: String,
    description: Option<String>,
    status: JiraStatus,
    assignee: Option<JiraUser>,
    reporter: Option<JiraUser>,
    priority: Option<JiraPriority>,
    sprint: Option<JiraSprint>,
    labels: Option<Vec<String>>,
    updated: Option<DateTime<Utc>>,
    created: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct JiraStatus {
    name: String,
    #[serde(rename = "statusCategory")]
    status_category: JiraStatusCategory,
}

#[derive(Debug, Deserialize)]
struct JiraStatusCategory {
    key: String,
}

#[derive(Debug, Deserialize)]
struct JiraUser {
    #[serde(rename = "accountId")]
    account_id: String,
    #[serde(rename = "displayName")]
    display_name: String,
    #[serde(rename = "emailAddress")]
    email_address: Option<String>,
    #[serde(rename = "avatarUrls")]
    avatar_urls: Option<JiraAvatarUrls>,
}

#[derive(Debug, Deserialize)]
struct JiraAvatarUrls {
    #[serde(rename = "48x48")]
    x48: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JiraPriority {
    name: String,
}

#[derive(Debug, Deserialize)]
struct JiraSprint {
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> JiraConfig {
        JiraConfig::new("https://test.atlassian.net", "user@test.com")
            .with_token("test-token")
            .with_default_project("TEST")
    }

    #[test]
    fn test_jira_config_creation() {
        let config = test_config();

        assert_eq!(config.base_url, "https://test.atlassian.net");
        assert_eq!(config.username, "user@test.com");
        assert_eq!(config.token, Some("test-token".to_string()));
        assert_eq!(config.default_project, Some("TEST".to_string()));
    }

    #[test]
    fn test_jira_config_trims_trailing_slash() {
        let config = JiraConfig::new("https://test.atlassian.net/", "user@test.com");
        assert_eq!(config.base_url, "https://test.atlassian.net");
    }

    #[test]
    fn test_client_requires_token() {
        let config = JiraConfig::new("https://test.atlassian.net", "user@test.com");
        let result = JiraClient::new(config);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            IntegrationError::ConfigError(_)
        ));
    }

    #[test]
    fn test_build_jql_with_text() {
        let config = test_config();
        let client = JiraClient::new(config).unwrap();

        let query = TicketSearchQuery::new().with_text("bug fix");
        let jql = client.build_jql(&query);

        assert!(jql.contains("project = TEST"));
        assert!(jql.contains("text ~ \"bug fix\""));
    }

    #[test]
    fn test_build_jql_with_project_override() {
        let config = test_config();
        let client = JiraClient::new(config).unwrap();

        let query = TicketSearchQuery::new()
            .with_project("OTHER")
            .with_text("feature");
        let jql = client.build_jql(&query);

        assert!(jql.contains("project = OTHER"));
        assert!(!jql.contains("project = TEST"));
    }

    #[test]
    fn test_build_jql_empty_query() {
        let config = JiraConfig::new("https://test.atlassian.net", "user@test.com")
            .with_token("token");
        let client = JiraClient::new(config).unwrap();

        let query = TicketSearchQuery::new();
        let jql = client.build_jql(&query);

        assert_eq!(jql, "ORDER BY updated DESC");
    }

    #[test]
    fn test_map_priority() {
        let config = test_config();
        let client = JiraClient::new(config).unwrap();

        assert_eq!(client.map_priority("Highest"), Priority::Highest);
        assert_eq!(client.map_priority("Critical"), Priority::High);
        assert_eq!(client.map_priority("Major"), Priority::Medium);
        assert_eq!(client.map_priority("Minor"), Priority::Low);
        assert_eq!(client.map_priority("Trivial"), Priority::Lowest);
        assert_eq!(client.map_priority("Unknown"), Priority::Medium);
    }

    #[test]
    fn test_auth_header_format() {
        let config = test_config();
        let client = JiraClient::new(config).unwrap();

        let header = client.auth_header();
        assert!(header.starts_with("Basic "));
    }
}
