//! Application configuration models
//!
//! Defines the structure of configuration settings for all integrations
//! and application preferences.

use serde::{Deserialize, Serialize};

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// Integration configurations
    pub integrations: IntegrationConfig,
    /// Shortcut configurations  
    pub shortcuts: ShortcutConfig,
    /// Appearance settings
    pub appearance: AppearanceConfig,
    /// User preferences
    pub preferences: PreferencesConfig,
}

/// Integration configuration for all external services
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntegrationConfig {
    /// Jira configuration
    pub jira: Option<JiraConfig>,
    /// Git hosting configuration
    pub git: Option<GitConfig>,
    /// Documentation platform configuration
    pub docs: Option<DocsConfig>,
    /// Monitoring platform configuration
    pub monitoring: Option<MonitoringConfig>,
    /// Gemini AI configuration
    pub gemini: Option<GeminiConfig>,
}

/// Jira integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraConfig {
    /// Base URL of the Jira instance
    pub base_url: String,
    /// Default project filter
    pub default_project: Option<String>,
    /// Username (email) for authentication
    pub username: Option<String>,
}

/// Git hosting provider type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GitProviderType {
    Bitbucket,
    GitHub,
    GitLab,
}

/// Git hosting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    /// Provider type (Bitbucket, GitHub, GitLab)
    pub provider: GitProviderType,
    /// Base URL (for self-hosted instances)
    pub base_url: Option<String>,
    /// Workspace/organization name
    pub workspace: Option<String>,
    /// Repositories to monitor
    pub repositories: Vec<String>,
}

/// Documentation platform configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsConfig {
    /// Platform type (confluence, notion)
    pub platform: String,
    /// Base URL
    pub base_url: String,
}

/// Monitoring platform configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Platform type (grafana, datadog)
    pub platform: String,
    /// Base URL
    pub base_url: String,
    /// Services to monitor
    pub services: Vec<ServiceConfig>,
}

/// Service monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service name
    pub name: String,
    /// Dashboard ID/path
    pub dashboard_id: Option<String>,
    /// Custom thresholds
    pub thresholds: Option<ThresholdConfig>,
}

/// Threshold configuration for alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    /// Error rate threshold for amber (%)
    pub error_rate_amber: f64,
    /// Error rate threshold for red (%)
    pub error_rate_red: f64,
    /// Latency threshold for amber (ms)
    pub latency_amber_ms: u64,
    /// Latency threshold for red (ms)
    pub latency_red_ms: u64,
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            error_rate_amber: 1.0,
            error_rate_red: 5.0,
            latency_amber_ms: 500,
            latency_red_ms: 1000,
        }
    }
}

/// Gemini AI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiConfig {
    /// Model to use (e.g., "gemini-pro")
    pub model: String,
    /// Optional daily token limit
    pub daily_token_limit: Option<u32>,
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self {
            model: "gemini-pro".to_string(),
            daily_token_limit: None,
        }
    }
}

/// Shortcut key configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    /// Global hotkey to open Flight Console
    pub flight_console: String,
    /// Quick navigation shortcuts
    pub quick_nav: QuickNavShortcuts,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            flight_console: "Alt+Space".to_string(),
            quick_nav: QuickNavShortcuts::default(),
        }
    }
}

/// Quick navigation shortcuts within the app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickNavShortcuts {
    /// Open Flight Console
    pub console: String,
    /// Open Radar Panel
    pub radar: String,
    /// Open Incident Radar
    pub incidents: String,
}

impl Default for QuickNavShortcuts {
    fn default() -> Self {
        Self {
            console: "Ctrl+1".to_string(),
            radar: "Ctrl+2".to_string(),
            incidents: "Ctrl+3".to_string(),
        }
    }
}

/// Appearance settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    /// Theme selection
    pub theme: ThemeMode,
    /// Glass effect intensity (0.0 - 1.0)
    pub glass_intensity: f32,
    /// Reduce transparency for accessibility
    pub reduce_transparency: bool,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: ThemeMode::System,
            glass_intensity: 0.8,
            reduce_transparency: false,
        }
    }
}

/// Theme mode enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferencesConfig {
    /// PR stale threshold in hours
    pub pr_stale_threshold_hours: u32,
    /// Whether to store analyzed content history
    pub store_analysis_history: bool,
}

impl Default for PreferencesConfig {
    fn default() -> Self {
        Self {
            pr_stale_threshold_hours: 48,
            store_analysis_history: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        
        assert!(config.integrations.jira.is_none());
        assert_eq!(config.shortcuts.flight_console, "Alt+Space");
        assert_eq!(config.appearance.theme, ThemeMode::System);
        assert_eq!(config.preferences.pr_stale_threshold_hours, 48);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        
        let json = serde_json::to_string(&config).unwrap();
        let parsed: AppConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.shortcuts.flight_console, config.shortcuts.flight_console);
    }

    #[test]
    fn test_threshold_defaults() {
        let thresholds = ThresholdConfig::default();
        
        assert_eq!(thresholds.error_rate_amber, 1.0);
        assert_eq!(thresholds.error_rate_red, 5.0);
        assert_eq!(thresholds.latency_amber_ms, 500);
        assert_eq!(thresholds.latency_red_ms, 1000);
    }

    #[test]
    fn test_git_provider_serialization() {
        let config = GitConfig {
            provider: GitProviderType::Bitbucket,
            base_url: None,
            workspace: Some("myworkspace".to_string()),
            repositories: vec!["repo1".to_string()],
        };
        
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"provider\":\"bitbucket\""));
    }

    #[test]
    fn test_jira_config() {
        let jira = JiraConfig {
            base_url: "https://mycompany.atlassian.net".to_string(),
            default_project: Some("PROJ".to_string()),
            username: Some("user@example.com".to_string()),
        };

        let json = serde_json::to_string(&jira).unwrap();
        let parsed: JiraConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.base_url, jira.base_url);
        assert_eq!(parsed.default_project, jira.default_project);
    }
}
