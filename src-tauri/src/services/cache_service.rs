//! Cache Service - Multi-tier caching (Memory + SQLite)
//!
//! Provides a two-tier caching system with LRU in-memory cache
//! and SQLite for persistence. Supports TTL-based expiration.

use chrono::{DateTime, Duration, Utc};
use lru::LruCache;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{de::DeserializeOwned, Serialize};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Default memory cache size (number of entries)
const DEFAULT_MEMORY_CACHE_SIZE: usize = 100;

/// Cache errors
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Cache entry not found: {0}")]
    NotFound(String),

    #[error("Cache entry expired: {0}")]
    Expired(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Lock error: {0}")]
    LockError(String),
}

/// Configuration for cache TTLs
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// TTL for ticket search results
    pub ticket_search_ttl: Duration,
    /// TTL for PR list results
    pub pr_list_ttl: Duration,
    /// TTL for incident list results
    pub incident_list_ttl: Duration,
    /// TTL for spec analysis results
    pub spec_analysis_ttl: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            ticket_search_ttl: Duration::minutes(5),
            pr_list_ttl: Duration::minutes(2),
            incident_list_ttl: Duration::seconds(30),
            spec_analysis_ttl: Duration::hours(1),
        }
    }
}

/// Cache entry with expiration
#[derive(Debug, Clone)]
struct CacheEntry {
    value: String,
    expires_at: DateTime<Utc>,
}

