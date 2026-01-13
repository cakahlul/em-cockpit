//! The EM Cockpit - Desktop app for Engineering Managers
//!
//! A spotlight-style cockpit application that lets Engineering Managers
//! command their work from anywhere (tickets, PRs, incidents, specs)
//! via a global hotkey, without opening a browser.

// Core modules
pub mod commands;
pub mod core;
pub mod integrations;
pub mod security;
pub mod services;
pub mod system;

// Re-export commonly used types
pub use core::{AppConfig, CockpitError};
pub use security::{CredentialError, CredentialManager};
pub use services::{CacheConfig, CacheError, CacheService};
pub use system::{HotkeyError, HotkeyManager, Shortcut, TrayError, TrayManager, TrayState};
pub use integrations::{
    traits::{IntegrationError, Ticket, PullRequest, Incident, Metric},
    JiraClient, GitProvider, GeminiClient, GrafanaClient,
};

/// Application state shared across the Tauri application
pub struct AppState {
    pub credential_manager: CredentialManager,
    pub cache_service: CacheService,
    pub config: AppConfig,
}

impl AppState {
    /// Create a new application state
    pub fn new(cache_path: std::path::PathBuf) -> Result<Self, CockpitError> {
        let cache_service = CacheService::new(cache_path)
            .map_err(|e| CockpitError::Cache(e))?;
        
        Ok(Self {
            credential_manager: CredentialManager::new(),
            cache_service,
            config: AppConfig::default(),
        })
    }

    /// Create an in-memory state (for testing)
    pub fn new_in_memory() -> Result<Self, CockpitError> {
        let cache_service = CacheService::new_in_memory()
            .map_err(|e| CockpitError::Cache(e))?;
        
        Ok(Self {
            credential_manager: CredentialManager::new(),
            cache_service,
            config: AppConfig::default(),
        })
    }
}

/// Run the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation_in_memory() {
        let state = AppState::new_in_memory();
        assert!(state.is_ok());
    }

    #[test]
    fn test_app_state_has_default_config() {
        let state = AppState::new_in_memory().unwrap();
        assert_eq!(state.config.shortcuts.flight_console, "Alt+Space");
    }
}
