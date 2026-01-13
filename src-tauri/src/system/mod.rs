//! System Integration Layer
//!
//! Provides OS-level integrations including global hotkeys,
//! system tray management, and native notifications.

mod hotkey;
mod tray;

pub use hotkey::{HotkeyManager, HotkeyError, Shortcut};
pub use tray::{TrayManager, TrayState, TrayError};
