//! Event Bus - Observer Pattern Implementation
//!
//! Provides pub-sub event system for decoupled communication
//! between components. Thread-safe and supports multiple subscribers.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::system::TrayState;

/// Event types that can be published
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppEvent {
    /// Tray state changed
    TrayStateChanged {
        old_state: TrayState,
        new_state: TrayState,
        reason: String,
    },
    /// PR data updated
    PrDataUpdated {
        total_open: usize,
        stale_count: usize,
        pending_review: usize,
    },
    /// Incident state changed
    IncidentStateChanged {
        active_count: usize,
        critical_count: usize,
        new_incidents: Vec<String>,
    },
    /// Search completed
    SearchCompleted {
        query: String,
        result_count: usize,
        duration_ms: u64,
    },
    /// Cache invalidated
    CacheInvalidated {
        cache_type: String,
        keys: Vec<String>,
    },
    /// Settings changed
    SettingsChanged {
        section: String,
    },
    /// Error occurred
    ErrorOccurred {
        source: String,
        message: String,
        recoverable: bool,
    },
    /// Polling tick completed
    PollingTick {
        poll_type: String,
        timestamp: DateTime<Utc>,
        success: bool,
    },
}

impl AppEvent {
    /// Get the event type name for logging/debugging
    pub fn type_name(&self) -> &'static str {
        match self {
            AppEvent::TrayStateChanged { .. } => "TrayStateChanged",
            AppEvent::PrDataUpdated { .. } => "PrDataUpdated",
            AppEvent::IncidentStateChanged { .. } => "IncidentStateChanged",
            AppEvent::SearchCompleted { .. } => "SearchCompleted",
            AppEvent::CacheInvalidated { .. } => "CacheInvalidated",
            AppEvent::SettingsChanged { .. } => "SettingsChanged",
            AppEvent::ErrorOccurred { .. } => "ErrorOccurred",
            AppEvent::PollingTick { .. } => "PollingTick",
        }
    }
}

/// Event handler function type
pub type EventHandler = Box<dyn Fn(&AppEvent) + Send + Sync>;

/// Unique subscription identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(usize);

/// Event Bus for pub-sub communication
pub struct EventBus {
    /// All subscribers indexed by ID
    subscribers: RwLock<HashMap<SubscriptionId, EventHandler>>,
    /// Next subscription ID
    next_id: RwLock<usize>,
    /// Event history for debugging (last N events)
    history: RwLock<Vec<(DateTime<Utc>, AppEvent)>>,
    /// Maximum history size
    max_history: usize,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self {
            subscribers: RwLock::new(HashMap::new()),
            next_id: RwLock::new(0),
            history: RwLock::new(Vec::new()),
            max_history: 100,
        }
    }

    /// Create event bus with custom history size
    pub fn with_history_size(max_history: usize) -> Self {
        Self {
            subscribers: RwLock::new(HashMap::new()),
            next_id: RwLock::new(0),
            history: RwLock::new(Vec::new()),
            max_history,
        }
    }

    /// Subscribe to all events
    pub fn subscribe<F>(&self, handler: F) -> SubscriptionId
    where
        F: Fn(&AppEvent) + Send + Sync + 'static,
    {
        let mut next_id = self.next_id.write().unwrap();
        let id = SubscriptionId(*next_id);
        *next_id += 1;

        let mut subscribers = self.subscribers.write().unwrap();
        subscribers.insert(id, Box::new(handler));

        log::debug!("EventBus: New subscription {:?}", id);
        id
    }

    /// Unsubscribe from events
    pub fn unsubscribe(&self, id: SubscriptionId) -> bool {
        let mut subscribers = self.subscribers.write().unwrap();
        let removed = subscribers.remove(&id).is_some();
        
        if removed {
            log::debug!("EventBus: Removed subscription {:?}", id);
        }
        removed
    }

    /// Publish an event to all subscribers
    pub fn publish(&self, event: AppEvent) {
        // Record in history
        {
            let mut history = self.history.write().unwrap();
            history.push((Utc::now(), event.clone()));
            
            // Trim history if needed
            while history.len() > self.max_history {
                history.remove(0);
            }
        }

        log::debug!("EventBus: Publishing {}", event.type_name());

        // Notify all subscribers
        let subscribers = self.subscribers.read().unwrap();
        for (id, handler) in subscribers.iter() {
            if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                handler(&event);
            })) {
                log::error!("EventBus: Handler {:?} panicked: {:?}", id, e);
            }
        }
    }

    /// Get subscriber count
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.read().unwrap().len()
    }

    /// Get recent event history
    pub fn get_history(&self) -> Vec<(DateTime<Utc>, AppEvent)> {
        self.history.read().unwrap().clone()
    }

    /// Clear event history
    pub fn clear_history(&self) {
        self.history.write().unwrap().clear();
    }

    /// Clear all subscribers
    pub fn clear_subscribers(&self) {
        self.subscribers.write().unwrap().clear();
        log::info!("EventBus: All subscribers cleared");
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

// Debug implementation
impl std::fmt::Debug for EventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBus")
            .field("subscriber_count", &self.subscriber_count())
            .field("history_size", &self.history.read().unwrap().len())
            .finish()
    }
}

