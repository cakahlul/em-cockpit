//! Git Hosting Integration
//!
//! Provides Git hosting API clients (Bitbucket, GitHub, GitLab)
//! implementing the PullRequestRepository trait with Strategy Pattern.

mod provider;

pub use provider::{GitProvider, GitConfig, GitProviderType};
