//! Credential Manager - Secure credential storage using OS keychain
//!
//! Provides secure storage, retrieval, and deletion of credentials
//! following the Single Responsibility Principle.
//! 
//! In test environments or when keychain is unavailable, falls back
//! to in-memory storage for testing purposes.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Service name used for keychain entries
const SERVICE_NAME: &str = "com.em-cockpit.credentials";

/// Supported credential types for the EM Cockpit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CredentialKey {
    JiraToken,
    GitToken,
    GeminiApiKey,
    GrafanaApiKey,
    ConfluenceToken,
}

impl CredentialKey {
    /// Convert credential key to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            CredentialKey::JiraToken => "jira_token",
            CredentialKey::GitToken => "git_token",
            CredentialKey::GeminiApiKey => "gemini_api_key",
            CredentialKey::GrafanaApiKey => "grafana_api_key",
            CredentialKey::ConfluenceToken => "confluence_token",
        }
    }

    /// Get all credential keys for iteration
    pub fn all() -> Vec<CredentialKey> {
        vec![
            CredentialKey::JiraToken,
            CredentialKey::GitToken,
            CredentialKey::GeminiApiKey,
            CredentialKey::GrafanaApiKey,
            CredentialKey::ConfluenceToken,
        ]
    }
}

/// Errors that can occur during credential operations
#[derive(Error, Debug)]
pub enum CredentialError {
    #[error("Credential not found: {0}")]
    NotFound(String),

    #[error("Failed to store credential: {0}")]
    StoreFailed(String),

    #[error("Failed to delete credential: {0}")]
    DeleteFailed(String),

    #[error("Keychain access denied: {0}")]
    AccessDenied(String),

    #[error("Invalid credential data: {0}")]
    InvalidData(String),
}

/// Storage backend for credentials
enum StorageBackend {
    /// Real OS keychain storage
    Keychain { service_name: String },
    /// In-memory storage (for testing or when keychain unavailable)
    InMemory { store: Arc<RwLock<HashMap<String, String>>> },
}

/// Credential Manager for secure credential storage
///
/// Uses the OS keychain (macOS Keychain, Windows Credential Manager, etc.)
/// to securely store API tokens and other sensitive data.
///
/// # Example
///
/// ```no_run
/// use em_cockpit_lib::security::{CredentialManager, CredentialKey};
///
/// let manager = CredentialManager::new();
/// manager.store(CredentialKey::JiraToken, "my-secret-token").unwrap();
/// let token = manager.retrieve(CredentialKey::JiraToken).unwrap();
/// manager.delete(CredentialKey::JiraToken).unwrap();
/// ```
pub struct CredentialManager {
    backend: StorageBackend,
}

impl CredentialManager {
    /// Create a new CredentialManager instance using OS keychain
    pub fn new() -> Self {
        Self {
            backend: StorageBackend::Keychain {
                service_name: SERVICE_NAME.to_string(),
            },
        }
    }

    /// Create a CredentialManager with in-memory storage (for testing)
    pub fn new_in_memory() -> Self {
        Self {
            backend: StorageBackend::InMemory {
                store: Arc::new(RwLock::new(HashMap::new())),
            },
        }
    }

    /// Store a credential securely
    ///
    /// # Arguments
    /// * `key` - The type of credential to store
    /// * `value` - The credential value (token, API key, etc.)
    ///
    /// # Returns
    /// * `Ok(())` if the credential was stored successfully
    /// * `Err(CredentialError)` if storage failed
    pub fn store(&self, key: CredentialKey, value: &str) -> Result<(), CredentialError> {
        if value.is_empty() {
            return Err(CredentialError::InvalidData(
                "Credential value cannot be empty".to_string(),
            ));
        }

        match &self.backend {
            StorageBackend::Keychain { service_name } => {
                let entry = keyring::Entry::new(service_name, key.as_str())
                    .map_err(|e| CredentialError::StoreFailed(e.to_string()))?;

                entry
                    .set_password(value)
                    .map_err(|e| CredentialError::StoreFailed(e.to_string()))?;
            }
            StorageBackend::InMemory { store } => {
                let mut store = store
                    .write()
                    .map_err(|e| CredentialError::StoreFailed(e.to_string()))?;
                store.insert(key.as_str().to_string(), value.to_string());
            }
        }

        log::debug!("Credential stored successfully: {}", key.as_str());
        Ok(())
    }

