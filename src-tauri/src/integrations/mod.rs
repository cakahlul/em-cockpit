//! Integrations Layer
//!
//! Provides external API integrations following the Repository Pattern
//! for Jira, Git hosting, Documentation, Monitoring, and AI services.

pub mod traits;
pub mod jira;
pub mod git;
pub mod ai;
pub mod monitoring;

// Re-export common types
pub use traits::{TicketRepository, PullRequestRepository, MetricsRepository};
pub use jira::{JiraClient, JiraConfig};
pub use git::{GitProvider, GitConfig, GitProviderType};
pub use ai::{GeminiClient, SpecAnalysis};
pub use monitoring::{GrafanaClient, MonitoringConfig as GrafanaConfig};