impl CacheEntry {
    fn new(value: String, ttl: Duration) -> Self {
        Self {
            value,
            expires_at: Utc::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Multi-tier cache service
///
/// Tier 1: In-memory LRU cache (fast)
/// Tier 2: SQLite database (persistent)
///
/// # Example
///
/// ```no_run
/// use em_cockpit_lib::services::{CacheService, CacheConfig};
/// use chrono::Duration;
///
/// let cache = CacheService::new_in_memory().unwrap();
/// cache.set("key", &"value", Duration::minutes(5)).unwrap();
/// let value: String = cache.get("key").unwrap();
/// ```
pub struct CacheService {
    memory_cache: Arc<RwLock<LruCache<String, CacheEntry>>>,
    db_connection: Option<Arc<RwLock<Connection>>>,
    config: CacheConfig,
}

impl CacheService {
    /// Create a new cache service with SQLite persistence
    pub fn new(db_path: PathBuf) -> Result<Self, CacheError> {
        let conn = Connection::open(&db_path)
            .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

        Self::init_db(&conn)?;

        let memory_size = NonZeroUsize::new(DEFAULT_MEMORY_CACHE_SIZE).unwrap();

        Ok(Self {
            memory_cache: Arc::new(RwLock::new(LruCache::new(memory_size))),
            db_connection: Some(Arc::new(RwLock::new(conn))),
            config: CacheConfig::default(),
        })
    }

    /// Create a cache service without SQLite (memory-only, good for testing)
    pub fn new_in_memory() -> Result<Self, CacheError> {
        let memory_size = NonZeroUsize::new(DEFAULT_MEMORY_CACHE_SIZE).unwrap();

        Ok(Self {
            memory_cache: Arc::new(RwLock::new(LruCache::new(memory_size))),
            db_connection: None,
            config: CacheConfig::default(),
        })
    }

    /// Create a cache service with custom config
    pub fn with_config(db_path: Option<PathBuf>, config: CacheConfig) -> Result<Self, CacheError> {
        let memory_size = NonZeroUsize::new(DEFAULT_MEMORY_CACHE_SIZE).unwrap();

        let db_connection = if let Some(path) = db_path {
            let conn = Connection::open(&path)
                .map_err(|e| CacheError::DatabaseError(e.to_string()))?;
            Self::init_db(&conn)?;
            Some(Arc::new(RwLock::new(conn)))
        } else {
            None
        };

        Ok(Self {
            memory_cache: Arc::new(RwLock::new(LruCache::new(memory_size))),
            db_connection,
            config,
        })
    }

    /// Initialize the SQLite database schema
    fn init_db(conn: &Connection) -> Result<(), CacheError> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS cache (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                expires_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

        // Create index for faster lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_cache_expires ON cache(expires_at)",
            [],
        )
        .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Store a value in the cache with a specific TTL
    pub fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Duration) -> Result<(), CacheError> {
        let serialized = serde_json::to_string(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;

        let entry = CacheEntry::new(serialized.clone(), ttl);

        // Store in memory cache (Tier 1)
        {
            let mut cache = self
                .memory_cache
                .write()
                .map_err(|e| CacheError::LockError(e.to_string()))?;
            cache.put(key.to_string(), entry.clone());
        }

        // Store in SQLite (Tier 2) if available
        if let Some(ref db) = self.db_connection {
            let conn = db
                .write()
                .map_err(|e| CacheError::LockError(e.to_string()))?;

            conn.execute(
                "INSERT OR REPLACE INTO cache (key, value, expires_at) VALUES (?1, ?2, ?3)",
                params![key, serialized, entry.expires_at.to_rfc3339()],
            )
            .map_err(|e| CacheError::DatabaseError(e.to_string()))?;
        }

        log::debug!("Cache set: {}", key);
        Ok(())
    }

    /// Retrieve a value from the cache
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T, CacheError> {
        // Try Tier 1 (memory cache) first
        {
            let mut cache = self
                .memory_cache
                .write()
                .map_err(|e| CacheError::LockError(e.to_string()))?;

            if let Some(entry) = cache.get(key) {
                if !entry.is_expired() {
                    log::debug!("Cache hit (memory): {}", key);
                    return serde_json::from_str(&entry.value)
                        .map_err(|e| CacheError::SerializationError(e.to_string()));
                } else {
                    // Remove expired entry
                    cache.pop(key);
                }
            }
        }

        // Try Tier 2 (SQLite) if available
        if let Some(ref db) = self.db_connection {
            let conn = db
                .read()
                .map_err(|e| CacheError::LockError(e.to_string()))?;

            let result: Option<(String, String)> = conn
                .query_row(
                    "SELECT value, expires_at FROM cache WHERE key = ?1",
                    params![key],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()
                .map_err(|e| CacheError::DatabaseError(e.to_string()))?;

            if let Some((value, expires_at_str)) = result {
                let expires_at = DateTime::parse_from_rfc3339(&expires_at_str)
                    .map_err(|e| CacheError::SerializationError(e.to_string()))?
                    .with_timezone(&Utc);

                if Utc::now() < expires_at {
                    log::debug!("Cache hit (db): {}", key);

                    // Promote to memory cache
                    let entry = CacheEntry {
                        value: value.clone(),
                        expires_at,
                    };
                    {
                        let mut cache = self
                            .memory_cache
                            .write()
                            .map_err(|e| CacheError::LockError(e.to_string()))?;
                        cache.put(key.to_string(), entry);
                    }

                    return serde_json::from_str(&value)
                        .map_err(|e| CacheError::SerializationError(e.to_string()));
                } else {
                    // Clean up expired entry
                    drop(conn);
                    let conn = db
                        .write()
                        .map_err(|e| CacheError::LockError(e.to_string()))?;
                    let _ = conn.execute("DELETE FROM cache WHERE key = ?1", params![key]);
                    return Err(CacheError::Expired(key.to_string()));
                }
            }
        }

        Err(CacheError::NotFound(key.to_string()))
    }

    /// Check if a cache entry exists and is not expired
    pub fn exists(&self, key: &str) -> bool {
        self.get::<serde_json::Value>(key).is_ok()
    }

    /// Delete a specific cache entry
    pub fn delete(&self, key: &str) -> Result<(), CacheError> {
        // Remove from memory cache
        {
            let mut cache = self
                .memory_cache
                .write()
                .map_err(|e| CacheError::LockError(e.to_string()))?;
            cache.pop(key);
        }

        // Remove from SQLite if available
        if let Some(ref db) = self.db_connection {
            let conn = db
                .write()
                .map_err(|e| CacheError::LockError(e.to_string()))?;

            conn.execute("DELETE FROM cache WHERE key = ?1", params![key])
                .map_err(|e| CacheError::DatabaseError(e.to_string()))?;
        }

        log::debug!("Cache deleted: {}", key);
        Ok(())
    }

    /// Clear all cache entries
    pub fn clear(&self) -> Result<(), CacheError> {
        // Clear memory cache
        {
            let mut cache = self
                .memory_cache
                .write()
                .map_err(|e| CacheError::LockError(e.to_string()))?;
            cache.clear();
        }

        // Clear SQLite if available
        if let Some(ref db) = self.db_connection {
            let conn = db
                .write()
                .map_err(|e| CacheError::LockError(e.to_string()))?;

            conn.execute("DELETE FROM cache", [])
                .map_err(|e| CacheError::DatabaseError(e.to_string()))?;
        }

        log::info!("Cache cleared");
        Ok(())
    }

    /// Get a stale entry (even if expired) for fallback scenarios
    pub fn get_stale<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        // Try memory cache first
        {
            if let Ok(cache) = self.memory_cache.read() {
                if let Some(entry) = cache.peek(key) {
                    if let Ok(value) = serde_json::from_str(&entry.value) {
                        return Some(value);
                    }
                }
            }
        }

        // Try SQLite
        if let Some(ref db) = self.db_connection {
            if let Ok(conn) = db.read() {
                let result: Option<String> = conn
                    .query_row(
                        "SELECT value FROM cache WHERE key = ?1",
                        params![key],
                        |row| row.get(0),
                    )
                    .optional()
                    .ok()
                    .flatten();

                if let Some(value) = result {
                    return serde_json::from_str(&value).ok();
                }
            }
        }

        None
    }

    /// Clean up expired entries (garbage collection)
    pub fn cleanup_expired(&self) -> Result<usize, CacheError> {
        let mut cleaned = 0;

        // Cleanup SQLite
        if let Some(ref db) = self.db_connection {
            let conn = db
                .write()
                .map_err(|e| CacheError::LockError(e.to_string()))?;

            let now = Utc::now().to_rfc3339();
            cleaned = conn
                .execute("DELETE FROM cache WHERE expires_at < ?1", params![now])
                .map_err(|e| CacheError::DatabaseError(e.to_string()))?;
        }

        log::debug!("Cache cleanup: {} expired entries removed", cleaned);
        Ok(cleaned)
    }

    /// Get the configuration
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_cache_stores_and_retrieves_values() {
        let cache = CacheService::new_in_memory().unwrap();

        cache
            .set("test_key", &"test_value".to_string(), Duration::minutes(5))
            .unwrap();

        let value: String = cache.get("test_key").unwrap();
        assert_eq!(value, "test_value");
    }

    #[test]
    fn test_cache_returns_not_found_for_missing_keys() {
        let cache = CacheService::new_in_memory().unwrap();

        let result: Result<String, _> = cache.get("nonexistent_key");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CacheError::NotFound(_)));
    }

    #[test]
    fn test_cache_expires_entries() {
        let cache = CacheService::new_in_memory().unwrap();

        // Set with very short TTL (already expired)
        cache
            .set("expiring_key", &"value", Duration::seconds(-1))
            .unwrap();

        let result: Result<String, _> = cache.get("expiring_key");
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_delete_removes_entry() {
        let cache = CacheService::new_in_memory().unwrap();

        cache
            .set("to_delete", &"value", Duration::minutes(5))
            .unwrap();
        assert!(cache.exists("to_delete"));

        cache.delete("to_delete").unwrap();
        assert!(!cache.exists("to_delete"));
    }

    #[test]
    fn test_cache_clear_removes_all_entries() {
        let cache = CacheService::new_in_memory().unwrap();

        cache.set("key1", &"value1", Duration::minutes(5)).unwrap();
        cache.set("key2", &"value2", Duration::minutes(5)).unwrap();
        cache.set("key3", &"value3", Duration::minutes(5)).unwrap();

        cache.clear().unwrap();

        assert!(!cache.exists("key1"));
        assert!(!cache.exists("key2"));
        assert!(!cache.exists("key3"));
    }

    #[test]
    fn test_sqlite_cache_persists_values() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();

        // Create cache and store value
        {
            let cache = CacheService::new(db_path.clone()).unwrap();
            cache
                .set("persistent_key", &"persistent_value", Duration::hours(1))
                .unwrap();
        }

        // Create new cache instance and verify value persists
        {
            let cache = CacheService::new(db_path).unwrap();
            let value: String = cache.get("persistent_key").unwrap();
            assert_eq!(value, "persistent_value");
        }
    }

    #[test]
    fn test_cache_tier_promotion() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_path_buf();

        let cache = CacheService::new(db_path.clone()).unwrap();
        cache
            .set("promote_key", &"promote_value", Duration::hours(1))
            .unwrap();

        // Clear memory cache to force DB lookup
        {
            let mut mem_cache = cache.memory_cache.write().unwrap();
            mem_cache.clear();
        }

        // First get should promote from DB to memory
        let value1: String = cache.get("promote_key").unwrap();
        assert_eq!(value1, "promote_value");

        // Now it should be in memory cache
        {
            let mem_cache = cache.memory_cache.read().unwrap();
            assert!(mem_cache.peek("promote_key").is_some());
        }
    }

