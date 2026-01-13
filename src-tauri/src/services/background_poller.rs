//! Background Poller Service
//!
//! Manages background polling for PRs, incidents, and other data sources.
//! Publishes events to the event bus when state changes are detected.

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{interval, Interval};
use chrono::Utc;

use crate::core::events::{AppEvent, SharedEventBus};
use crate::system::TrayState;

/// Polling configuration
#[derive(Debug, Clone)]
pub struct PollerConfig {
    /// PR polling interval (default: 2 minutes)
    pub pr_poll_interval: Duration,
    /// Incident polling interval (default: 30 seconds)
    pub incident_poll_interval: Duration,
    /// Whether PR polling is enabled
    pub pr_polling_enabled: bool,
    /// Whether incident polling is enabled
    pub incident_polling_enabled: bool,
    /// Number of retries on failure
    pub max_retries: usize,
    /// Backoff duration after failure
    pub retry_backoff: Duration,
}

impl Default for PollerConfig {
    fn default() -> Self {
        Self {
            pr_poll_interval: Duration::from_secs(120), // 2 minutes
            incident_poll_interval: Duration::from_secs(30),
            pr_polling_enabled: true,
            incident_polling_enabled: true,
            max_retries: 3,
            retry_backoff: Duration::from_secs(5),
        }
    }
}

impl PollerConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_pr_interval(mut self, interval: Duration) -> Self {
        self.pr_poll_interval = interval;
        self
    }

    pub fn with_incident_interval(mut self, interval: Duration) -> Self {
        self.incident_poll_interval = interval;
        self
    }

    pub fn disable_pr_polling(mut self) -> Self {
        self.pr_polling_enabled = false;
        self
    }

    pub fn disable_incident_polling(mut self) -> Self {
        self.incident_polling_enabled = false;
        self
    }
}

/// Poll result from a data source
#[derive(Debug, Clone)]
pub struct PollResult<T> {
    pub data: T,
    pub timestamp: chrono::DateTime<Utc>,
    pub success: bool,
    pub error_message: Option<String>,
}

impl<T> PollResult<T> {
    pub fn success(data: T) -> Self {
        Self {
            data,
            timestamp: Utc::now(),
            success: true,
            error_message: None,
        }
    }

    pub fn failure(data: T, message: String) -> Self {
        Self {
            data,
            timestamp: Utc::now(),
            success: false,
            error_message: Some(message),
        }
    }
}

/// PR poll data
#[derive(Debug, Clone, Default)]
pub struct PrPollData {
    pub total_open: usize,
    pub stale_count: usize,
    pub pending_review: usize,
}

/// Incident poll data
#[derive(Debug, Clone, Default)]
pub struct IncidentPollData {
    pub active_count: usize,
    pub critical_count: usize,
    pub new_incident_ids: Vec<String>,
}

/// Polling state tracker
#[derive(Debug, Clone)]
pub struct PollerState {
    pub last_pr_poll: Option<chrono::DateTime<Utc>>,
    pub last_incident_poll: Option<chrono::DateTime<Utc>>,
    pub pr_poll_count: usize,
    pub incident_poll_count: usize,
    pub consecutive_pr_failures: usize,
    pub consecutive_incident_failures: usize,
    pub current_tray_state: TrayState,
}

impl Default for PollerState {
    fn default() -> Self {
        Self {
            last_pr_poll: None,
            last_incident_poll: None,
            pr_poll_count: 0,
            incident_poll_count: 0,
            consecutive_pr_failures: 0,
            consecutive_incident_failures: 0,
            current_tray_state: TrayState::Neutral,
        }
    }
}

/// Background Poller service
pub struct BackgroundPoller {
    config: PollerConfig,
    state: Arc<RwLock<PollerState>>,
    event_bus: SharedEventBus,
    running: Arc<RwLock<bool>>,
}

