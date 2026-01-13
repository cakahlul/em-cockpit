//! Global Hotkey Management
//!
//! Provides cross-platform global hotkey registration and handling
//! for the Flight Console and other keyboard shortcuts.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// Errors that can occur during hotkey operations
#[derive(Error, Debug)]
pub enum HotkeyError {
    #[error("Invalid shortcut format: {0}")]
    InvalidFormat(String),

    #[error("Failed to register hotkey: {0}")]
    RegistrationFailed(String),

    #[error("Failed to unregister hotkey: {0}")]
    UnregistrationFailed(String),

    #[error("Hotkey conflict: {0}")]
    Conflict(String),

    #[error("Unsupported modifier: {0}")]
    UnsupportedModifier(String),

    #[error("Unsupported key: {0}")]
    UnsupportedKey(String),
}

/// Modifier keys for shortcuts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Modifier {
    Alt,
    Ctrl,
    Shift,
    Meta, // Cmd on macOS, Win on Windows
}

impl Modifier {
    /// Parse modifier from string
    pub fn from_str_case_insensitive(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "alt" | "option" | "opt" => Some(Modifier::Alt),
            "ctrl" | "control" => Some(Modifier::Ctrl),
            "shift" => Some(Modifier::Shift),
            "meta" | "cmd" | "command" | "win" | "super" => Some(Modifier::Meta),
            _ => None,
        }
    }

    /// Convert to display string
    pub fn as_str(&self) -> &'static str {
        match self {
            Modifier::Alt => "Alt",
            Modifier::Ctrl => "Ctrl",
            Modifier::Shift => "Shift",
            Modifier::Meta => {
                #[cfg(target_os = "macos")]
                return "Cmd";
                #[cfg(not(target_os = "macos"))]
                return "Win";
            }
        }
    }

    /// Convert to Tauri accelerator format
    pub fn to_accelerator(&self) -> &'static str {
        match self {
            Modifier::Alt => "Alt",
            Modifier::Ctrl => "Ctrl",
            Modifier::Shift => "Shift",
            Modifier::Meta => "Super",
        }
    }
}

impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Represents a keyboard shortcut
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Shortcut {
    /// Modifier keys (Alt, Ctrl, Shift, Meta)
    pub modifiers: Vec<Modifier>,
    /// Main key (e.g., "Space", "A", "F1")
    pub key: String,
}

impl Shortcut {
    /// Create a new shortcut
    pub fn new(modifiers: Vec<Modifier>, key: &str) -> Self {
        Self {
            modifiers,
            key: key.to_string(),
        }
    }

    /// Create a shortcut with a single modifier
    pub fn with_modifier(modifier: Modifier, key: &str) -> Self {
        Self {
            modifiers: vec![modifier],
            key: key.to_string(),
        }
    }

    /// Parse a shortcut from a string like "Alt+Space" or "Ctrl+Shift+A"
    pub fn parse(s: &str) -> Result<Self, HotkeyError> {
        let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();

        if parts.is_empty() {
            return Err(HotkeyError::InvalidFormat(
                "Shortcut cannot be empty".to_string(),
            ));
        }

        if parts.len() == 1 {
            return Err(HotkeyError::InvalidFormat(
                "Shortcut must have at least one modifier".to_string(),
            ));
        }

        let mut modifiers = Vec::new();
        let key = parts.last().unwrap().to_string();

        // All parts except the last are modifiers
        for part in &parts[..parts.len() - 1] {
            if let Some(modifier) = Modifier::from_str_case_insensitive(part) {
                if !modifiers.contains(&modifier) {
                    modifiers.push(modifier);
                }
            } else {
                return Err(HotkeyError::UnsupportedModifier(part.to_string()));
            }
        }

        // Validate key
        let valid_key = Self::normalize_key(&key)?;

        Ok(Self {
            modifiers,
            key: valid_key,
        })
    }

