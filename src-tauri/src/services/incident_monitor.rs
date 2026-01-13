//! Incident Monitor Service
//!
//! Monitors incidents from monitoring platforms,
//! providing severity-based filtering and tray state updates.

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::integrations::traits::{Incident, IncidentStatus, IntegrationError, MetricsRepository, Severity};
use crate::services::CacheService;
use crate::system::TrayState;

/// Summary of incident status
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IncidentSummary {
    /// Total active incidents
    pub total_active: usize,
    /// Critical severity count
    pub critical_count: usize,
    /// High severity count
    pub high_count: usize,
    /// Medium severity count
    pub medium_count: usize,
    /// Low severity count
    pub low_count: usize,
    /// Incidents by service
    pub by_service: HashMap<String, usize>,
    /// Most severe incident
    pub most_severe: Option<Severity>,
    /// Tray state based on incidents
    pub tray_state: TrayState,
    /// Longest active incident duration in minutes
    pub longest_duration_mins: Option<i64>,
}

impl IncidentSummary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate tray state from incidents
    pub fn calculate_tray_state(incidents: &[Incident]) -> TrayState {
        let has_critical = incidents.iter().any(|i| i.severity == Severity::Critical);
        let has_high = incidents.iter().any(|i| i.severity == Severity::High);
        
        if has_critical || has_high {
            TrayState::Red
        } else if !incidents.is_empty() {
            TrayState::Amber
        } else {
            TrayState::Green
        }
    }

    /// Get most severe severity from list
    pub fn get_most_severe(incidents: &[Incident]) -> Option<Severity> {
        incidents.iter().map(|i| i.severity).max()
    }
}

/// Incident filter options
#[derive(Debug, Clone, Default)]
pub struct IncidentFilter {
    pub min_severity: Option<Severity>,
    pub services: Vec<String>,
    pub active_only: bool,
}

impl IncidentFilter {
    pub fn new() -> Self {
        Self {
            active_only: true,
            ..Default::default()
        }
    }

    pub fn with_min_severity(mut self, severity: Severity) -> Self {
        self.min_severity = Some(severity);
        self
    }

    pub fn with_services(mut self, services: Vec<String>) -> Self {
        self.services = services;
        self
    }

    pub fn include_resolved(mut self) -> Self {
        self.active_only = false;
        self
    }

    pub fn matches(&self, incident: &Incident) -> bool {
        // Check severity
        if let Some(min_sev) = self.min_severity {
            if incident.severity < min_sev {
                return false;
            }
        }

        // Check services
        if !self.services.is_empty() && !self.services.contains(&incident.service) {
            return false;
        }

        // Check active status
        if self.active_only && incident.status != IncidentStatus::Firing {
            return false;
        }

        true
    }
}

/// Configuration for incident monitoring
#[derive(Debug, Clone)]
pub struct IncidentMonitorConfig {
    pub refresh_interval: Duration,
    pub services: Vec<String>,
    pub alert_on_severity: Severity,
}

impl Default for IncidentMonitorConfig {
    fn default() -> Self {
        Self {
            refresh_interval: Duration::seconds(30),
            services: Vec::new(),
            alert_on_severity: Severity::High,
        }
    }
}

impl IncidentMonitorConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_services(mut self, services: Vec<String>) -> Self {
        self.services = services;
        self
    }

    pub fn alert_on(mut self, severity: Severity) -> Self {
        self.alert_on_severity = severity;
        self
    }
}

/// Incident Monitor Service
pub struct IncidentMonitor<M: MetricsRepository> {
    metrics_repo: Arc<M>,
    config: IncidentMonitorConfig,
    cache: Option<Arc<CacheService>>,
}

impl<M: MetricsRepository> IncidentMonitor<M> {
    pub fn new(metrics_repo: Arc<M>, config: IncidentMonitorConfig) -> Self {
        Self {
            metrics_repo,
            config,
            cache: None,
        }
    }

    pub fn with_cache(mut self, cache: Arc<CacheService>) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Get summary of current incidents
    pub async fn get_summary(&self) -> Result<IncidentSummary, IntegrationError> {
        // Check cache
        let cache_key = "incident_summary";
        if let Some(ref cache) = self.cache {
            if let Ok(cached) = cache.get::<IncidentSummary>(cache_key) {
                return Ok(cached);
            }
        }

        let incidents = self.fetch_all_incidents().await?;
        let summary = self.compute_summary(&incidents);

        // Cache result
        if let Some(ref cache) = self.cache {
            let _ = cache.set(cache_key, &summary, self.config.refresh_interval);
        }

        Ok(summary)
    }

    /// Fetch all active incidents
    pub async fn fetch_all_incidents(&self) -> Result<Vec<Incident>, IntegrationError> {
        self.metrics_repo.get_incidents().await
    }