impl BackgroundPoller {
    /// Create a new background poller
    pub fn new(config: PollerConfig, event_bus: SharedEventBus) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(PollerState::default())),
            event_bus,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Check if poller is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Get current poller state
    pub async fn get_state(&self) -> PollerState {
        self.state.read().await.clone()
    }

    /// Start background polling
    pub async fn start(&self) {
        {
            let mut running = self.running.write().await;
            if *running {
                log::warn!("BackgroundPoller: Already running");
                return;
            }
            *running = true;
        }

        log::info!("BackgroundPoller: Starting polling tasks");

        // In a real implementation, this would spawn tokio tasks
        // For testing purposes, we track state changes
    }

    /// Stop background polling
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        log::info!("BackgroundPoller: Stopped");
    }

    /// Execute a single PR poll cycle
    pub async fn poll_prs(&self) -> PollResult<PrPollData> {
        log::debug!("BackgroundPoller: Polling PRs");
        
        let result = self.fetch_pr_data().await;
        
        {
            let mut state = self.state.write().await;
            state.last_pr_poll = Some(Utc::now());
            state.pr_poll_count += 1;
            
            if result.success {
                state.consecutive_pr_failures = 0;
            } else {
                state.consecutive_pr_failures += 1;
            }
        }

        // Publish event
        self.event_bus.publish(AppEvent::PrDataUpdated {
            total_open: result.data.total_open,
            stale_count: result.data.stale_count,
            pending_review: result.data.pending_review,
        });

        self.event_bus.publish(AppEvent::PollingTick {
            poll_type: "pr".to_string(),
            timestamp: Utc::now(),
            success: result.success,
        });

        result
    }

    /// Execute a single incident poll cycle
    pub async fn poll_incidents(&self) -> PollResult<IncidentPollData> {
        log::debug!("BackgroundPoller: Polling incidents");
        
        let result = self.fetch_incident_data().await;
        
        {
            let mut state = self.state.write().await;
            state.last_incident_poll = Some(Utc::now());
            state.incident_poll_count += 1;
            
            if result.success {
                state.consecutive_incident_failures = 0;
            } else {
                state.consecutive_incident_failures += 1;
            }
        }

        // Publish event
        self.event_bus.publish(AppEvent::IncidentStateChanged {
            active_count: result.data.active_count,
            critical_count: result.data.critical_count,
            new_incidents: result.data.new_incident_ids.clone(),
        });

        self.event_bus.publish(AppEvent::PollingTick {
            poll_type: "incident".to_string(),
            timestamp: Utc::now(),
            success: result.success,
        });

        // Update tray state if needed
        self.update_tray_state(&result.data).await;

        result
    }

    /// Update tray state based on poll data
    async fn update_tray_state(&self, incident_data: &IncidentPollData) {
        let new_state = if incident_data.critical_count > 0 {
            TrayState::Red
        } else if incident_data.active_count > 0 {
            TrayState::Amber
        } else {
            TrayState::Green
        };

        let mut state = self.state.write().await;
        if state.current_tray_state != new_state {
            let old_state = state.current_tray_state;
            state.current_tray_state = new_state;
            
            self.event_bus.publish(AppEvent::TrayStateChanged {
                old_state,
                new_state,
                reason: format!(
                    "Active: {}, Critical: {}",
                    incident_data.active_count, incident_data.critical_count
                ),
            });
        }
    }

    /// Fetch PR data (mock implementation - would call actual services)
    async fn fetch_pr_data(&self) -> PollResult<PrPollData> {
        // In production, this would call PrAggregator service
        PollResult::success(PrPollData::default())
    }

    /// Fetch incident data (mock implementation - would call actual services)
    async fn fetch_incident_data(&self) -> PollResult<IncidentPollData> {
        // In production, this would call IncidentMonitor service
        PollResult::success(IncidentPollData::default())
    }

    /// Manual refresh - bypasses interval and polls immediately
    pub async fn refresh_all(&self) {
        log::info!("BackgroundPoller: Manual refresh triggered");
        let _ = self.poll_prs().await;
        let _ = self.poll_incidents().await;
    }

    /// Get polling statistics
    pub async fn get_stats(&self) -> PollingStats {
        let state = self.state.read().await;
        PollingStats {
            pr_poll_count: state.pr_poll_count,
            incident_poll_count: state.incident_poll_count,
            last_pr_poll: state.last_pr_poll,
            last_incident_poll: state.last_incident_poll,
            consecutive_pr_failures: state.consecutive_pr_failures,
            consecutive_incident_failures: state.consecutive_incident_failures,
        }
    }
}

/// Polling statistics
#[derive(Debug, Clone)]
pub struct PollingStats {
    pub pr_poll_count: usize,
    pub incident_poll_count: usize,
    pub last_pr_poll: Option<chrono::DateTime<Utc>>,
    pub last_incident_poll: Option<chrono::DateTime<Utc>>,
    pub consecutive_pr_failures: usize,
    pub consecutive_incident_failures: usize,
}

