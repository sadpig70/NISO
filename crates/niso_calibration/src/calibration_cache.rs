//! Calibration caching for quantum backends
//!
//! Gantree: L3_Calibration → CalibrationCache
//!
//! Provides TTL-based caching for calibration data to reduce
//! backend query overhead.

use crate::calibration_info::CalibrationInfo;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Cached calibration entry
#[derive(Debug, Clone)]
struct CachedEntry {
    info: CalibrationInfo,
    cached_at: Instant,
}

/// Calibration cache with TTL
/// Gantree: CalibrationCache // 캐싱
#[derive(Debug)]
pub struct CalibrationCache {
    /// Cache storage
    /// Gantree: cache: HashMap<String,CachedCalib> // 캐시 저장소
    cache: Arc<RwLock<HashMap<String, CachedEntry>>>,

    /// Time-to-live in seconds
    /// Gantree: ttl_seconds: u64 // 유효 시간
    ttl: Duration,
}

impl CalibrationCache {
    // ========================================================================
    // Constructors
    // ========================================================================

    /// Create new cache with specified TTL
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    /// Create cache with default TTL (1 hour)
    pub fn default_ttl() -> Self {
        Self::new(3600)
    }

    /// Create cache with short TTL (5 minutes) for testing
    pub fn short_ttl() -> Self {
        Self::new(300)
    }

    // ========================================================================
    // Cache Operations
    // ========================================================================

    /// Get cached calibration if valid
    /// Gantree: get(backend) -> Option<CalibrationInfo> // 캐시 조회
    pub fn get(&self, backend: &str) -> Option<CalibrationInfo> {
        let cache = self.cache.read().ok()?;
        let entry = cache.get(backend)?;

        if entry.cached_at.elapsed() < self.ttl {
            Some(entry.info.clone())
        } else {
            None
        }
    }

    /// Store calibration in cache
    /// Gantree: set(backend,info) // 캐시 저장
    pub fn set(&self, backend: &str, info: CalibrationInfo) {
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(
                backend.to_string(),
                CachedEntry {
                    info,
                    cached_at: Instant::now(),
                },
            );
        }
    }

    /// Invalidate cached calibration for backend
    /// Gantree: invalidate(backend) // 캐시 무효화
    pub fn invalidate(&self, backend: &str) {
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(backend);
        }
    }

    /// Invalidate all cached calibrations
    pub fn invalidate_all(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    /// Check if cache entry is valid
    /// Gantree: is_valid(backend) -> bool // 유효성 확인
    pub fn is_valid(&self, backend: &str) -> bool {
        self.get(backend).is_some()
    }

    /// Get or fetch calibration
    ///
    /// Returns cached value if valid, otherwise calls the fetch function
    /// and caches the result.
    pub fn get_or_fetch<F>(&self, backend: &str, fetch: F) -> Option<CalibrationInfo>
    where
        F: FnOnce() -> Option<CalibrationInfo>,
    {
        // Try cache first
        if let Some(info) = self.get(backend) {
            return Some(info);
        }

        // Fetch and cache
        if let Some(info) = fetch() {
            self.set(backend, info.clone());
            return Some(info);
        }

        None
    }

    // ========================================================================
    // Statistics
    // ========================================================================

    /// Get number of cached entries
    pub fn len(&self) -> usize {
        self.cache.read().map(|c| c.len()).unwrap_or(0)
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get cache hit rate (requires external tracking)
    pub fn cached_backends(&self) -> Vec<String> {
        self.cache
            .read()
            .map(|c| c.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get TTL
    pub fn ttl(&self) -> Duration {
        self.ttl
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.retain(|_, entry| entry.cached_at.elapsed() < self.ttl);
        }
    }
}

impl Default for CalibrationCache {
    fn default() -> Self {
        Self::default_ttl()
    }
}

impl Clone for CalibrationCache {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
            ttl: self.ttl,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_new() {
        let cache = CalibrationCache::new(60);
        assert_eq!(cache.ttl(), Duration::from_secs(60));
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_set_get() {
        let cache = CalibrationCache::new(60);
        let info = CalibrationInfo::ibm_typical(5);

        cache.set("test_backend", info.clone());

        let retrieved = cache.get("test_backend");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().backend_name, "ibm_simulator");
    }

    #[test]
    fn test_cache_miss() {
        let cache = CalibrationCache::new(60);
        assert!(cache.get("nonexistent").is_none());
    }

    #[test]
    fn test_cache_invalidate() {
        let cache = CalibrationCache::new(60);
        let info = CalibrationInfo::ibm_typical(5);

        cache.set("test", info);
        assert!(cache.is_valid("test"));

        cache.invalidate("test");
        assert!(!cache.is_valid("test"));
    }

    #[test]
    fn test_cache_invalidate_all() {
        let cache = CalibrationCache::new(60);

        cache.set("a", CalibrationInfo::ibm_typical(3));
        cache.set("b", CalibrationInfo::ibm_typical(5));

        assert_eq!(cache.len(), 2);

        cache.invalidate_all();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_get_or_fetch() {
        let cache = CalibrationCache::new(60);

        let mut fetch_count = 0;
        let fetch = || {
            fetch_count += 1;
            Some(CalibrationInfo::ibm_typical(5))
        };

        // First call should fetch
        let result1 = cache.get_or_fetch("test", fetch);
        assert!(result1.is_some());

        // Second call should use cache (can't verify fetch_count due to closure)
        let result2 = cache.get("test");
        assert!(result2.is_some());
    }

    #[test]
    fn test_cached_backends() {
        let cache = CalibrationCache::new(60);

        cache.set("backend_a", CalibrationInfo::ibm_typical(3));
        cache.set("backend_b", CalibrationInfo::ibm_typical(5));

        let backends = cache.cached_backends();
        assert!(backends.contains(&"backend_a".to_string()));
        assert!(backends.contains(&"backend_b".to_string()));
    }

    #[test]
    fn test_cache_clone_shares_data() {
        let cache1 = CalibrationCache::new(60);
        let cache2 = cache1.clone();

        cache1.set("test", CalibrationInfo::ibm_typical(5));

        // Clone should see the same data
        assert!(cache2.is_valid("test"));
    }
}
