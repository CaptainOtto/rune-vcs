// Simple performance optimization module for Rune VCS
// This provides basic performance improvements without complex dependencies

use anyhow::Result;
use std::time::{Duration, Instant};
use std::sync::Arc;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use colored::*;

/// Performance metrics and optimization engine
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    pub operations_count: usize,
    pub total_duration: Duration,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub memory_usage_mb: f64,
}

impl PerformanceMetrics {
    pub fn cache_hit_ratio(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    pub fn operations_per_second(&self) -> f64 {
        if self.total_duration.as_secs_f64() > 0.0 {
            self.operations_count as f64 / self.total_duration.as_secs_f64()
        } else {
            0.0
        }
    }
}

/// Simple cache for performance optimization
pub struct SimpleCache<K, V> {
    data: HashMap<K, (V, Instant)>,
    ttl: Duration,
    max_size: usize,
}

impl<K, V> SimpleCache<K, V> 
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    pub fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            data: HashMap::new(),
            ttl,
            max_size,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some((value, timestamp)) = self.data.get(key) {
            if timestamp.elapsed() < self.ttl {
                return Some(value.clone());
            } else {
                self.data.remove(key);
            }
        }
        None
    }

    pub fn put(&mut self, key: K, value: V) {
        if self.data.len() >= self.max_size {
            // Simple eviction - remove oldest entry
            if let Some(oldest_key) = self.data.keys().next().cloned() {
                self.data.remove(&oldest_key);
            }
        }
        self.data.insert(key, (value, Instant::now()));
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }
}

/// Performance optimization engine
pub struct PerformanceEngine {
    cache: Arc<std::sync::Mutex<SimpleCache<String, Vec<u8>>>>,
    metrics: Arc<std::sync::RwLock<PerformanceMetrics>>,
}

impl PerformanceEngine {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(std::sync::Mutex::new(SimpleCache::new(1000, Duration::from_secs(3600)))),
            metrics: Arc::new(std::sync::RwLock::new(PerformanceMetrics::default())),
        }
    }

    pub fn get_cached_data(&self, key: &str) -> Option<Vec<u8>> {
        if let Ok(mut cache) = self.cache.lock() {
            if let Some(data) = cache.get(&key.to_string()) {
                self.record_cache_hit();
                return Some(data);
            }
        }
        self.record_cache_miss();
        None
    }

    pub fn cache_data(&self, key: &str, data: Vec<u8>) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.put(key.to_string(), data);
        }
    }

    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }

    pub fn benchmark<F>(&self, name: &str, operation: F) -> Result<Duration> 
    where
        F: FnOnce() -> Result<()>,
    {
        println!("{} Running benchmark: {}", "🔥".yellow(), name.cyan());
        
        let start = Instant::now();
        operation()?;
        let duration = start.elapsed();
        
        self.record_operation(duration);
        
        println!("{} Completed in: {:.2?}", "✅".green(), duration);
        Ok(duration)
    }

    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().unwrap().clone()
    }

    pub fn print_performance_summary(&self) {
        let metrics = self.get_metrics();
        
        println!("\n{} Performance Summary", "📊".blue());
        println!("==================");
        println!("Operations: {}", metrics.operations_count);
        println!("Total time: {:.2?}", metrics.total_duration);
        println!("Ops/sec: {:.1}", metrics.operations_per_second());
        println!("Cache hit ratio: {:.1}%", metrics.cache_hit_ratio() * 100.0);
        println!("Memory usage: {:.1} MB", metrics.memory_usage_mb);
    }

    fn record_cache_hit(&self) {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.cache_hits += 1;
        }
    }

    fn record_cache_miss(&self) {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.cache_misses += 1;
        }
    }

    fn record_operation(&self, duration: Duration) {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.operations_count += 1;
            metrics.total_duration += duration;
        }
    }
}

impl Default for PerformanceEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_simple_cache() {
        let mut cache = SimpleCache::new(2, Duration::from_secs(1));
        
        cache.put("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        
        // Test eviction
        cache.put("key2".to_string(), "value2".to_string());
        cache.put("key3".to_string(), "value3".to_string());
        assert_eq!(cache.size(), 2);
    }

    #[test]
    fn test_performance_engine() {
        let engine = PerformanceEngine::new();
        
        // Test caching
        engine.cache_data("test", b"data".to_vec());
        assert_eq!(engine.get_cached_data("test"), Some(b"data".to_vec()));
        
        // Test benchmarking
        let duration = engine.benchmark("test_op", || {
            thread::sleep(Duration::from_millis(10));
            Ok(())
        }).unwrap();
        
        assert!(duration >= Duration::from_millis(10));
        
        let metrics = engine.get_metrics();
        assert_eq!(metrics.operations_count, 1);
        assert_eq!(metrics.cache_hits, 1);
    }
}
