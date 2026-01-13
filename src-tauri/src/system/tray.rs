//! System Tray Management (Tray Beacon)
//!
//! Provides system tray icon management with state-based coloring
//! and context menu for quick actions.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Errors that can occur during tray operations
#[derive(Error, Debug)]
pub enum TrayError {
    #[error("Failed to create tray icon: {0}")]
    CreationFailed(String),

    #[error("Failed to update tray icon: {0}")]
    UpdateFailed(String),

    #[error("Tray not initialized")]
    NotInitialized,

    #[error("Lock error: {0}")]
    LockError(String),
}

/// Tray icon state representing application status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrayState {
    /// No pending items, all systems nominal
    Neutral,
    /// All monitored systems nominal, no pending PR reviews
    Green,
    /// Warning state: PRs pending >24h or upcoming events
    Amber,
    /// Critical state: Active incident or error rate above thresholds
    Red,
}

impl TrayState {
    /// Get the color hex code for this state
    pub fn color_hex(&self) -> &'static str {
        match self {
            TrayState::Neutral => "#6B7280", // Gray
            TrayState::Green => "#10B981",   // Green
            TrayState::Amber => "#F59E0B",   // Amber
            TrayState::Red => "#EF4444",     // Red
        }
    }

    /// Get the priority of this state (higher = more urgent)
    pub fn priority(&self) -> u8 {
        match self {
            TrayState::Neutral => 0,
            TrayState::Green => 1,
            TrayState::Amber => 2,
            TrayState::Red => 3,
        }
    }

    /// Check if this state should trigger a visual alert
    pub fn should_alert(&self) -> bool {
        matches!(self, TrayState::Red)
    }

    /// Check if this is a critical state
    pub fn is_critical(&self) -> bool {
        matches!(self, TrayState::Red)
    }

    /// Check if this is a warning state
    pub fn is_warning(&self) -> bool {
        matches!(self, TrayState::Amber)
    }

    /// Combine two states, taking the more severe one
    pub fn combine(&self, other: &TrayState) -> TrayState {
        if self.priority() >= other.priority() {
            *self
        } else {
            *other
        }
    }
}

impl Default for TrayState {
    fn default() -> Self {
        TrayState::Neutral
    }
}

impl fmt::Display for TrayState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrayState::Neutral => write!(f, "Neutral"),
            TrayState::Green => write!(f, "All Clear"),
            TrayState::Amber => write!(f, "Attention Needed"),
            TrayState::Red => write!(f, "Critical"),
        }
    }
}

/// Tray status details for tooltip
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrayStatus {
    /// Current tray state
    pub state: TrayState,
    /// Number of PRs waiting for review
    pub pending_prs: usize,
    /// Number of stale PRs (>48h)
    pub stale_prs: usize,
    /// Number of active incidents
    pub active_incidents: usize,
    /// Custom status message
    pub message: Option<String>,
}

impl TrayStatus {
    /// Create a new tray status
    pub fn new() -> Self {
        Self::default()
    }

    /// Update with PR counts
    pub fn with_prs(mut self, pending: usize, stale: usize) -> Self {
        self.pending_prs = pending;
        self.stale_prs = stale;
        self.recalculate_state();
        self
    }

    /// Update with incident count
    pub fn with_incidents(mut self, count: usize) -> Self {
        self.active_incidents = count;
        self.recalculate_state();
        self
    }

    /// Set a custom message
    pub fn with_message(mut self, message: &str) -> Self {
        self.message = Some(message.to_string());
        self
    }

    /// Calculate the overall state based on status details
    pub fn recalculate_state(&mut self) {
        // Red: Active incidents
        if self.active_incidents > 0 {
            self.state = TrayState::Red;
            return;
        }

        // Amber: Stale PRs (>24h would trigger this in real usage)
        if self.stale_prs > 0 {
            self.state = TrayState::Amber;
            return;
        }

        // Green: Everything is fine
        if self.pending_prs == 0 {
            self.state = TrayState::Green;
            return;
        }

        // Neutral: Has pending PRs but not stale
        self.state = TrayState::Neutral;
    }