    /// Get filtered incidents
    pub async fn get_incidents(&self, filter: &IncidentFilter) -> Result<Vec<Incident>, IntegrationError> {
        let incidents = self.fetch_all_incidents().await?;
        Ok(incidents.into_iter().filter(|i| filter.matches(i)).collect())
    }

    /// Get critical incidents that should trigger alerts
    pub async fn get_alertable_incidents(&self) -> Result<Vec<Incident>, IntegrationError> {
        let filter = IncidentFilter::new()
            .with_min_severity(self.config.alert_on_severity);
        self.get_incidents(&filter).await
    }

    /// Check if any incident requires immediate attention
    pub async fn has_critical_incidents(&self) -> Result<bool, IntegrationError> {
        let filter = IncidentFilter::new()
            .with_min_severity(Severity::Critical);
        let incidents = self.get_incidents(&filter).await?;
        Ok(!incidents.is_empty())
    }

    /// Get current tray state based on incidents
    pub async fn get_tray_state(&self) -> Result<TrayState, IntegrationError> {
        let incidents = self.fetch_all_incidents().await?;
        Ok(IncidentSummary::calculate_tray_state(&incidents))
    }

    fn compute_summary(&self, incidents: &[Incident]) -> IncidentSummary {
        let active: Vec<&Incident> = incidents
            .iter()
            .filter(|i| i.status == IncidentStatus::Firing)
            .collect();

        let mut by_service: HashMap<String, usize> = HashMap::new();
        let mut critical_count = 0;
        let mut high_count = 0;
        let mut medium_count = 0;
        let mut low_count = 0;

        for incident in &active {
            *by_service.entry(incident.service.clone()).or_default() += 1;
            match incident.severity {
                Severity::Critical => critical_count += 1,
                Severity::High => high_count += 1,
                Severity::Medium => medium_count += 1,
                Severity::Low => low_count += 1,
            }
        }

        let now = Utc::now();
        let longest_duration_mins = active
            .iter()
            .map(|i| now.signed_duration_since(i.started_at).num_minutes())
            .max();

        IncidentSummary {
            total_active: active.len(),
            critical_count,
            high_count,
            medium_count,
            low_count,
            by_service,
            most_severe: IncidentSummary::get_most_severe(incidents),
            tray_state: IncidentSummary::calculate_tray_state(incidents),
            longest_duration_mins,
        }
    }
}

