// Simple performance optimization module for Rune VCS
pub mod simple;
pub mod advanced;

// Re-export for convenience
pub use simple::{PerformanceEngine, PerformanceMetrics, SimpleCache};
pub use advanced::{AdvancedPerformanceEngine, AdvancedMetrics, PerformanceConfig};
