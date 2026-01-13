//! Integration Tests for EM Cockpit Backend
//!
//! End-to-end tests covering complete user flows and service integration.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Instant;

use em_cockpit_lib::{
    AppState,
    core::{EventBus, create_event_bus, AppEvent},
    services::{BackgroundPoller, PollerConfig, CacheError},
    system::TrayState,
};

/// Test helper to create an in-memory app state for testing
fn create_test_state() -> AppState {
    AppState::new_in_memory().expect("Failed to create test state")
}

/// Test helper to create a shared event bus
fn create_test_event_bus() -> Arc<EventBus> {
    create_event_bus()
}

// ============================================
// User Flow Tests
// ============================================

#[test]
fn test_app_initialization_flow() {
    // Test that the app initializes correctly with all components
    let state = create_test_state();
    
    // Verify config loaded
    assert!(!state.config.shortcuts.flight_console.is_empty());
    
    // Verify default settings
    assert_eq!(state.config.shortcuts.flight_console, "Alt+Space");
}

#[test]
fn test_event_bus_integration() {
    // Test that events flow through the system correctly
    let event_bus = create_test_event_bus();
    let events_received = Arc::new(AtomicUsize::new(0));
    let events_clone = events_received.clone();
    
    // Subscribe to events
    event_bus.subscribe(move |event| {
        events_clone.fetch_add(1, Ordering::SeqCst);
        match event {
            AppEvent::TrayStateChanged { .. } => {}
            AppEvent::PrDataUpdated { .. } => {}
            _ => {}
        }
    });
    
    // Publish events
    event_bus.publish(AppEvent::TrayStateChanged {
        old_state: TrayState::Neutral,
        new_state: TrayState::Green,
        reason: "Test".to_string(),
    });
    
    event_bus.publish(AppEvent::PrDataUpdated {
        total_open: 5,
        stale_count: 2,
        pending_review: 3,
    });
    
    // Verify events received
    assert_eq!(events_received.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn test_background_poller_integration() {
    let event_bus = create_test_event_bus();
    let events_received = Arc::new(AtomicUsize::new(0));
    let events_clone = events_received.clone();
    
    event_bus.subscribe(move |_| {
        events_clone.fetch_add(1, Ordering::SeqCst);
    });
    
    let config = PollerConfig::default();
    let poller = BackgroundPoller::new(config, event_bus.clone());
    
    // Start poller
    poller.start().await;
    assert!(poller.is_running().await);
    
    // Trigger manual poll
    poller.poll_prs().await;
    poller.poll_incidents().await;
    
    // Stop poller
    poller.stop().await;
    assert!(!poller.is_running().await);
    
    // Verify events were published
    assert!(events_received.load(Ordering::SeqCst) >= 2);
}

// ============================================
// Cache Integration Tests
// ============================================

#[test]
fn test_cache_integration() {
    let state = create_test_state();
    
    // Test cache operations
    let key = "test_key";
    let value = "test_value";
    
    state.cache_service.set(key, &value, chrono::Duration::hours(1)).unwrap();
    let retrieved: String = state.cache_service.get(key).unwrap();
    
    assert_eq!(retrieved, value);
}

#[test]
fn test_cache_miss_handling() {
    let state = create_test_state();
    
    let result: Result<String, CacheError> = state.cache_service.get("nonexistent_key");
    assert!(result.is_err());
}

// ============================================
// Poller State Tracking Tests
// ============================================

#[tokio::test]
async fn test_poller_state_tracking() {
    let event_bus = create_test_event_bus();
    let config = PollerConfig::default();
    let poller = BackgroundPoller::new(config, event_bus);
    
    // Initial state
    let stats = poller.get_stats().await;
    assert_eq!(stats.pr_poll_count, 0);
    assert_eq!(stats.incident_poll_count, 0);
    
    // Poll
    poller.poll_prs().await;
    poller.poll_incidents().await;
    
    // Verify state updated
    let stats = poller.get_stats().await;
    assert_eq!(stats.pr_poll_count, 1);
    assert_eq!(stats.incident_poll_count, 1);
    assert!(stats.last_pr_poll.is_some());
    assert!(stats.last_incident_poll.is_some());
}

#[tokio::test]
async fn test_poller_state_consistency() {
    let event_bus = create_test_event_bus();
    let config = PollerConfig::default();
    let poller = BackgroundPoller::new(config, event_bus);
    
    // Run multiple poll cycles
    for _ in 0..10 {
        poller.poll_prs().await;
    }
    
    let stats = poller.get_stats().await;
    assert_eq!(stats.pr_poll_count, 10);
    assert_eq!(stats.consecutive_pr_failures, 0);
}

// ============================================
// Error Handling Tests
// ============================================

#[test]
fn test_event_handler_error_isolation() {
    let event_bus = create_test_event_bus();
    let success_count = Arc::new(AtomicUsize::new(0));
    let success_clone = success_count.clone();
    
    // First handler that panics
    event_bus.subscribe(|_| {
        panic!("Intentional panic for testing");
    });
    
    // Second handler that should still work
    event_bus.subscribe(move |_| {
        success_clone.fetch_add(1, Ordering::SeqCst);
    });
    
    // Publish - should not crash
    event_bus.publish(AppEvent::SettingsChanged { section: "test".to_string() });
    
    // Second handler should have executed
    assert_eq!(success_count.load(Ordering::SeqCst), 1);
}

// ============================================
// Concurrent Access Tests
// ============================================

#[test]
fn test_event_bus_concurrent_publishing() {
    let event_bus = create_test_event_bus();
    let total_events = Arc::new(AtomicUsize::new(0));
    let total_clone = total_events.clone();
    
    event_bus.subscribe(move |_| {
        total_clone.fetch_add(1, Ordering::SeqCst);
    });
    
    // Spawn multiple threads publishing events
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let bus = event_bus.clone();
            thread::spawn(move || {
                for j in 0..10 {
                    bus.publish(AppEvent::PollingTick {
                        poll_type: format!("thread-{}-{}", i, j),
                        timestamp: chrono::Utc::now(),
                        success: true,
                    });
                }
            })
        })
        .collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // All 100 events should have been received
    assert_eq!(total_events.load(Ordering::SeqCst), 100);
}