    /// Retrieve a credential
    ///
    /// # Arguments
    /// * `key` - The type of credential to retrieve
    ///
    /// # Returns
    /// * `Ok(String)` containing the credential value
    /// * `Err(CredentialError::NotFound)` if the credential doesn't exist
    pub fn retrieve(&self, key: CredentialKey) -> Result<String, CredentialError> {
        match &self.backend {
            StorageBackend::Keychain { service_name } => {
                let entry = keyring::Entry::new(service_name, key.as_str())
                    .map_err(|e| CredentialError::NotFound(e.to_string()))?;

                let password = entry.get_password().map_err(|e| match e {
                    keyring::Error::NoEntry => {
                        CredentialError::NotFound(format!("Credential '{}' not found", key.as_str()))
                    }
                    _ => CredentialError::AccessDenied(e.to_string()),
                })?;

                log::debug!("Credential retrieved successfully: {}", key.as_str());
                Ok(password)
            }
            StorageBackend::InMemory { store } => {
                let store = store
                    .read()
                    .map_err(|e| CredentialError::NotFound(e.to_string()))?;
                
                store
                    .get(key.as_str())
                    .cloned()
                    .ok_or_else(|| {
                        CredentialError::NotFound(format!("Credential '{}' not found", key.as_str()))
                    })
            }
        }
    }

    /// Delete a credential
    ///
    /// # Arguments
    /// * `key` - The type of credential to delete
    ///
    /// # Returns
    /// * `Ok(())` if the credential was deleted successfully
    /// * `Err(CredentialError)` if deletion failed
    pub fn delete(&self, key: CredentialKey) -> Result<(), CredentialError> {
        match &self.backend {
            StorageBackend::Keychain { service_name } => {
                let entry = keyring::Entry::new(service_name, key.as_str())
                    .map_err(|e| CredentialError::DeleteFailed(e.to_string()))?;

                match entry.delete_credential() {
                    Ok(()) => {
                        log::debug!("Credential deleted successfully: {}", key.as_str());
                        Ok(())
                    }
                    Err(keyring::Error::NoEntry) => {
                        log::debug!("Credential already deleted or never existed: {}", key.as_str());
                        Err(CredentialError::NotFound(format!(
                            "Credential '{}' not found",
                            key.as_str()
                        )))
                    }
                    Err(e) => Err(CredentialError::DeleteFailed(e.to_string())),
                }
            }
            StorageBackend::InMemory { store } => {
                let mut store = store
                    .write()
                    .map_err(|e| CredentialError::DeleteFailed(e.to_string()))?;
                
                if store.remove(key.as_str()).is_some() {
                    log::debug!("Credential deleted successfully: {}", key.as_str());
                    Ok(())
                } else {
                    Err(CredentialError::NotFound(format!(
                        "Credential '{}' not found",
                        key.as_str()
                    )))
                }
            }
        }
    }

    /// Check if a credential exists
    ///
    /// # Arguments
    /// * `key` - The type of credential to check
    ///
    /// # Returns
    /// * `true` if the credential exists
    /// * `false` otherwise
    pub fn exists(&self, key: CredentialKey) -> bool {
        self.retrieve(key).is_ok()
    }

    /// Execute panic wipe - delete ALL stored credentials
    ///
    /// This is an emergency function to clear all sensitive data.
    /// It attempts to delete all known credentials, continuing even if some fail.
    ///
    /// # Returns
    /// * `Ok(count)` - Number of credentials successfully deleted
    pub fn panic_wipe(&self) -> Result<usize, CredentialError> {
        let mut deleted_count = 0;

        for key in CredentialKey::all() {
            match self.delete(key) {
                Ok(()) => {
                    deleted_count += 1;
                    log::info!("Panic wipe: deleted {}", key.as_str());
                }
                Err(CredentialError::NotFound(_)) => {
                    log::debug!("Panic wipe: {} already gone", key.as_str());
                }
                Err(e) => {
                    log::error!("Panic wipe: failed to delete {}: {}", key.as_str(), e);
                }
            }
        }

        log::warn!("PANIC WIPE COMPLETED: {} credentials deleted", deleted_count);
        Ok(deleted_count)
    }
}