    /// Normalize key name to consistent format
    fn normalize_key(key: &str) -> Result<String, HotkeyError> {
        let normalized = match key.to_lowercase().as_str() {
            "space" | " " => "Space",
            "enter" | "return" => "Enter",
            "escape" | "esc" => "Escape",
            "tab" => "Tab",
            "backspace" => "Backspace",
            "delete" | "del" => "Delete",
            "up" | "arrowup" => "ArrowUp",
            "down" | "arrowdown" => "ArrowDown",
            "left" | "arrowleft" => "ArrowLeft",
            "right" | "arrowright" => "ArrowRight",
            "home" => "Home",
            "end" => "End",
            "pageup" => "PageUp",
            "pagedown" => "PageDown",
            // Function keys
            "f1" => "F1",
            "f2" => "F2",
            "f3" => "F3",
            "f4" => "F4",
            "f5" => "F5",
            "f6" => "F6",
            "f7" => "F7",
            "f8" => "F8",
            "f9" => "F9",
            "f10" => "F10",
            "f11" => "F11",
            "f12" => "F12",
            // Single character keys
            k if k.len() == 1 && k.chars().next().unwrap().is_alphanumeric() => {
                return Ok(k.to_uppercase());
            }
            // Numbers
            k if k.starts_with("digit") || k.starts_with("key") => {
                return Ok(k[k.len() - 1..].to_uppercase());
            }
            _ => {
                return Err(HotkeyError::UnsupportedKey(key.to_string()));
            }
        };

        Ok(normalized.to_string())
    }

    /// Convert to Tauri accelerator format
    pub fn to_accelerator(&self) -> String {
        let mut parts: Vec<String> = self
            .modifiers
            .iter()
            .map(|m| m.to_accelerator().to_string())
            .collect();
        parts.push(self.key.clone());
        parts.join("+")
    }

    /// Check if this shortcut conflicts with common system shortcuts
    pub fn conflicts_with_system(&self) -> bool {
        // Check for common system shortcuts that might conflict
        let accelerator = self.to_accelerator().to_lowercase();

        // Common system shortcuts to avoid
        let system_shortcuts = [
            "ctrl+c",
            "ctrl+v",
            "ctrl+x",
            "ctrl+z",
            "ctrl+a",
            "ctrl+s",
            "ctrl+q",
            "ctrl+w",
            "super+space", // Spotlight on macOS
            "alt+f4",      // Close window on Windows
            "alt+tab",     // Window switching
        ];

        system_shortcuts
            .iter()
            .any(|s| accelerator == s.to_lowercase())
    }

    /// Check if shortcuts have the same key combination
    pub fn conflicts_with(&self, other: &Shortcut) -> bool {
        if self.key.to_lowercase() != other.key.to_lowercase() {
            return false;
        }

        // Check if modifiers match (order-independent)
        let mut self_mods = self.modifiers.clone();
        let mut other_mods = other.modifiers.clone();
        self_mods.sort_by_key(|m| format!("{:?}", m));
        other_mods.sort_by_key(|m| format!("{:?}", m));

        self_mods == other_mods
    }
}

impl FromStr for Shortcut {
    type Err = HotkeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Shortcut::parse(s)
    }
}

impl fmt::Display for Shortcut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self
            .modifiers
            .iter()
            .map(|m| m.to_string())
            .chain(std::iter::once(self.key.clone()))
            .collect();
        write!(f, "{}", parts.join("+"))
    }
}

/// Hotkey action types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HotkeyAction {
    /// Open the Flight Console
    OpenFlightConsole,
    /// Open the Radar Panel
    OpenRadarPanel,
    /// Open the Incident Radar
    OpenIncidentRadar,
    /// Open the Spec Scanner
    OpenSpecScanner,
    /// Open the Hangar (Settings)
    OpenHangar,
    /// Custom action with identifier
    Custom(String),
}

