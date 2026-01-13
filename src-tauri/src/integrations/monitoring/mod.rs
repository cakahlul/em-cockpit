//! Monitoring Integration
//!
//! Provides monitoring platform clients (Grafana, Datadog)
//! implementing the MetricsRepository trait.

mod grafana;

pub use grafana::{GrafanaClient, MonitoringConfig};