impl Default for CredentialManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a test credential manager with in-memory storage
    fn test_manager() -> CredentialManager {
        CredentialManager::new_in_memory()
    }

    #[test]
    fn test_store_credential_successfully() {
        let manager = test_manager();
        let result = manager.store(CredentialKey::JiraToken, "test-token-123");
        assert!(result.is_ok(), "Should store credential successfully");
    }

    #[test]
    fn test_retrieve_credential_matches_stored_value() {
        let manager = test_manager();
        let test_value = "my-secret-token-xyz";
        
        manager.store(CredentialKey::JiraToken, test_value).unwrap();
        let retrieved = manager.retrieve(CredentialKey::JiraToken).unwrap();
        
        assert_eq!(retrieved, test_value, "Retrieved value should match stored value");
    }

    #[test]
    fn test_delete_credential_removes_from_storage() {
        let manager = test_manager();
        
        manager.store(CredentialKey::GitToken, "token-to-delete").unwrap();
        assert!(manager.exists(CredentialKey::GitToken), "Credential should exist after store");
        
        manager.delete(CredentialKey::GitToken).unwrap();
        assert!(!manager.exists(CredentialKey::GitToken), "Credential should not exist after delete");
    }

    #[test]
    fn test_retrieve_nonexistent_credential_returns_not_found() {
        let manager = test_manager();
        
        let result = manager.retrieve(CredentialKey::GeminiApiKey);
        
        assert!(result.is_err(), "Should return error for nonexistent credential");
        assert!(
            matches!(result.unwrap_err(), CredentialError::NotFound(_)),
            "Error should be NotFound"
        );
    }

    #[test]
    fn test_store_empty_credential_returns_error() {
        let manager = test_manager();
        
        let result = manager.store(CredentialKey::JiraToken, "");
        
        assert!(result.is_err(), "Should reject empty credential");
        assert!(
            matches!(result.unwrap_err(), CredentialError::InvalidData(_)),
            "Error should be InvalidData"
        );
    }

    #[test]
    fn test_panic_wipe_clears_all_credentials() {
        let manager = test_manager();
        
        // Store multiple credentials
        manager.store(CredentialKey::JiraToken, "jira-token").unwrap();
        manager.store(CredentialKey::GitToken, "git-token").unwrap();
        manager.store(CredentialKey::GeminiApiKey, "gemini-key").unwrap();
        
        // Verify they exist
        assert!(manager.exists(CredentialKey::JiraToken));
        assert!(manager.exists(CredentialKey::GitToken));
        assert!(manager.exists(CredentialKey::GeminiApiKey));
        
        // Panic wipe
        let deleted = manager.panic_wipe().unwrap();
        
        assert_eq!(deleted, 3, "Should delete exactly 3 credentials");
        
        // Verify all are gone
        assert!(!manager.exists(CredentialKey::JiraToken));
        assert!(!manager.exists(CredentialKey::GitToken));
        assert!(!manager.exists(CredentialKey::GeminiApiKey));
    }

    #[test]
    fn test_credential_key_as_str() {
        assert_eq!(CredentialKey::JiraToken.as_str(), "jira_token");
        assert_eq!(CredentialKey::GitToken.as_str(), "git_token");
        assert_eq!(CredentialKey::GeminiApiKey.as_str(), "gemini_api_key");
        assert_eq!(CredentialKey::GrafanaApiKey.as_str(), "grafana_api_key");
        assert_eq!(CredentialKey::ConfluenceToken.as_str(), "confluence_token");
    }

    #[test]
    fn test_credential_key_all_returns_all_keys() {
        let all_keys = CredentialKey::all();
        
        assert_eq!(all_keys.len(), 5, "Should have all 5 credential types");
        assert!(all_keys.contains(&CredentialKey::JiraToken));
        assert!(all_keys.contains(&CredentialKey::GitToken));
        assert!(all_keys.contains(&CredentialKey::GeminiApiKey));
        assert!(all_keys.contains(&CredentialKey::GrafanaApiKey));
        assert!(all_keys.contains(&CredentialKey::ConfluenceToken));
    }

    #[test]
    fn test_exists_returns_true_for_stored_credential() {
        let manager = test_manager();
        
        manager.store(CredentialKey::GrafanaApiKey, "grafana-key").unwrap();
        
        assert!(manager.exists(CredentialKey::GrafanaApiKey));
    }

    #[test]
    fn test_exists_returns_false_for_nonexistent_credential() {
        let manager = test_manager();
        
        assert!(!manager.exists(CredentialKey::ConfluenceToken));
    }

    #[test]
    fn test_overwrite_existing_credential() {
        let manager = test_manager();
        
        manager.store(CredentialKey::JiraToken, "original-token").unwrap();
        manager.store(CredentialKey::JiraToken, "new-token").unwrap();
        
        let retrieved = manager.retrieve(CredentialKey::JiraToken).unwrap();
        assert_eq!(retrieved, "new-token", "Should retrieve the updated value");
    }
}