/// Callback type for hotkey events
pub type HotkeyCallback = Box<dyn Fn(HotkeyAction) + Send + Sync>;

/// Manages global hotkey registration and handling
///
/// Uses Tauri's global-shortcut plugin for cross-platform support.
pub struct HotkeyManager {
    /// Registered shortcuts and their actions
    registered: std::sync::RwLock<Vec<(Shortcut, HotkeyAction)>>,
    /// Whether the manager is active
    active: std::sync::atomic::AtomicBool,
}

impl HotkeyManager {
    /// Create a new HotkeyManager
    pub fn new() -> Self {
        Self {
            registered: std::sync::RwLock::new(Vec::new()),
            active: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Register a hotkey with an action
    pub fn register(&self, shortcut: Shortcut, action: HotkeyAction) -> Result<(), HotkeyError> {
        // Check for system conflicts
        if shortcut.conflicts_with_system() {
            return Err(HotkeyError::Conflict(format!(
                "Shortcut {} conflicts with system shortcuts",
                shortcut
            )));
        }

        // Check for existing registration conflicts
        {
            let registered = self.registered.read().map_err(|e| {
                HotkeyError::RegistrationFailed(format!("Lock error: {}", e))
            })?;

            for (existing, _) in registered.iter() {
                if shortcut.conflicts_with(existing) {
                    return Err(HotkeyError::Conflict(format!(
                        "Shortcut {} is already registered",
                        shortcut
                    )));
                }
            }
        }

        // Add to registered list
        {
            let mut registered = self.registered.write().map_err(|e| {
                HotkeyError::RegistrationFailed(format!("Lock error: {}", e))
            })?;
            registered.push((shortcut.clone(), action));
        }

        log::info!("Hotkey registered: {}", shortcut);
        Ok(())
    }

    /// Unregister a hotkey
    pub fn unregister(&self, shortcut: &Shortcut) -> Result<(), HotkeyError> {
        let mut registered = self.registered.write().map_err(|e| {
            HotkeyError::UnregistrationFailed(format!("Lock error: {}", e))
        })?;

        let original_len = registered.len();
        registered.retain(|(s, _)| !s.conflicts_with(shortcut));

        if registered.len() == original_len {
            return Err(HotkeyError::UnregistrationFailed(format!(
                "Shortcut {} was not registered",
                shortcut
            )));
        }

        log::info!("Hotkey unregistered: {}", shortcut);
        Ok(())
    }

    /// Unregister all hotkeys
    pub fn unregister_all(&self) -> Result<(), HotkeyError> {
        let mut registered = self.registered.write().map_err(|e| {
            HotkeyError::UnregistrationFailed(format!("Lock error: {}", e))
        })?;

        let count = registered.len();
        registered.clear();

        log::info!("All {} hotkeys unregistered", count);
        Ok(())
    }

    /// Get the action for a shortcut
    pub fn get_action(&self, shortcut: &Shortcut) -> Option<HotkeyAction> {
        let registered = self.registered.read().ok()?;

        for (s, action) in registered.iter() {
            if s.conflicts_with(shortcut) {
                return Some(action.clone());
            }
        }

        None
    }

    /// Check if a shortcut is registered
    pub fn is_registered(&self, shortcut: &Shortcut) -> bool {
        self.get_action(shortcut).is_some()
    }

    /// Get all registered shortcuts
    pub fn get_all_registered(&self) -> Vec<(Shortcut, HotkeyAction)> {
        self.registered.read().map(|r| r.clone()).unwrap_or_default()
    }

    /// Set active state
    pub fn set_active(&self, active: bool) {
        self.active
            .store(active, std::sync::atomic::Ordering::SeqCst);
    }

    /// Check if active
    pub fn is_active(&self) -> bool {
        self.active.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Shortcut Parsing Tests =====

    #[test]
    fn test_parse_simple_shortcut() {
        let shortcut = Shortcut::parse("Alt+Space").unwrap();

        assert_eq!(shortcut.modifiers.len(), 1);
        assert_eq!(shortcut.modifiers[0], Modifier::Alt);
        assert_eq!(shortcut.key, "Space");
    }

    #[test]
    fn test_parse_multi_modifier_shortcut() {
        let shortcut = Shortcut::parse("Ctrl+Shift+A").unwrap();

        assert_eq!(shortcut.modifiers.len(), 2);
        assert!(shortcut.modifiers.contains(&Modifier::Ctrl));
        assert!(shortcut.modifiers.contains(&Modifier::Shift));
        assert_eq!(shortcut.key, "A");
    }

    #[test]
    fn test_parse_case_insensitive() {
        let shortcut = Shortcut::parse("alt+SPACE").unwrap();

        assert_eq!(shortcut.modifiers[0], Modifier::Alt);
        assert_eq!(shortcut.key, "Space");
    }

    #[test]
    fn test_parse_option_as_alt() {
        let shortcut = Shortcut::parse("Option+Space").unwrap();
        assert_eq!(shortcut.modifiers[0], Modifier::Alt);
    }

    #[test]
    fn test_parse_cmd_as_meta() {
        let shortcut = Shortcut::parse("Cmd+K").unwrap();
        assert_eq!(shortcut.modifiers[0], Modifier::Meta);
    }

    #[test]
    fn test_parse_empty_returns_error() {
        let result = Shortcut::parse("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), HotkeyError::InvalidFormat(_)));
    }

    #[test]
    fn test_parse_no_modifier_returns_error() {
        let result = Shortcut::parse("Space");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), HotkeyError::InvalidFormat(_)));
    }

    #[test]
    fn test_parse_invalid_modifier_returns_error() {
        let result = Shortcut::parse("InvalidMod+A");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HotkeyError::UnsupportedModifier(_)
        ));
    }

    #[test]
    fn test_parse_function_keys() {
        let shortcut = Shortcut::parse("Ctrl+F1").unwrap();
        assert_eq!(shortcut.key, "F1");

        let shortcut = Shortcut::parse("Alt+F12").unwrap();
        assert_eq!(shortcut.key, "F12");
    }

    #[test]
    fn test_shortcut_to_accelerator() {
        let shortcut = Shortcut::parse("Alt+Space").unwrap();
        assert_eq!(shortcut.to_accelerator(), "Alt+Space");

        let shortcut = Shortcut::parse("Ctrl+Shift+A").unwrap();
        let accel = shortcut.to_accelerator();
        assert!(accel.contains("Ctrl"));
        assert!(accel.contains("Shift"));
        assert!(accel.contains("A"));
    }

    #[test]
    fn test_shortcut_display() {
        let shortcut = Shortcut::parse("Alt+Space").unwrap();
        let display = format!("{}", shortcut);
        assert!(display.contains("Alt"));
        assert!(display.contains("Space"));
    }

    #[test]
    fn test_shortcut_conflicts_with_same() {
        let s1 = Shortcut::parse("Alt+Space").unwrap();
        let s2 = Shortcut::parse("Alt+Space").unwrap();
        assert!(s1.conflicts_with(&s2));
    }

    #[test]
    fn test_shortcut_no_conflict_different_key() {
        let s1 = Shortcut::parse("Alt+Space").unwrap();
        let s2 = Shortcut::parse("Alt+Enter").unwrap();
        assert!(!s1.conflicts_with(&s2));
    }

    #[test]
    fn test_shortcut_no_conflict_different_modifier() {
        let s1 = Shortcut::parse("Alt+Space").unwrap();
        let s2 = Shortcut::parse("Ctrl+Space").unwrap();
        assert!(!s1.conflicts_with(&s2));
    }

    #[test]
    fn test_conflicts_with_system() {
        let copy = Shortcut::parse("Ctrl+C").unwrap();
        assert!(copy.conflicts_with_system());

        let custom = Shortcut::parse("Alt+Space").unwrap();
        assert!(!custom.conflicts_with_system());
    }

    // ===== HotkeyManager Tests =====

    #[test]
    fn test_register_hotkey_successfully() {
        let manager = HotkeyManager::new();
        let shortcut = Shortcut::parse("Alt+Space").unwrap();

        let result = manager.register(shortcut, HotkeyAction::OpenFlightConsole);
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_duplicate_hotkey_fails() {
        let manager = HotkeyManager::new();
        let shortcut = Shortcut::parse("Alt+Space").unwrap();

        manager
            .register(shortcut.clone(), HotkeyAction::OpenFlightConsole)
            .unwrap();

        let result = manager.register(shortcut, HotkeyAction::OpenRadarPanel);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), HotkeyError::Conflict(_)));
    }

    #[test]
    fn test_register_system_shortcut_fails() {
        let manager = HotkeyManager::new();
        let shortcut = Shortcut::parse("Ctrl+C").unwrap();

        let result = manager.register(shortcut, HotkeyAction::OpenFlightConsole);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), HotkeyError::Conflict(_)));
    }

    #[test]
    fn test_unregister_hotkey() {
        let manager = HotkeyManager::new();
        let shortcut = Shortcut::parse("Alt+Space").unwrap();

        manager
            .register(shortcut.clone(), HotkeyAction::OpenFlightConsole)
            .unwrap();
        assert!(manager.is_registered(&shortcut));

        manager.unregister(&shortcut).unwrap();
        assert!(!manager.is_registered(&shortcut));
    }

    #[test]
    fn test_unregister_nonexistent_fails() {
        let manager = HotkeyManager::new();
        let shortcut = Shortcut::parse("Alt+Space").unwrap();

        let result = manager.unregister(&shortcut);
        assert!(result.is_err());
    }

    #[test]
    fn test_unregister_all() {
        let manager = HotkeyManager::new();

        manager
            .register(
                Shortcut::parse("Alt+Space").unwrap(),
                HotkeyAction::OpenFlightConsole,
            )
            .unwrap();
        manager
            .register(
                Shortcut::parse("Ctrl+1").unwrap(),
                HotkeyAction::OpenRadarPanel,
            )
            .unwrap();

        assert_eq!(manager.get_all_registered().len(), 2);

        manager.unregister_all().unwrap();

        assert_eq!(manager.get_all_registered().len(), 0);
    }

    #[test]
    fn test_get_action() {
        let manager = HotkeyManager::new();
        let shortcut = Shortcut::parse("Alt+Space").unwrap();

        manager
            .register(shortcut.clone(), HotkeyAction::OpenFlightConsole)
            .unwrap();

        let action = manager.get_action(&shortcut);
        assert_eq!(action, Some(HotkeyAction::OpenFlightConsole));
    }

    #[test]
    fn test_get_action_not_found() {
        let manager = HotkeyManager::new();
        let shortcut = Shortcut::parse("Alt+Space").unwrap();

        let action = manager.get_action(&shortcut);
        assert_eq!(action, None);
    }

    #[test]
    fn test_is_active() {
        let manager = HotkeyManager::new();

        assert!(!manager.is_active());

        manager.set_active(true);
        assert!(manager.is_active());

        manager.set_active(false);
        assert!(!manager.is_active());
    }

    // ===== Modifier Tests =====

    #[test]
    fn test_modifier_from_string() {
        assert_eq!(
            Modifier::from_str_case_insensitive("alt"),
            Some(Modifier::Alt)
        );
        assert_eq!(
            Modifier::from_str_case_insensitive("CTRL"),
            Some(Modifier::Ctrl)
        );
        assert_eq!(
            Modifier::from_str_case_insensitive("Command"),
            Some(Modifier::Meta)
        );
        assert_eq!(Modifier::from_str_case_insensitive("invalid"), None);
    }
}