// Debug implementation
impl<M: MetricsRepository> std::fmt::Debug for IncidentMonitor<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IncidentMonitor")
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::traits::Metric;
    use std::sync::Mutex;

    struct MockMetricsRepo {
        incidents: Mutex<Vec<Incident>>,
    }

    impl MockMetricsRepo {
        fn new(incidents: Vec<Incident>) -> Self {
            Self {
                incidents: Mutex::new(incidents),
            }
        }
    }

    #[async_trait]
    impl MetricsRepository for MockMetricsRepo {
        async fn get_metrics(&self, _service: &str) -> Result<Vec<Metric>, IntegrationError> {
            Ok(vec![])
        }

        async fn get_incidents(&self) -> Result<Vec<Incident>, IntegrationError> {
            Ok(self.incidents.lock().unwrap().clone())
        }
    }

    fn create_test_incident(id: &str, service: &str, severity: Severity) -> Incident {
        Incident {
            id: id.to_string(),
            service: service.to_string(),
            severity,
            status: IncidentStatus::Firing,
            started_at: Utc::now() - Duration::minutes(30),
            resolved_at: None,
            description: format!("Test incident {}", id),
            runbook_url: None,
        }
    }

    #[test]
    fn test_incident_summary_calculate_tray_state() {
        let critical = vec![create_test_incident("1", "svc", Severity::Critical)];
        assert_eq!(IncidentSummary::calculate_tray_state(&critical), TrayState::Red);

        let high = vec![create_test_incident("1", "svc", Severity::High)];
        assert_eq!(IncidentSummary::calculate_tray_state(&high), TrayState::Red);

        let medium = vec![create_test_incident("1", "svc", Severity::Medium)];
        assert_eq!(IncidentSummary::calculate_tray_state(&medium), TrayState::Amber);

        let empty: Vec<Incident> = vec![];
        assert_eq!(IncidentSummary::calculate_tray_state(&empty), TrayState::Green);
    }

    #[test]
    fn test_incident_summary_get_most_severe() {
        let incidents = vec![
            create_test_incident("1", "svc", Severity::Low),
            create_test_incident("2", "svc", Severity::Critical),
            create_test_incident("3", "svc", Severity::Medium),
        ];
        assert_eq!(IncidentSummary::get_most_severe(&incidents), Some(Severity::Critical));

        let empty: Vec<Incident> = vec![];
        assert_eq!(IncidentSummary::get_most_severe(&empty), None);
    }

    #[test]
    fn test_incident_filter_matches() {
        let incident = create_test_incident("1", "api-service", Severity::High);

        // Default filter (active only)
        let filter = IncidentFilter::new();
        assert!(filter.matches(&incident));

        // Min severity filter
        let filter = IncidentFilter::new().with_min_severity(Severity::Critical);
        assert!(!filter.matches(&incident)); // High < Critical

        let filter = IncidentFilter::new().with_min_severity(Severity::Medium);
        assert!(filter.matches(&incident)); // High >= Medium

        // Service filter
        let filter = IncidentFilter::new().with_services(vec!["api-service".to_string()]);
        assert!(filter.matches(&incident));

        let filter = IncidentFilter::new().with_services(vec!["other-service".to_string()]);
        assert!(!filter.matches(&incident));
    }

    #[test]
    fn test_incident_filter_resolved() {
        let mut incident = create_test_incident("1", "svc", Severity::High);
        incident.status = IncidentStatus::Resolved;

        let filter = IncidentFilter::new(); // active only
        assert!(!filter.matches(&incident));

        let filter = IncidentFilter::new().include_resolved();
        assert!(filter.matches(&incident));
    }

    #[test]
    fn test_incident_monitor_config_builder() {
        let config = IncidentMonitorConfig::new()
            .with_services(vec!["svc1".to_string()])
            .alert_on(Severity::Medium);

        assert_eq!(config.services.len(), 1);
        assert_eq!(config.alert_on_severity, Severity::Medium);
    }

    #[tokio::test]
    async fn test_fetch_all_incidents() {
        let incidents = vec![
            create_test_incident("1", "svc1", Severity::High),
            create_test_incident("2", "svc2", Severity::Low),
        ];
        let repo = Arc::new(MockMetricsRepo::new(incidents));
        let monitor = IncidentMonitor::new(repo, IncidentMonitorConfig::new());

        let result = monitor.fetch_all_incidents().await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_get_filtered_incidents() {
        let incidents = vec![
            create_test_incident("1", "svc1", Severity::Critical),
            create_test_incident("2", "svc1", Severity::Low),
            create_test_incident("3", "svc2", Severity::High),
        ];
        let repo = Arc::new(MockMetricsRepo::new(incidents));
        let monitor = IncidentMonitor::new(repo, IncidentMonitorConfig::new());

        let filter = IncidentFilter::new().with_min_severity(Severity::High);
        let result = monitor.get_incidents(&filter).await.unwrap();
        
        assert_eq!(result.len(), 2); // Critical and High
    }

    #[tokio::test]
    async fn test_has_critical_incidents() {
        let incidents = vec![
            create_test_incident("1", "svc", Severity::High),
        ];
        let repo = Arc::new(MockMetricsRepo::new(incidents));
        let monitor = IncidentMonitor::new(repo, IncidentMonitorConfig::new());

        assert!(!monitor.has_critical_incidents().await.unwrap());

        let incidents2 = vec![
            create_test_incident("1", "svc", Severity::Critical),
        ];
        let repo2 = Arc::new(MockMetricsRepo::new(incidents2));
        let monitor2 = IncidentMonitor::new(repo2, IncidentMonitorConfig::new());

        assert!(monitor2.has_critical_incidents().await.unwrap());
    }

    #[tokio::test]
    async fn test_get_summary() {
        let incidents = vec![
            create_test_incident("1", "svc1", Severity::Critical),
            create_test_incident("2", "svc1", Severity::High),
            create_test_incident("3", "svc2", Severity::Medium),
        ];
        let repo = Arc::new(MockMetricsRepo::new(incidents));
        let monitor = IncidentMonitor::new(repo, IncidentMonitorConfig::new());

        let summary = monitor.get_summary().await.unwrap();
        
        assert_eq!(summary.total_active, 3);
        assert_eq!(summary.critical_count, 1);
        assert_eq!(summary.high_count, 1);
        assert_eq!(summary.medium_count, 1);
        assert_eq!(summary.by_service.get("svc1"), Some(&2));
        assert_eq!(summary.most_severe, Some(Severity::Critical));
        assert_eq!(summary.tray_state, TrayState::Red);
    }

    #[tokio::test]
    async fn test_get_tray_state() {
        let incidents = vec![
            create_test_incident("1", "svc", Severity::Medium),
        ];
        let repo = Arc::new(MockMetricsRepo::new(incidents));
        let monitor = IncidentMonitor::new(repo, IncidentMonitorConfig::new());

        let state = monitor.get_tray_state().await.unwrap();
        assert_eq!(state, TrayState::Amber);
    }
}
