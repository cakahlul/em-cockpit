//! Services module for business logic
//!
//! Contains service implementations for caching, search, PR monitoring, etc.

mod cache_service;
mod search_service;
mod pr_aggregator;
mod incident_monitor;
mod background_poller;

pub use cache_service::CacheService;
pub use cache_service::CacheError;
pub use cache_service::CacheConfig;
pub use search_service::{SearchService, SearchResult, SearchResultType, SearchResultMetadata};
pub use pr_aggregator::{PrAggregator, PrSummary};
pub use incident_monitor::{IncidentMonitor, IncidentSummary};
pub use background_poller::{BackgroundPoller, PollerConfig, PollerState, PollingStats};
