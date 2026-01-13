//! Incident Commands
//!
//! Tauri commands for incident monitoring and alerts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::commands::search::CommandError;

/// Incident filter parameters
#[derive(Debug, Clone, Deserialize)]
pub struct IncidentFilterParams {
    #[serde(default)]
    pub services: Vec<String>,
    #[serde(rename = "minSeverity")]
    pub min_severity: Option<String>,
    #[serde(rename = "activeOnly", default = "default_true")]
    pub active_only: bool,
}

fn default_true() -> bool {
    true
}

/// Incident summary response
#[derive(Debug, Clone, Serialize)]
pub struct IncidentSummaryResponse {
    #[serde(rename = "totalActive")]
    pub total_active: usize,
    #[serde(rename = "criticalCount")]
    pub critical_count: usize,
    #[serde(rename = "highCount")]
    pub high_count: usize,
    #[serde(rename = "mediumCount")]
    pub medium_count: usize,
    #[serde(rename = "lowCount")]
    pub low_count: usize,
    #[serde(rename = "byService")]
    pub by_service: HashMap<String, usize>,
    #[serde(rename = "trayState")]
    pub tray_state: String,
    #[serde(rename = "mostSevere")]
    pub most_severe: Option<String>,
}

/// Incident item for list response
#[derive(Debug, Clone, Serialize)]
pub struct IncidentItemDto {
    pub id: String,
    pub service: String,
    pub severity: String,
    #[serde(rename = "severityLevel")]
    pub severity_level: u8, // For sorting: 4=critical, 3=high, 2=medium, 1=low
    pub status: String,
    pub description: String,
    #[serde(rename = "startedAt")]
    pub started_at: String,
    #[serde(rename = "resolvedAt")]
    pub resolved_at: Option<String>,
    #[serde(rename = "durationMins")]
    pub duration_mins: i64,
    #[serde(rename = "runbookUrl")]
    pub runbook_url: Option<String>,
}

/// Get incident summary
#[tauri::command]
pub async fn get_incident_summary() -> Result<IncidentSummaryResponse, CommandError> {
    // TODO: Wire up to actual IncidentMonitor service
    Ok(IncidentSummaryResponse {
        total_active: 0,
        critical_count: 0,
        high_count: 0,
        medium_count: 0,
        low_count: 0,
        by_service: HashMap::new(),
        tray_state: "green".to_string(),
        most_severe: None,
    })
}

/// Get list of incidents
#[tauri::command]
pub async fn get_incidents(
    params: Option<IncidentFilterParams>,
) -> Result<Vec<IncidentItemDto>, CommandError> {
    // TODO: Wire up to actual IncidentMonitor service
    Ok(vec![])
}

/// Get active critical incidents
#[tauri::command]
pub async fn get_critical_incidents() -> Result<Vec<IncidentItemDto>, CommandError> {
    // TODO: Wire up to actual IncidentMonitor service
    Ok(vec![])
}

/// Check if there are any critical incidents
#[tauri::command]
pub async fn has_critical_incidents() -> Result<bool, CommandError> {
    // TODO: Wire up to actual IncidentMonitor service
    Ok(false)
}

/// Get current tray state based on incidents
#[tauri::command]
pub async fn get_incident_tray_state() -> Result<String, CommandError> {
    Ok("green".to_string())
}

/// Refresh incident data (bypass cache)
#[tauri::command]
pub async fn refresh_incidents() -> Result<IncidentSummaryResponse, CommandError> {
    get_incident_summary().await
}

/// Acknowledge an incident (mark as seen)
#[tauri::command]
pub async fn acknowledge_incident(incident_id: String) -> Result<(), CommandError> {
    if incident_id.is_empty() {
        return Err(CommandError::validation("Incident ID is required"));
    }
    // TODO: Implement incident acknowledgment
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incident_filter_defaults() {
        let json = r#"{"services": []}"#;
        let params: IncidentFilterParams = serde_json::from_str(json).unwrap();
        
        assert!(params.active_only);
        assert!(params.min_severity.is_none());
    }

    #[test]
    fn test_incident_summary_serialization() {
        let summary = IncidentSummaryResponse {
            total_active: 3,
            critical_count: 1,
            high_count: 1,
            medium_count: 1,
            low_count: 0,
            by_service: HashMap::from([("api".to_string(), 2)]),
            tray_state: "red".to_string(),
            most_severe: Some("critical".to_string()),
        };

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"totalActive\":3"));
        assert!(json.contains("\"criticalCount\":1"));
        assert!(json.contains("\"trayState\":\"red\""));
    }

    #[test]
    fn test_incident_item_serialization() {
        let incident = IncidentItemDto {
            id: "inc-1".to_string(),
            service: "api".to_string(),
            severity: "critical".to_string(),
            severity_level: 4,
            status: "firing".to_string(),
            description: "High error rate".to_string(),
            started_at: "2024-01-01T00:00:00Z".to_string(),
            resolved_at: None,
            duration_mins: 30,
            runbook_url: Some("https://runbook.com/api".to_string()),
        };

        let json = serde_json::to_string(&incident).unwrap();
        assert!(json.contains("\"severityLevel\":4"));
        assert!(json.contains("\"durationMins\":30"));
    }

    #[tokio::test]
    async fn test_get_incident_summary() {
        let result = get_incident_summary().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_acknowledge_incident_empty_id() {
        let result = acknowledge_incident("".to_string()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "VALIDATION_ERROR");
    }

    #[tokio::test]
    async fn test_acknowledge_incident_valid() {
        let result = acknowledge_incident("inc-123".to_string()).await;
        assert!(result.is_ok());
    }
}