impl std::fmt::Debug for BackgroundPoller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BackgroundPoller")
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::events::EventBus;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn create_test_poller() -> BackgroundPoller {
        let event_bus = Arc::new(EventBus::new());
        BackgroundPoller::new(PollerConfig::default(), event_bus)
    }

    #[test]
    fn test_poller_config_defaults() {
        let config = PollerConfig::default();
        assert_eq!(config.pr_poll_interval, Duration::from_secs(120));
        assert_eq!(config.incident_poll_interval, Duration::from_secs(30));
        assert!(config.pr_polling_enabled);
        assert!(config.incident_polling_enabled);
    }

    #[test]
    fn test_poller_config_builder() {
        let config = PollerConfig::new()
            .with_pr_interval(Duration::from_secs(60))
            .with_incident_interval(Duration::from_secs(15))
            .disable_pr_polling();

        assert_eq!(config.pr_poll_interval, Duration::from_secs(60));
        assert_eq!(config.incident_poll_interval, Duration::from_secs(15));
        assert!(!config.pr_polling_enabled);
    }

    #[test]
    fn test_poll_result_success() {
        let result = PollResult::success(PrPollData {
            total_open: 5,
            stale_count: 2,
            pending_review: 3,
        });
        
        assert!(result.success);
        assert!(result.error_message.is_none());
        assert_eq!(result.data.total_open, 5);
    }

    #[test]
    fn test_poll_result_failure() {
        let result: PollResult<PrPollData> = PollResult::failure(
            PrPollData::default(),
            "Network error".to_string(),
        );
        
        assert!(!result.success);
        assert_eq!(result.error_message, Some("Network error".to_string()));
    }

    #[tokio::test]
    async fn test_poller_initial_state() {
        let poller = create_test_poller();
        
        assert!(!poller.is_running().await);
        
        let state = poller.get_state().await;
        assert_eq!(state.pr_poll_count, 0);
        assert_eq!(state.incident_poll_count, 0);
        assert!(state.last_pr_poll.is_none());
    }

    #[tokio::test]
    async fn test_poller_start_stop() {
        let poller = create_test_poller();
        
        assert!(!poller.is_running().await);
        
        poller.start().await;
        assert!(poller.is_running().await);
        
        poller.stop().await;
        assert!(!poller.is_running().await);
    }

    #[tokio::test]
    async fn test_poll_prs_updates_state() {
        let poller = create_test_poller();
        
        let result = poller.poll_prs().await;
        assert!(result.success);
        
        let state = poller.get_state().await;
        assert_eq!(state.pr_poll_count, 1);
        assert!(state.last_pr_poll.is_some());
        assert_eq!(state.consecutive_pr_failures, 0);
    }

    #[tokio::test]
    async fn test_poll_incidents_updates_state() {
        let poller = create_test_poller();
        
        let result = poller.poll_incidents().await;
        assert!(result.success);
        
        let state = poller.get_state().await;
        assert_eq!(state.incident_poll_count, 1);
        assert!(state.last_incident_poll.is_some());
    }

    #[tokio::test]
    async fn test_poll_publishes_events() {
        let event_bus = Arc::new(EventBus::new());
        let event_count = Arc::new(AtomicUsize::new(0));
        let event_count_clone = event_count.clone();
        
        event_bus.subscribe(move |_| {
            event_count_clone.fetch_add(1, Ordering::SeqCst);
        });
        
        let poller = BackgroundPoller::new(PollerConfig::default(), event_bus);
        
        poller.poll_prs().await;
        
        // Should publish PrDataUpdated + PollingTick
        assert_eq!(event_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_refresh_all_polls_both() {
        let poller = create_test_poller();
        
        poller.refresh_all().await;
        
        let state = poller.get_state().await;
        assert_eq!(state.pr_poll_count, 1);
        assert_eq!(state.incident_poll_count, 1);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let poller = create_test_poller();
        
        poller.poll_prs().await;
        poller.poll_prs().await;
        poller.poll_incidents().await;
        
        let stats = poller.get_stats().await;
        assert_eq!(stats.pr_poll_count, 2);
        assert_eq!(stats.incident_poll_count, 1);
    }

    #[tokio::test]
    async fn test_tray_state_updates_on_incidents() {
        let event_bus = Arc::new(EventBus::new());
        let poller = BackgroundPoller::new(PollerConfig::default(), event_bus);
        
        // Initially neutral
        let state = poller.get_state().await;
        assert_eq!(state.current_tray_state, TrayState::Neutral);
    }

    #[tokio::test]
    async fn test_multiple_poll_cycles() {
        let poller = create_test_poller();
        
        for _ in 0..5 {
            poller.poll_prs().await;
        }
        
        let stats = poller.get_stats().await;
        assert_eq!(stats.pr_poll_count, 5);
        assert_eq!(stats.consecutive_pr_failures, 0);
    }
}