#[test]
fn test_event_bus_concurrent_subscription() {
    let event_bus = create_test_event_bus();
    
    // Subscribe from multiple threads
    let handles: Vec<_> = (0..5)
        .map(|_| {
            let bus = event_bus.clone();
            thread::spawn(move || {
                bus.subscribe(|_| {});
            })
        })
        .collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    assert_eq!(event_bus.subscriber_count(), 5);
}

// ============================================
// Performance Tests
// ============================================

#[test]
fn test_cache_performance() {
    let state = create_test_state();
    let iterations = 100; // Keep within LRU cache size limits
    
    // Write performance
    let start = Instant::now();
    for i in 0..iterations {
        let key = format!("perf_test_{}", i);
        let value = format!("value_{}", i);
        state.cache_service.set(&key, &value, chrono::Duration::hours(1)).unwrap();
    }
    let write_duration = start.elapsed();
    
    // Should complete in reasonable time (< 1s for 100 ops)
    assert!(write_duration.as_secs() < 1, "Cache writes too slow: {:?}", write_duration);
    
    // Read performance - only read recently written values
    let start = Instant::now();
    for i in 0..iterations {
        let key = format!("perf_test_{}", i);
        let result: Result<String, CacheError> = state.cache_service.get(&key);
        // Some may have been evicted from LRU, just check performance
        if result.is_ok() {
            assert!(!result.unwrap().is_empty());
        }
    }
    let read_duration = start.elapsed();
    
    assert!(read_duration.as_secs() < 1, "Cache reads too slow: {:?}", read_duration);
}

#[test]
fn test_event_publishing_performance() {
    let event_bus = create_test_event_bus();
    let iterations = 10000;
    
    event_bus.subscribe(|_| {});
    
    let start = Instant::now();
    for _ in 0..iterations {
        event_bus.publish(AppEvent::SettingsChanged { section: "perf".to_string() });
    }
    let duration = start.elapsed();
    
    // Should handle 10k events in < 100ms
    assert!(duration.as_millis() < 100, "Event publishing too slow: {:?}", duration);
}

// ============================================
// History Consistency Tests
// ============================================

#[test]
fn test_event_history_consistency() {
    let event_bus = Arc::new(EventBus::with_history_size(5));
    
    // Publish more events than history size
    for i in 0..10 {
        event_bus.publish(AppEvent::SettingsChanged { 
            section: format!("section_{}", i) 
        });
    }
    
    let history = event_bus.get_history();
    assert_eq!(history.len(), 5); // Only last 5 kept
}
