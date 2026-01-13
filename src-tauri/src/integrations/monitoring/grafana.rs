//! Grafana/Monitoring Client
//!
//! Implements MetricsRepository for Grafana-compatible APIs.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::integrations::traits::{
    Incident, IncidentStatus, IntegrationError, Metric, MetricsRepository, Severity,
};

/// Monitoring platform configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub platform: String,
    pub base_url: String,
    #[serde(skip)]
    pub api_key: Option<String>,
    pub services: Vec<ServiceConfig>,
}

/// Service configuration for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub dashboard_id: Option<String>,
    pub thresholds: ThresholdConfig,
}

/// Threshold configuration for alert states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    pub error_rate_amber: f64,
    pub error_rate_red: f64,
    pub latency_amber_ms: u64,
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

impl MonitoringConfig {
    pub fn grafana(base_url: &str) -> Self {
        Self {
            platform: "grafana".to_string(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: None,
            services: Vec::new(),
        }
    }

    pub fn with_api_key(mut self, key: &str) -> Self {
        self.api_key = Some(key.to_string());
        self
    }

    pub fn with_service(mut self, name: &str, thresholds: ThresholdConfig) -> Self {
        self.services.push(ServiceConfig {
            name: name.to_string(),
            dashboard_id: None,
            thresholds,
        });
        self
    }
}

/// Grafana client
#[derive(Debug)]
pub struct GrafanaClient {
    config: MonitoringConfig,
    http_client: Client,
}

impl GrafanaClient {
    pub fn new(config: MonitoringConfig) -> Result<Self, IntegrationError> {
        if config.api_key.is_none() {
            return Err(IntegrationError::ConfigError(
                "Grafana API key is required".to_string(),
            ));
        }

        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| IntegrationError::Network(e.to_string()))?;

        Ok(Self { config, http_client })
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.config.api_key.as_deref().unwrap_or(""))
    }

    fn severity_from_labels(&self, labels: &serde_json::Value) -> Severity {
        if let Some(severity) = labels.get("severity").and_then(|v| v.as_str()) {
            match severity.to_lowercase().as_str() {
                "critical" | "p1" => Severity::Critical,
                "high" | "p2" => Severity::High,
                "medium" | "warning" | "p3" => Severity::Medium,
                "low" | "info" | "p4" => Severity::Low,
                _ => Severity::Medium,
            }
        } else {
            Severity::Medium
        }
    }
}

#[async_trait]
impl MetricsRepository for GrafanaClient {
    async fn get_metrics(&self, service: &str) -> Result<Vec<Metric>, IntegrationError> {
        // Query Grafana's datasource proxy for Prometheus metrics
        let url = format!(
            "{}/api/datasources/proxy/1/api/v1/query",
            self.config.base_url
        );

        // Query for error rate and latency
        let queries = vec![
            (format!("sum(rate(http_requests_total{{service=\"{}\",status=~\"5..\"}}[5m])) / sum(rate(http_requests_total{{service=\"{}\"}}[5m])) * 100", service, service), "error_rate", "%"),
            (format!("histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket{{service=\"{}\"}}[5m])) by (le)) * 1000", service), "latency_p95", "ms"),
        ];

        let mut metrics = Vec::new();

        for (query, name, unit) in queries {
            let response = self.http_client
                .get(&url)
                .header("Authorization", self.auth_header())
                .query(&[("query", &query)])
                .send()
                .await?;

            if response.status().as_u16() == 200 {
                let result: PrometheusResponse = response.json().await
                    .map_err(|e| IntegrationError::ParseError(e.to_string()))?;

                if let Some(first_result) = result.data.result.first() {
                    if let Some(value) = first_result.value.get(1).and_then(|v| v.as_str()) {
                        if let Ok(val) = value.parse::<f64>() {
                            metrics.push(Metric {
                                name: name.to_string(),
                                value: val,
                                unit: unit.to_string(),
                                timestamp: Utc::now(),
                            });
                        }
                    }
                }
            }
        }

        Ok(metrics)
    }

