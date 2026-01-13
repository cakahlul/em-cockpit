//! Tauri Commands Module
//!
//! Exposes backend functionality to the Vue.js frontend.
//! All commands follow Tauri's async command pattern.

pub mod search;
pub mod prs;
pub mod incidents;
pub mod settings;

// Re-export command handlers for registration
pub use search::*;
pub use prs::*;
pub use incidents::*;
pub use settings::*;
