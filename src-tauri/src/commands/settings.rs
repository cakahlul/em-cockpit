//! Settings Commands
//!
//! Tauri commands for application configuration and credentials.

use serde::{Deserialize, Serialize};

use crate::commands::search::CommandError;

/// Integration configuration for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfigDto {
    pub jira: Option<JiraConfigDto>,
    pub git: Option<GitConfigDto>,
    pub gemini: Option<GeminiConfigDto>,
    pub grafana: Option<GrafanaConfigDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraConfigDto {
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    pub username: String,
    #[serde(rename = "defaultProject")]
    pub default_project: Option<String>,
    #[serde(rename = "hasToken")]
    pub has_token: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfigDto {
    pub provider: String,
    #[serde(rename = "baseUrl")]
    pub base_url: Option<String>,
    pub workspace: Option<String>,
    pub username: String,
    pub repositories: Vec<String>,
    #[serde(rename = "hasToken")]
    pub has_token: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiConfigDto {
    pub model: String,
    #[serde(rename = "hasApiKey")]
    pub has_api_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrafanaConfigDto {
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    pub services: Vec<String>,
    #[serde(rename = "hasApiKey")]
    pub has_api_key: bool,
}

/// Shortcut configuration for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfigDto {
    #[serde(rename = "flightConsole")]
    pub flight_console: String,
    #[serde(rename = "radarPanel")]
    pub radar_panel: String,
    #[serde(rename = "incidentRadar")]
    pub incident_radar: String,
}

/// Appearance configuration for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfigDto {
    pub theme: String,
    #[serde(rename = "glassIntensity")]
    pub glass_intensity: f32,
    #[serde(rename = "reduceTransparency")]
    pub reduce_transparency: bool,
}

/// Full settings response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsResponse {
    pub integrations: IntegrationConfigDto,
    pub shortcuts: ShortcutConfigDto,
    pub appearance: AppearanceConfigDto,
    #[serde(rename = "prStaleThresholdHours")]
    pub pr_stale_threshold_hours: u32,
}

/// Credential save request
#[derive(Debug, Clone, Deserialize)]
pub struct SaveCredentialRequest {
    #[serde(rename = "credentialType")]
    pub credential_type: String,
    pub value: String,
}

/// Get all settings
#[tauri::command]
pub async fn get_settings() -> Result<SettingsResponse, CommandError> {
    Ok(SettingsResponse {
        integrations: IntegrationConfigDto {
            jira: None,
            git: None,
            gemini: None,
            grafana: None,
        },
        shortcuts: ShortcutConfigDto {
            flight_console: "Alt+Space".to_string(),
            radar_panel: "Ctrl+2".to_string(),
            incident_radar: "Ctrl+3".to_string(),
        },
        appearance: AppearanceConfigDto {
            theme: "system".to_string(),
            glass_intensity: 0.8,
            reduce_transparency: false,
        },
        pr_stale_threshold_hours: 48,
    })
}

/// Save Jira configuration
#[tauri::command]
pub async fn save_jira_config(config: JiraConfigDto) -> Result<(), CommandError> {
    if config.base_url.is_empty() {
        return Err(CommandError::validation("Jira base URL is required"));
    }
    if config.username.is_empty() {
        return Err(CommandError::validation("Jira username is required"));
    }
    // TODO: Wire up to actual config storage
    log::info!("Saving Jira config for: {}", config.base_url);
    Ok(())
}

/// Save Git configuration
#[tauri::command]
pub async fn save_git_config(config: GitConfigDto) -> Result<(), CommandError> {
    if config.username.is_empty() {
        return Err(CommandError::validation("Git username is required"));
    }
    // TODO: Wire up to actual config storage
    log::info!("Saving Git config for provider: {}", config.provider);
    Ok(())
}

/// Save Gemini configuration
#[tauri::command]
pub async fn save_gemini_config(config: GeminiConfigDto) -> Result<(), CommandError> {
    if config.model.is_empty() {
        return Err(CommandError::validation("Gemini model is required"));
    }
    // TODO: Wire up to actual config storage
    log::info!("Saving Gemini config for model: {}", config.model);
    Ok(())
}

/// Save Grafana configuration
#[tauri::command]
pub async fn save_grafana_config(config: GrafanaConfigDto) -> Result<(), CommandError> {
    if config.base_url.is_empty() {
        return Err(CommandError::validation("Grafana base URL is required"));
    }
    // TODO: Wire up to actual config storage
    log::info!("Saving Grafana config for: {}", config.base_url);
    Ok(())
}