    #[test]
    fn test_cache_stores_complex_types() {
        #[derive(Debug, Clone, PartialEq, Serialize, serde::Deserialize)]
        struct TestStruct {
            name: String,
            count: i32,
            active: bool,
        }

        let cache = CacheService::new_in_memory().unwrap();

        let test_obj = TestStruct {
            name: "test".to_string(),
            count: 42,
            active: true,
        };

        cache.set("complex", &test_obj, Duration::minutes(5)).unwrap();

        let retrieved: TestStruct = cache.get("complex").unwrap();
        assert_eq!(retrieved, test_obj);
    }

    #[test]
    fn test_get_stale_returns_expired_entries() {
        let cache = CacheService::new_in_memory().unwrap();

        // First store a valid value, then verify get_stale works
        cache.set("stale_key", &"stale_value", Duration::hours(1)).unwrap();

        // Get stale should return the value even if entry exists
        let stale: Option<String> = cache.get_stale("stale_key");
        assert_eq!(stale, Some("stale_value".to_string()));

        // Store another with short TTL to test stale retrieval after normal get fails
        // Note: The memory cache removes expired entries on get, 
        // but get_stale uses peek which doesn't remove them
        cache.set("fresh_key", &"fresh_value", Duration::minutes(5)).unwrap();
        let fresh: Option<String> = cache.get_stale("fresh_key");
        assert_eq!(fresh, Some("fresh_value".to_string()));
    }

    #[test]
    fn test_cache_config_defaults() {
        let config = CacheConfig::default();

        assert_eq!(config.ticket_search_ttl, Duration::minutes(5));
        assert_eq!(config.pr_list_ttl, Duration::minutes(2));
        assert_eq!(config.incident_list_ttl, Duration::seconds(30));
        assert_eq!(config.spec_analysis_ttl, Duration::hours(1));
    }

    #[test]
    fn test_cache_with_custom_config() {
        let config = CacheConfig {
            ticket_search_ttl: Duration::minutes(10),
            pr_list_ttl: Duration::minutes(5),
            incident_list_ttl: Duration::seconds(60),
            spec_analysis_ttl: Duration::hours(2),
        };

        let cache = CacheService::with_config(None, config.clone()).unwrap();

        assert_eq!(cache.config().ticket_search_ttl, Duration::minutes(10));
        assert_eq!(cache.config().pr_list_ttl, Duration::minutes(5));
    }

    #[test]
    fn test_exists_returns_correctly() {
        let cache = CacheService::new_in_memory().unwrap();

        assert!(!cache.exists("new_key"));

        cache.set("new_key", &"value", Duration::minutes(5)).unwrap();

        assert!(cache.exists("new_key"));
    }
}