    /// Generate tooltip text
    pub fn tooltip(&self) -> String {
        let mut parts = Vec::new();

        if self.pending_prs > 0 {
            parts.push(format!(
                "{} PR{} waiting",
                self.pending_prs,
                if self.pending_prs == 1 { "" } else { "s" }
            ));
        }

        if self.stale_prs > 0 {
            parts.push(format!(
                "{} stale PR{}",
                self.stale_prs,
                if self.stale_prs == 1 { "" } else { "s" }
            ));
        }

        if self.active_incidents > 0 {
            parts.push(format!(
                "{} active incident{}",
                self.active_incidents,
                if self.active_incidents == 1 { "" } else { "s" }
            ));
        }

        if parts.is_empty() {
            if let Some(ref msg) = self.message {
                return msg.clone();
            }
            return "All systems nominal.".to_string();
        }

        parts.join(". ") + "."
    }
}

/// Context menu action
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TrayMenuAction {
    OpenFlightConsole,
    OpenRadarPanel,
    OpenIncidentRadar,
    OpenHangar,
    Quit,
}

impl TrayMenuAction {
    /// Get the display label for this action
    pub fn label(&self) -> &'static str {
        match self {
            TrayMenuAction::OpenFlightConsole => "Open Flight Console",
            TrayMenuAction::OpenRadarPanel => "Open Radar Panel",
            TrayMenuAction::OpenIncidentRadar => "Open Incident Radar",
            TrayMenuAction::OpenHangar => "Open Hangar (Settings)",
            TrayMenuAction::Quit => "Quit",
        }
    }

    /// Get all menu actions in order
    pub fn all() -> Vec<TrayMenuAction> {
        vec![
            TrayMenuAction::OpenFlightConsole,
            TrayMenuAction::OpenRadarPanel,
            TrayMenuAction::OpenIncidentRadar,
            TrayMenuAction::OpenHangar,
            TrayMenuAction::Quit,
        ]
    }
}

/// Manages the system tray icon and its state
pub struct TrayManager {
    /// Current status
    status: Arc<RwLock<TrayStatus>>,
    /// Previous state (for transition detection)
    previous_state: Arc<RwLock<TrayState>>,
    /// Whether the tray is initialized
    initialized: std::sync::atomic::AtomicBool,
}