// Thread-safe wrapper for shared access
pub type SharedEventBus = Arc<EventBus>;

/// Create a shared event bus
pub fn create_event_bus() -> SharedEventBus {
    Arc::new(EventBus::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;

    #[test]
    fn test_subscribe_returns_unique_id() {
        let bus = EventBus::new();
        
        let id1 = bus.subscribe(|_| {});
        let id2 = bus.subscribe(|_| {});
        
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_publish_notifies_subscribers() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        
        bus.subscribe(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });
        
        bus.publish(AppEvent::SettingsChanged { section: "test".to_string() });
        
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_multiple_subscribers_receive_events() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        
        for _ in 0..3 {
            let counter_clone = counter.clone();
            bus.subscribe(move |_| {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            });
        }
        
        bus.publish(AppEvent::SettingsChanged { section: "test".to_string() });
        
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_unsubscribe_removes_handler() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        
        let id = bus.subscribe(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });
        
        assert!(bus.unsubscribe(id));
        bus.publish(AppEvent::SettingsChanged { section: "test".to_string() });
        
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_unsubscribe_nonexistent_returns_false() {
        let bus = EventBus::new();
        assert!(!bus.unsubscribe(SubscriptionId(999)));
    }

    #[test]
    fn test_subscriber_count() {
        let bus = EventBus::new();
        
        assert_eq!(bus.subscriber_count(), 0);
        
        let id1 = bus.subscribe(|_| {});
        assert_eq!(bus.subscriber_count(), 1);
        
        let id2 = bus.subscribe(|_| {});
        assert_eq!(bus.subscriber_count(), 2);
        
        bus.unsubscribe(id1);
        assert_eq!(bus.subscriber_count(), 1);
    }

    #[test]
    fn test_event_history_recorded() {
        let bus = EventBus::with_history_size(10);
        
        bus.publish(AppEvent::SettingsChanged { section: "test1".to_string() });
        bus.publish(AppEvent::SettingsChanged { section: "test2".to_string() });
        
        let history = bus.get_history();
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_history_trimmed_to_max_size() {
        let bus = EventBus::with_history_size(2);
        
        bus.publish(AppEvent::SettingsChanged { section: "1".to_string() });
        bus.publish(AppEvent::SettingsChanged { section: "2".to_string() });
        bus.publish(AppEvent::SettingsChanged { section: "3".to_string() });
        
        let history = bus.get_history();
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_clear_history() {
        let bus = EventBus::new();
        
        bus.publish(AppEvent::SettingsChanged { section: "test".to_string() });
        assert!(!bus.get_history().is_empty());
        
        bus.clear_history();
        assert!(bus.get_history().is_empty());
    }

    #[test]
    fn test_clear_subscribers() {
        let bus = EventBus::new();
        
        bus.subscribe(|_| {});
        bus.subscribe(|_| {});
        assert_eq!(bus.subscriber_count(), 2);
        
        bus.clear_subscribers();
        assert_eq!(bus.subscriber_count(), 0);
    }

    #[test]
    fn test_app_event_type_name() {
        let event = AppEvent::TrayStateChanged {
            old_state: TrayState::Neutral,
            new_state: TrayState::Green,
            reason: "test".to_string(),
        };
        assert_eq!(event.type_name(), "TrayStateChanged");
        
        let event = AppEvent::PrDataUpdated {
            total_open: 5,
            stale_count: 2,
            pending_review: 3,
        };
        assert_eq!(event.type_name(), "PrDataUpdated");
    }

    #[test]
    fn test_thread_safe_concurrent_publish() {
        let bus = Arc::new(EventBus::new());
        let counter = Arc::new(AtomicUsize::new(0));
        
        let counter_clone = counter.clone();
        bus.subscribe(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });
        
        // Spawn multiple threads publishing events
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let bus = bus.clone();
                thread::spawn(move || {
                    bus.publish(AppEvent::SettingsChanged { 
                        section: format!("thread-{}", i) 
                    });
                })
            })
            .collect();
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_panicking_handler_does_not_crash_bus() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        
        // First handler panics
        bus.subscribe(|_| {
            panic!("intentional panic");
        });
        
        // Second handler should still execute
        bus.subscribe(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });
        
        // This should not panic
        bus.publish(AppEvent::SettingsChanged { section: "test".to_string() });
        
        // The non-panicking handler should have been called
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
