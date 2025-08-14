use rune_performance::*;

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_performance_metrics_creation() {
        let metrics = PerformanceMetrics::new();
        assert_eq!(metrics.total_files, 0);
        assert_eq!(metrics.total_size, 0);
        assert!(metrics.processing_time.is_zero());
    }

    #[test]
    fn test_performance_reporter_creation() {
        let reporter = PerformanceReporter::new();
        assert!(reporter.metrics.processing_time.is_zero());
    }

    #[test]
    fn test_performance_config_defaults() {
        let config = PerformanceConfig::default();
        assert!(config.enable_profiling);
        assert!(config.enable_metrics);
        assert_eq!(config.sample_rate, 1.0);
        assert_eq!(config.buffer_size, 1000);
    }
}