    async fn get_incidents(&self) -> Result<Vec<Incident>, IntegrationError> {
        // Query Grafana Alerting API
        let url = format!("{}/api/alertmanager/grafana/api/v2/alerts", self.config.base_url);

        let response = self.http_client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        match response.status().as_u16() {
            200 => {
                let alerts: Vec<GrafanaAlert> = response.json().await
                    .map_err(|e| IntegrationError::ParseError(e.to_string()))?;

                let incidents: Vec<Incident> = alerts
                    .into_iter()
                    .filter(|a| a.status.state == "active")
                    .map(|a| {
                        let service = a.labels.get("service")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();

                        Incident {
                            id: a.fingerprint.clone(),
                            service,
                            severity: self.severity_from_labels(&a.labels),
                            status: IncidentStatus::Firing,
                            started_at: a.starts_at,
                            resolved_at: None,
                            description: a.annotations.get("description")
                                .and_then(|v| v.as_str())
                                .unwrap_or(&a.fingerprint)
                                .to_string(),
                            runbook_url: a.annotations.get("runbook_url")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                        }
                    })
                    .collect();

                Ok(incidents)
            }
            401 => Err(IntegrationError::Auth("Invalid API key".to_string())),
            429 => Err(IntegrationError::RateLimit),
            status => {
                let body = response.text().await.unwrap_or_default();
                Err(IntegrationError::ApiError(format!("Status {}: {}", status, body)))
            }
        }
    }
}

// ===== Prometheus/Grafana API Types =====

#[derive(Debug, Deserialize)]
struct PrometheusResponse {
    data: PrometheusData,
}

#[derive(Debug, Deserialize)]
struct PrometheusData {
    result: Vec<PrometheusResult>,
}

#[derive(Debug, Deserialize)]
struct PrometheusResult {
    value: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct GrafanaAlert {
    fingerprint: String,
    labels: serde_json::Value,
    annotations: serde_json::Value,
    #[serde(rename = "startsAt")]
    starts_at: DateTime<Utc>,
    status: GrafanaAlertStatus,
}

#[derive(Debug, Deserialize)]
struct GrafanaAlertStatus {
    state: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitoring_config_creation() {
        let config = MonitoringConfig::grafana("https://grafana.example.com")
            .with_api_key("test-key")
            .with_service("api-service", ThresholdConfig::default());

        assert_eq!(config.platform, "grafana");
        assert_eq!(config.base_url, "https://grafana.example.com");
        assert_eq!(config.services.len(), 1);
    }

    #[test]
    fn test_config_trims_trailing_slash() {
        let config = MonitoringConfig::grafana("https://grafana.example.com/");
        assert_eq!(config.base_url, "https://grafana.example.com");
    }

    #[test]
    fn test_client_requires_api_key() {
        let config = MonitoringConfig::grafana("https://grafana.example.com");
        let result = GrafanaClient::new(config);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IntegrationError::ConfigError(_)));
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
    fn test_severity_parsing() {
        let config = MonitoringConfig::grafana("https://test.com").with_api_key("key");
        let client = GrafanaClient::new(config).unwrap();

        let labels = serde_json::json!({"severity": "critical"});
        assert_eq!(client.severity_from_labels(&labels), Severity::Critical);

        let labels = serde_json::json!({"severity": "warning"});
        assert_eq!(client.severity_from_labels(&labels), Severity::Medium);

        let labels = serde_json::json!({});
        assert_eq!(client.severity_from_labels(&labels), Severity::Medium);
    }

    #[test]
    fn test_auth_header_format() {
        let config = MonitoringConfig::grafana("https://test.com").with_api_key("my-token");
        let client = GrafanaClient::new(config).unwrap();

        let header = client.auth_header();
        assert_eq!(header, "Bearer my-token");
    }
}
