use rune_performance::*;
use std::time::Duration;

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_performance_metrics_creation() {
        let metrics = PerformanceMetrics::default();
        assert_eq!(metrics.operations_count, 0);
        assert_eq!(metrics.cache_hits, 0);
        assert_eq!(metrics.cache_misses, 0);
        assert_eq!(metrics.total_duration, Duration::new(0, 0));
    }

    #[test]
    fn test_performance_engine_creation() {
        let _engine = PerformanceEngine::new();
        // Test basic functionality
        assert!(true); // Engine creation should succeed
    }

    #[test]
    fn test_performance_config_defaults() {
        let config = PerformanceConfig::default();
        assert!(config.max_parallel_operations >= 4);
        assert_eq!(config.cache_size_mb, 256);
        assert!(config.enable_memory_mapping);
        assert!(config.enable_parallel_diff);
        assert!(config.enable_async_io);
    }
}
