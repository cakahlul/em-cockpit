//! Security module for credential management and encryption
//!
//! This module provides secure storage and retrieval of credentials
//! using the OS keychain (macOS Keychain, Windows Credential Manager, etc.)

mod credential_manager;

pub use credential_manager::CredentialManager;
pub use credential_manager::CredentialError;