impl TrayManager {
    /// Create a new TrayManager
    pub fn new() -> Self {
        Self {
            status: Arc::new(RwLock::new(TrayStatus::default())),
            previous_state: Arc::new(RwLock::new(TrayState::Neutral)),
            initialized: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Initialize the tray (call once at app startup)
    pub fn initialize(&self) -> Result<(), TrayError> {
        self.initialized
            .store(true, std::sync::atomic::Ordering::SeqCst);
        log::info!("Tray manager initialized");
        Ok(())
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Get current status
    pub fn get_status(&self) -> Result<TrayStatus, TrayError> {
        self.status
            .read()
            .map(|s| s.clone())
            .map_err(|e| TrayError::LockError(e.to_string()))
    }

    /// Get current state
    pub fn get_state(&self) -> Result<TrayState, TrayError> {
        self.status
            .read()
            .map(|s| s.state)
            .map_err(|e| TrayError::LockError(e.to_string()))
    }

    /// Update the tray status
    pub fn update_status(&self, new_status: TrayStatus) -> Result<StateChange, TrayError> {
        let previous = self.get_state()?;

        // Update status
        {
            let mut status = self
                .status
                .write()
                .map_err(|e| TrayError::LockError(e.to_string()))?;
            *status = new_status.clone();
        }

        // Track previous state
        {
            let mut prev = self
                .previous_state
                .write()
                .map_err(|e| TrayError::LockError(e.to_string()))?;
            *prev = previous;
        }

        let change = StateChange {
            from: previous,
            to: new_status.state,
            should_animate: previous != new_status.state,
            should_alert: new_status.state.should_alert() && !previous.should_alert(),
        };

        if change.should_animate {
            log::info!("Tray state changed: {} -> {}", previous, new_status.state);
        }

        Ok(change)
    }

    /// Update PR counts
    pub fn update_prs(&self, pending: usize, stale: usize) -> Result<StateChange, TrayError> {
        let mut status = self.get_status()?;
        status.pending_prs = pending;
        status.stale_prs = stale;
        status.recalculate_state();
        self.update_status(status)
    }

    /// Update incident count
    pub fn update_incidents(&self, count: usize) -> Result<StateChange, TrayError> {
        let mut status = self.get_status()?;
        status.active_incidents = count;
        status.recalculate_state();
        self.update_status(status)
    }

    /// Get current tooltip text
    pub fn get_tooltip(&self) -> Result<String, TrayError> {
        let status = self.get_status()?;
        Ok(status.tooltip())
    }

    /// Reset to neutral state
    pub fn reset(&self) -> Result<(), TrayError> {
        self.update_status(TrayStatus::default())?;
        Ok(())
    }
}

impl Default for TrayManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a state transition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateChange {
    /// Previous state
    pub from: TrayState,
    /// New state
    pub to: TrayState,
    /// Whether to animate the transition
    pub should_animate: bool,
    /// Whether to trigger an alert (e.g., pulse effect)
    pub should_alert: bool,
}

impl StateChange {
    /// Check if this is an escalation (worse state)
    pub fn is_escalation(&self) -> bool {
        self.to.priority() > self.from.priority()
    }

    /// Check if this is a de-escalation (better state)
    pub fn is_de_escalation(&self) -> bool {
        self.to.priority() < self.from.priority()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== TrayState Tests =====

    #[test]
    fn test_tray_state_priority() {
        assert!(TrayState::Red.priority() > TrayState::Amber.priority());
        assert!(TrayState::Amber.priority() > TrayState::Green.priority());
        assert!(TrayState::Green.priority() > TrayState::Neutral.priority());
    }

    #[test]
    fn test_tray_state_combine_takes_higher_priority() {
        assert_eq!(TrayState::Red.combine(&TrayState::Green), TrayState::Red);
        assert_eq!(TrayState::Green.combine(&TrayState::Red), TrayState::Red);
        assert_eq!(
            TrayState::Amber.combine(&TrayState::Neutral),
            TrayState::Amber
        );
    }

    #[test]
    fn test_tray_state_should_alert() {
        assert!(TrayState::Red.should_alert());
        assert!(!TrayState::Amber.should_alert());
        assert!(!TrayState::Green.should_alert());
        assert!(!TrayState::Neutral.should_alert());
    }

    #[test]
    fn test_tray_state_color_hex() {
        assert_eq!(TrayState::Red.color_hex(), "#EF4444");
        assert_eq!(TrayState::Amber.color_hex(), "#F59E0B");
        assert_eq!(TrayState::Green.color_hex(), "#10B981");
        assert_eq!(TrayState::Neutral.color_hex(), "#6B7280");
    }

    // ===== TrayStatus Tests =====

    #[test]
    fn test_tray_status_default() {
        let status = TrayStatus::new();
        assert_eq!(status.state, TrayState::Neutral);
        assert_eq!(status.pending_prs, 0);
        assert_eq!(status.active_incidents, 0);
    }

    #[test]
    fn test_tray_status_with_incidents_is_red() {
        let status = TrayStatus::new().with_incidents(1);
        assert_eq!(status.state, TrayState::Red);
    }

    #[test]
    fn test_tray_status_with_stale_prs_is_amber() {
        let status = TrayStatus::new().with_prs(5, 2);
        assert_eq!(status.state, TrayState::Amber);
    }

    #[test]
    fn test_tray_status_no_pending_is_green() {
        let status = TrayStatus::new().with_prs(0, 0);
        assert_eq!(status.state, TrayState::Green);
    }

    #[test]
    fn test_tray_status_pending_but_not_stale_is_neutral() {
        let status = TrayStatus::new().with_prs(3, 0);
        assert_eq!(status.state, TrayState::Neutral);
    }

    #[test]
    fn test_tray_status_incidents_override_prs() {
        let status = TrayStatus::new().with_prs(0, 0).with_incidents(1);
        assert_eq!(status.state, TrayState::Red);
    }

    #[test]
    fn test_tooltip_with_prs() {
        let status = TrayStatus::new().with_prs(3, 1);
        let tooltip = status.tooltip();
        assert!(tooltip.contains("3 PRs waiting"));
        assert!(tooltip.contains("1 stale PR"));
    }

    #[test]
    fn test_tooltip_with_incidents() {
        let status = TrayStatus::new().with_incidents(2);
        let tooltip = status.tooltip();
        assert!(tooltip.contains("2 active incidents"));
    }

    #[test]
    fn test_tooltip_all_clear() {
        let status = TrayStatus::new().with_prs(0, 0).with_incidents(0);
        let tooltip = status.tooltip();
        assert_eq!(tooltip, "All systems nominal.");
    }

    #[test]
    fn test_tooltip_singular_forms() {
        let status = TrayStatus::new().with_prs(1, 1).with_incidents(1);
        let tooltip = status.tooltip();
        assert!(tooltip.contains("1 PR waiting"));
        assert!(tooltip.contains("1 stale PR"));
        assert!(tooltip.contains("1 active incident"));
    }

    // ===== TrayManager Tests =====

    #[test]
    fn test_tray_manager_initialization() {
        let manager = TrayManager::new();
        assert!(!manager.is_initialized());

        manager.initialize().unwrap();
        assert!(manager.is_initialized());
    }

    #[test]
    fn test_tray_manager_initial_state() {
        let manager = TrayManager::new();
        let state = manager.get_state().unwrap();
        assert_eq!(state, TrayState::Neutral);
    }

    #[test]
    fn test_tray_manager_update_status() {
        let manager = TrayManager::new();

        let status = TrayStatus::new().with_incidents(1);
        let change = manager.update_status(status).unwrap();

        assert_eq!(change.from, TrayState::Neutral);
        assert_eq!(change.to, TrayState::Red);
        assert!(change.should_animate);
        assert!(change.should_alert);
    }

    #[test]
    fn test_tray_manager_update_prs() {
        let manager = TrayManager::new();

        let change = manager.update_prs(5, 2).unwrap();

        assert_eq!(change.to, TrayState::Amber);
        assert!(change.should_animate);
    }

    #[test]
    fn test_tray_manager_update_incidents() {
        let manager = TrayManager::new();

        let change = manager.update_incidents(1).unwrap();

        assert_eq!(change.to, TrayState::Red);
        assert!(change.should_alert);
    }

    #[test]
    fn test_tray_manager_get_tooltip() {
        let manager = TrayManager::new();

        manager.update_prs(3, 1).unwrap();
        let tooltip = manager.get_tooltip().unwrap();

        assert!(tooltip.contains("3 PRs"));
    }

    #[test]
    fn test_tray_manager_reset() {
        let manager = TrayManager::new();

        manager.update_incidents(1).unwrap();
        assert_eq!(manager.get_state().unwrap(), TrayState::Red);

        manager.reset().unwrap();
        assert_eq!(manager.get_state().unwrap(), TrayState::Neutral);
    }

    #[test]
    fn test_state_change_escalation() {
        let change = StateChange {
            from: TrayState::Green,
            to: TrayState::Red,
            should_animate: true,
            should_alert: true,
        };

        assert!(change.is_escalation());
        assert!(!change.is_de_escalation());
    }

    #[test]
    fn test_state_change_de_escalation() {
        let change = StateChange {
            from: TrayState::Red,
            to: TrayState::Green,
            should_animate: true,
            should_alert: false,
        };

        assert!(!change.is_escalation());
        assert!(change.is_de_escalation());
    }

    // ===== TrayMenuAction Tests =====

    #[test]
    fn test_tray_menu_action_labels() {
        assert_eq!(TrayMenuAction::OpenFlightConsole.label(), "Open Flight Console");
        assert_eq!(TrayMenuAction::Quit.label(), "Quit");
    }

    #[test]
    fn test_tray_menu_action_all() {
        let actions = TrayMenuAction::all();
        assert_eq!(actions.len(), 5);
        assert!(actions.contains(&TrayMenuAction::OpenFlightConsole));
        assert!(actions.contains(&TrayMenuAction::Quit));
    }
}