/// Save a credential securely
#[tauri::command]
pub async fn save_credential(request: SaveCredentialRequest) -> Result<(), CommandError> {
    if request.value.is_empty() {
        return Err(CommandError::validation("Credential value cannot be empty"));
    }

    let valid_types = ["jira_token", "git_token", "gemini_api_key", "grafana_api_key"];
    if !valid_types.contains(&request.credential_type.as_str()) {
        return Err(CommandError::validation(&format!(
            "Invalid credential type: {}",
            request.credential_type
        )));
    }

    // TODO: Wire up to CredentialManager
    log::info!("Saving credential: {}", request.credential_type);
    Ok(())
}

/// Delete a credential
#[tauri::command]
pub async fn delete_credential(credential_type: String) -> Result<(), CommandError> {
    if credential_type.is_empty() {
        return Err(CommandError::validation("Credential type is required"));
    }
    // TODO: Wire up to CredentialManager
    log::info!("Deleting credential: {}", credential_type);
    Ok(())
}

/// Check if a credential exists
#[tauri::command]
pub async fn has_credential(credential_type: String) -> Result<bool, CommandError> {
    // TODO: Wire up to CredentialManager
    Ok(false)
}

/// Save shortcut configuration
#[tauri::command]
pub async fn save_shortcuts(shortcuts: ShortcutConfigDto) -> Result<(), CommandError> {
    if shortcuts.flight_console.is_empty() {
        return Err(CommandError::validation("Flight Console shortcut is required"));
    }
    // TODO: Wire up to config storage and HotkeyManager
    log::info!("Saving shortcuts");
    Ok(())
}

/// Save appearance configuration
#[tauri::command]
pub async fn save_appearance(appearance: AppearanceConfigDto) -> Result<(), CommandError> {
    // Validate glass intensity
    if appearance.glass_intensity < 0.0 || appearance.glass_intensity > 1.0 {
        return Err(CommandError::validation(
            "Glass intensity must be between 0 and 1",
        ));
    }
    // TODO: Wire up to config storage
    log::info!("Saving appearance: theme={}", appearance.theme);
    Ok(())
}

/// Test integration connection
#[tauri::command]
pub async fn test_connection(integration: String) -> Result<bool, CommandError> {
    let valid = ["jira", "git", "gemini", "grafana"];
    if !valid.contains(&integration.as_str()) {
        return Err(CommandError::validation(&format!(
            "Invalid integration: {}",
            integration
        )));
    }
    // TODO: Actually test connection
    Ok(true)
}

/// Execute panic wipe (delete all credentials)
#[tauri::command]
pub async fn panic_wipe() -> Result<usize, CommandError> {
    // TODO: Wire up to CredentialManager.panic_wipe()
    log::warn!("PANIC WIPE requested!");
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jira_config_dto_serialization() {
        let config = JiraConfigDto {
            base_url: "https://company.atlassian.net".to_string(),
            username: "user@example.com".to_string(),
            default_project: Some("PROJ".to_string()),
            has_token: true,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"baseUrl\":"));
        assert!(json.contains("\"hasToken\":true"));
    }

    #[test]
    fn test_shortcut_config_serialization() {
        let config = ShortcutConfigDto {
            flight_console: "Alt+Space".to_string(),
            radar_panel: "Ctrl+2".to_string(),
            incident_radar: "Ctrl+3".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"flightConsole\":"));
    }

    #[tokio::test]
    async fn test_get_settings() {
        let result = get_settings().await;
        assert!(result.is_ok());
        
        let settings = result.unwrap();
        assert_eq!(settings.shortcuts.flight_console, "Alt+Space");
        assert_eq!(settings.pr_stale_threshold_hours, 48);
    }

    #[tokio::test]
    async fn test_save_jira_config_validation() {
        let config = JiraConfigDto {
            base_url: "".to_string(),
            username: "user".to_string(),
            default_project: None,
            has_token: false,
        };

        let result = save_jira_config(config).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "VALIDATION_ERROR");
    }

    #[tokio::test]
    async fn test_save_credential_validation() {
        let request = SaveCredentialRequest {
            credential_type: "jira_token".to_string(),
            value: "".to_string(),
        };

        let result = save_credential(request).await;
        assert!(result.is_err());

        let request = SaveCredentialRequest {
            credential_type: "invalid_type".to_string(),
            value: "token".to_string(),
        };

        let result = save_credential(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_save_appearance_validation() {
        let appearance = AppearanceConfigDto {
            theme: "dark".to_string(),
            glass_intensity: 1.5, // Invalid
            reduce_transparency: false,
        };

        let result = save_appearance(appearance).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_test_connection() {
        let result = test_connection("jira".to_string()).await;
        assert!(result.is_ok());

        let result = test_connection("invalid".to_string()).await;
        assert!(result.is_err());
    }
}
