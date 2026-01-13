//! Core module for application-wide types and configuration
//!
//! Contains shared types, configuration models, and event system.

mod config;
mod errors;
pub mod events;

pub use config::AppConfig;
pub use config::IntegrationConfig;
pub use errors::CockpitError;
pub use events::{AppEvent, EventBus, SharedEventBus, SubscriptionId, create_event_bus};
