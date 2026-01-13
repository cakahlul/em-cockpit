//! Common traits for integration repositories
//!
//! Defines the Repository Pattern interfaces that all integrations implement,
//! following Interface Segregation and Dependency Inversion principles.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during integration operations
#[derive(Error, Debug)]
pub enum IntegrationError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

impl From<reqwest::Error> for IntegrationError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            IntegrationError::Network("Request timeout".to_string())
        } else if err.is_connect() {
            IntegrationError::Network("Connection failed".to_string())
        } else {
            IntegrationError::Network(err.to_string())
        }
    }
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
}

/// Priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Highest,
    High,
    Medium,
    Low,
    Lowest,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Highest => "Highest",
            Priority::High => "High",
            Priority::Medium => "Medium",
            Priority::Low => "Low",
            Priority::Lowest => "Lowest",
        }
    }
}

/// Ticket status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketStatus {
    pub name: String,
    pub category: StatusCategory,
}

/// Status category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusCategory {
    Todo,
    InProgress,
    Done,
}

/// Ticket/Issue representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticket {
    pub id: String,
    pub key: String,
    pub summary: String,
    pub description: Option<String>,
    pub status: TicketStatus,
    pub assignee: Option<User>,
    pub reporter: Option<User>,
    pub priority: Option<Priority>,
    pub sprint: Option<String>,
    pub labels: Vec<String>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Pull request state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrState {
    Open,
    Merged,
    Declined,
    Draft,
}

impl PrState {
    pub fn as_str(&self) -> &'static str {
        match self {
            PrState::Open => "Open",
            PrState::Merged => "Merged",
            PrState::Declined => "Declined",
            PrState::Draft => "Draft",
        }
    }
}

/// CI/CD check status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChecksStatus {
    Pass,
    Fail,
    Running,
    None,
}

/// Reviewer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reviewer {
    pub user: User,
    pub approved: bool,
}

/// Pull request representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: String,
    pub repository: String,
    pub title: String,
    pub description: Option<String>,
    pub state: PrState,
    pub author: User,
    pub reviewers: Vec<Reviewer>,
    pub source_branch: String,
    pub target_branch: String,
    pub checks_status: ChecksStatus,
    pub is_stale: bool,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub url: String,
}

/// Incident severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Low => "Low",
            Severity::Medium => "Medium",
            Severity::High => "High",
            Severity::Critical => "Critical",
        }
    }
}

/// Incident status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentStatus {
    Firing,
    Resolved,
}

/// Metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
}

/// Incident representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub id: String,
    pub service: String,
    pub severity: Severity,
    pub status: IncidentStatus,
    pub started_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub description: String,
    pub runbook_url: Option<String>,
}

/// Search query for tickets
#[derive(Debug, Clone, Default)]
pub struct TicketSearchQuery {
    pub text: Option<String>,
    pub project: Option<String>,
    pub assignee: Option<String>,
    pub status: Option<String>,
    pub limit: usize,
}

impl TicketSearchQuery {
    pub fn new() -> Self {
        Self {
            limit: 10,
            ..Default::default()
        }
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.text = Some(text.to_string());
        self
    }

    pub fn with_project(mut self, project: &str) -> Self {
        self.project = Some(project.to_string());
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

/// Filter for pull requests
#[derive(Debug, Clone, Default)]
pub struct PrFilter {
    pub repositories: Vec<String>,
    pub state: Option<PrState>,
    pub stale_only: bool,
    pub stale_threshold_hours: u32,
    pub limit: usize,
}

impl PrFilter {
    pub fn new() -> Self {
        Self {
            stale_threshold_hours: 48,
            limit: 20,
            ..Default::default()
        }
    }

    pub fn with_repositories(mut self, repos: Vec<String>) -> Self {
        self.repositories = repos;
        self
    }

    pub fn stale_only(mut self) -> Self {
        self.stale_only = true;
        self
    }
}

/// Repository trait for ticket operations (Jira)
#[async_trait]
pub trait TicketRepository: Send + Sync {
    /// Find a ticket by ID/key
    async fn find_by_id(&self, id: &str) -> Result<Ticket, IntegrationError>;

    /// Search tickets
    async fn search(&self, query: &TicketSearchQuery) -> Result<Vec<Ticket>, IntegrationError>;
}

/// Repository trait for pull request operations (Git hosting)
#[async_trait]
pub trait PullRequestRepository: Send + Sync {
    /// Find a PR by ID
    async fn find_by_id(&self, repo: &str, id: &str) -> Result<PullRequest, IntegrationError>;

    /// Find PRs where user is a reviewer
    async fn find_by_reviewer(
        &self,
        user_id: &str,
        filter: &PrFilter,
    ) -> Result<Vec<PullRequest>, IntegrationError>;

    /// Get all open PRs for repositories
    async fn get_open_prs(&self, filter: &PrFilter) -> Result<Vec<PullRequest>, IntegrationError>;
}

/// Repository trait for metrics/incident operations (Monitoring)
#[async_trait]
pub trait MetricsRepository: Send + Sync {
    /// Get current metrics for a service
    async fn get_metrics(&self, service: &str) -> Result<Vec<Metric>, IntegrationError>;

    /// Get active incidents
    async fn get_incidents(&self) -> Result<Vec<Incident>, IntegrationError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_as_str() {
        assert_eq!(Priority::Highest.as_str(), "Highest");
        assert_eq!(Priority::Low.as_str(), "Low");
    }

    #[test]
    fn test_pr_state_as_str() {
        assert_eq!(PrState::Open.as_str(), "Open");
        assert_eq!(PrState::Merged.as_str(), "Merged");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
    }

    #[test]
    fn test_ticket_search_query_builder() {
        let query = TicketSearchQuery::new()
            .with_text("bug")
            .with_project("PROJ")
            .with_limit(5);

        assert_eq!(query.text, Some("bug".to_string()));
        assert_eq!(query.project, Some("PROJ".to_string()));
        assert_eq!(query.limit, 5);
    }

    #[test]
    fn test_pr_filter_builder() {
        let filter = PrFilter::new()
            .with_repositories(vec!["repo1".to_string()])
            .stale_only();

        assert_eq!(filter.repositories.len(), 1);
        assert!(filter.stale_only);
        assert_eq!(filter.stale_threshold_hours, 48);
    }
}
