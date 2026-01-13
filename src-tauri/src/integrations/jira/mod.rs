//! Jira Integration
//!
//! Provides Jira API client implementing the TicketRepository trait.

mod client;

pub use client::JiraClient;
pub use client::JiraConfig;
