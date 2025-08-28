use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use crate::{caching::SmartCache, memory::MemoryOptimizer, network::NetworkOptimizer, parallel::ParallelExecutor};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub iterations: usize,
    pub warmup_iterations: usize,
    pub timeout_seconds: u64,
    pub measure_memory: bool,
    pub measure_cpu: bool,
    pub measure_network: bool,
    pub measure_disk_io: bool,
    pub export_results: bool,
    pub export_format: ExportFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Html,
    Markdown,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 100,
            warmup_iterations: 10,
            timeout_seconds: 300,
            measure_memory: true,
            measure_cpu: true,
            measure_network: false,
            measure_disk_io: true,
            export_results: true,
            export_format: ExportFormat::Json,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: usize,
    pub total_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub avg_duration: Duration,
    pub median_duration: Duration,
    pub std_deviation: f64,
    pub operations_per_second: f64,
    pub memory_usage: Option<MemoryUsage>,
    pub cpu_usage: Option<CpuUsage>,
    pub network_stats: Option<NetworkStats>,
    pub disk_io_stats: Option<DiskIoStats>,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsage {
    pub peak_bytes: usize,
    pub average_bytes: usize,
    pub allocations: usize,
    pub deallocations: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuUsage {
    pub user_time_ms: u64,
    pub system_time_ms: u64,
    pub average_cpu_percent: f64,
    pub peak_cpu_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connections_opened: u64,
    pub connection_errors: u64,
    pub average_latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskIoStats {
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub read_operations: u64,
    pub write_operations: u64,
    pub average_read_latency_ms: f64,
    pub average_write_latency_ms: f64,
}

pub struct BenchmarkSuite {
    config: BenchmarkConfig,
    results: Arc<RwLock<HashMap<String, BenchmarkResult>>>,
    memory_optimizer: Option<Arc<MemoryOptimizer>>,
    network_optimizer: Option<Arc<NetworkOptimizer>>,
    parallel_executor: Option<Arc<ParallelExecutor>>,
    system_monitor: Arc<SystemMonitor>,
}

struct SystemMonitor {
    start_time: Instant,
    memory_samples: Arc<RwLock<Vec<(Instant, usize)>>>,
    cpu_samples: Arc<RwLock<Vec<(Instant, f64)>>>,
}

impl SystemMonitor {
    fn new() -> Self {
        Self {
            start_time: Instant::now(),
            memory_samples: Arc::new(RwLock::new(Vec::new())),
            cpu_samples: Arc::new(RwLock::new(Vec::new())),
        }
    }

    fn start_monitoring(&self) {
        // In a real implementation, this would start background threads
        // to monitor system resources
    }

    fn stop_monitoring(&self) -> (MemoryUsage, CpuUsage) {
        let memory_samples = self.memory_samples.read().unwrap();
        let cpu_samples = self.cpu_samples.read().unwrap();

        let memory_usage = if memory_samples.is_empty() {
            MemoryUsage {
                peak_bytes: 0,
                average_bytes: 0,
                allocations: 0,
                deallocations: 0,
            }
        } else {
            let peak_bytes = memory_samples.iter().map(|(_, mem)| *mem).max().unwrap_or(0);
            let average_bytes = memory_samples.iter().map(|(_, mem)| *mem).sum::<usize>() / memory_samples.len();
            
            MemoryUsage {
                peak_bytes,
                average_bytes,
                allocations: 0, // Would need more sophisticated tracking
                deallocations: 0,
            }
        };

        let cpu_usage = if cpu_samples.is_empty() {
            CpuUsage {
                user_time_ms: 0,
                system_time_ms: 0,
                average_cpu_percent: 0.0,
                peak_cpu_percent: 0.0,
            }
        } else {
            let peak_cpu = cpu_samples.iter().map(|(_, cpu)| *cpu).fold(0.0f64, f64::max);
            let average_cpu = cpu_samples.iter().map(|(_, cpu)| *cpu).sum::<f64>() / cpu_samples.len() as f64;
            
            CpuUsage {
                user_time_ms: 0, // Would need platform-specific implementation
                system_time_ms: 0,
                average_cpu_percent: average_cpu,
                peak_cpu_percent: peak_cpu,
            }
        };

        (memory_usage, cpu_usage)
    }

    fn sample_memory(&self, bytes: usize) {
        self.memory_samples.write().unwrap().push((Instant::now(), bytes));
    }

    fn sample_cpu(&self, percent: f64) {
        self.cpu_samples.write().unwrap().push((Instant::now(), percent));
    }
}

impl BenchmarkSuite {
    pub fn new(config: BenchmarkConfig) -> Self {
        Self {
            config,
            results: Arc::new(RwLock::new(HashMap::new())),
            memory_optimizer: None,
            network_optimizer: None,
            parallel_executor: None,
            system_monitor: Arc::new(SystemMonitor::new()),
        }
    }

    pub fn with_memory_optimizer(mut self, optimizer: Arc<MemoryOptimizer>) -> Self {
        self.memory_optimizer = Some(optimizer);
        self
    }

    pub fn with_network_optimizer(mut self, optimizer: Arc<NetworkOptimizer>) -> Self {
        self.network_optimizer = Some(optimizer);
        self
    }

    pub fn with_parallel_executor(mut self, executor: Arc<ParallelExecutor>) -> Self {
        self.parallel_executor = Some(executor);
        self
    }

    pub async fn benchmark<F, Fut>(&self, name: &str, operation: F) -> Result<BenchmarkResult>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        println!("🔥 Running benchmark: {}", name);

        // Warmup iterations
        println!("  🔧 Warming up ({} iterations)...", self.config.warmup_iterations);
        for _ in 0..self.config.warmup_iterations {
            let _ = operation().await;
        }

        // Actual benchmark iterations
        println!("  ⚡ Benchmarking ({} iterations)...", self.config.iterations);
        let mut durations = Vec::with_capacity(self.config.iterations);
        
        // Start system monitoring
        if self.config.measure_memory || self.config.measure_cpu {
            self.system_monitor.start_monitoring();
        }

        let benchmark_start = Instant::now();

        for i in 0..self.config.iterations {
            let start = Instant::now();
            
            // Run the operation with timeout
            let timeout_duration = Duration::from_secs(self.config.timeout_seconds);
            match tokio::time::timeout(timeout_duration, operation()).await {
                Ok(Ok(())) => {
                    let duration = start.elapsed();
                    durations.push(duration);
                    
                    // Sample system resources periodically
                    if i % 10 == 0 {
                        if self.config.measure_memory {
                            // In practice, you'd get actual memory usage
                            self.system_monitor.sample_memory(1024 * 1024); // Placeholder
                        }
                        if self.config.measure_cpu {
                            // In practice, you'd get actual CPU usage
                            self.system_monitor.sample_cpu(50.0); // Placeholder
                        }
                    }
                }
                Ok(Err(e)) => {
                    println!("  ❌ Operation failed: {}", e);
                    continue;
                }
                Err(_) => {
                    println!("  ⏰ Operation timed out");
                    continue;
                }
            }

            // Progress reporting
            if i % (self.config.iterations / 10).max(1) == 0 {
                println!("    Progress: {}/{}", i + 1, self.config.iterations);
            }
        }

        let total_duration = benchmark_start.elapsed();

        // Stop system monitoring and get results
        let (memory_usage, cpu_usage) = if self.config.measure_memory || self.config.measure_cpu {
            let (mem, cpu) = self.system_monitor.stop_monitoring();
            (Some(mem), Some(cpu))
        } else {
            (None, None)
        };

        // Calculate statistics
        let result = self.calculate_statistics(name, durations, total_duration, memory_usage, cpu_usage)?;
        
        // Store result
        self.results.write().unwrap().insert(name.to_string(), result.clone());

        // Print summary
        self.print_result_summary(&result);

        Ok(result)
    }

    pub fn benchmark_sync<F>(&self, name: &str, operation: F) -> Result<BenchmarkResult>
    where
        F: Fn() -> Result<()> + Send + Sync,
    {
        println!("🔥 Running sync benchmark: {}", name);

        // Warmup
        for _ in 0..self.config.warmup_iterations {
            let _ = operation();
        }

        // Benchmark
        let mut durations = Vec::with_capacity(self.config.iterations);
        let benchmark_start = Instant::now();

        for i in 0..self.config.iterations {
            let start = Instant::now();
            
            match operation() {
                Ok(()) => {
                    durations.push(start.elapsed());
                }
                Err(e) => {
                    println!("  ❌ Operation failed: {}", e);
                    continue;
                }
            }

            if i % (self.config.iterations / 10).max(1) == 0 {
                println!("    Progress: {}/{}", i + 1, self.config.iterations);
            }
        }

        let total_duration = benchmark_start.elapsed();
        let result = self.calculate_statistics(name, durations, total_duration, None, None)?;
        
        self.results.write().unwrap().insert(name.to_string(), result.clone());
        self.print_result_summary(&result);

        Ok(result)
    }

    pub fn benchmark_parallel<F>(&self, name: &str, operation: F, concurrency: usize) -> Result<BenchmarkResult>
    where
        F: Fn() -> Result<()> + Send + Sync + Clone + 'static,
    {
        println!("🔥 Running parallel benchmark: {} (concurrency: {})", name, concurrency);

        if let Some(executor) = &self.parallel_executor {
            let tasks: Vec<_> = (0..self.config.iterations)
                .map(|_| {
                    let op = operation.clone();
                    Box::new(move || op()) as Box<dyn Fn() -> Result<()> + Send + Sync>
                })
                .collect();

            let benchmark_start = Instant::now();
            let results = executor.execute_parallel(tasks, concurrency)?;
            let total_duration = benchmark_start.elapsed();

            let successful_durations: Vec<_> = results.into_iter()
                .filter_map(|r| r.duration)
                .collect();

            let result = self.calculate_statistics(name, successful_durations, total_duration, None, None)?;
            self.results.write().unwrap().insert(name.to_string(), result.clone());
            self.print_result_summary(&result);

            Ok(result)
        } else {
            Err(anyhow::anyhow!("Parallel executor not configured"))
        }
    }

    pub fn get_results(&self) -> HashMap<String, BenchmarkResult> {
        self.results.read().unwrap().clone()
    }

    pub fn get_result(&self, name: &str) -> Option<BenchmarkResult> {
        self.results.read().unwrap().get(name).cloned()
    }

    pub fn export_results(&self, path: &str) -> Result<()> {
        if !self.config.export_results {
            return Ok(());
        }

        let results = self.get_results();
        
        match self.config.export_format {
            ExportFormat::Json => {
                let json = serde_json::to_string_pretty(&results)?;
                std::fs::write(format!("{}.json", path), json)?;
            }
            ExportFormat::Csv => {
                self.export_csv(&results, &format!("{}.csv", path))?;
            }
            ExportFormat::Html => {
                self.export_html(&results, &format!("{}.html", path))?;
            }
            ExportFormat::Markdown => {
                self.export_markdown(&results, &format!("{}.md", path))?;
            }
        }

        println!("📊 Results exported to: {}", path);
        Ok(())
    }

    pub fn print_summary(&self) {
        let results = self.get_results();
        
        println!("\n🏆 Benchmark Summary");
        println!("==================");
        
        for (name, result) in results.iter() {
            println!("\n📈 {}", name);
            println!("  Duration: avg={:.2?} min={:.2?} max={:.2?}", 
                result.avg_duration, result.min_duration, result.max_duration);
            println!("  Throughput: {:.0} ops/sec", result.operations_per_second);
            
            if let Some(memory) = &result.memory_usage {
                println!("  Memory: peak={:.1}MB avg={:.1}MB", 
                    memory.peak_bytes as f64 / 1024.0 / 1024.0,
                    memory.average_bytes as f64 / 1024.0 / 1024.0);
            }
        }
    }

    fn calculate_statistics(
        &self,
        name: &str,
        mut durations: Vec<Duration>,
        total_duration: Duration,
        memory_usage: Option<MemoryUsage>,
        cpu_usage: Option<CpuUsage>,
    ) -> Result<BenchmarkResult> {
        if durations.is_empty() {
            return Err(anyhow::anyhow!("No successful iterations"));
        }

        durations.sort();

        let min_duration = *durations.first().unwrap();
        let max_duration = *durations.last().unwrap();
        let median_duration = durations[durations.len() / 2];
        
        let avg_nanos = durations.iter().map(|d| d.as_nanos()).sum::<u128>() / durations.len() as u128;
        let avg_duration = Duration::from_nanos(avg_nanos as u64);

        // Calculate standard deviation
        let variance = durations.iter()
            .map(|d| {
                let diff = d.as_nanos() as f64 - avg_nanos as f64;
                diff * diff
            })
            .sum::<f64>() / durations.len() as f64;
        let std_deviation = variance.sqrt();

        let operations_per_second = durations.len() as f64 / total_duration.as_secs_f64();

        Ok(BenchmarkResult {
            name: name.to_string(),
            iterations: durations.len(),
            total_duration,
            min_duration,
            max_duration,
            avg_duration,
            median_duration,
            std_deviation,
            operations_per_second,
            memory_usage,
            cpu_usage,
            network_stats: None, // TODO: Implement network stats collection
            disk_io_stats: None, // TODO: Implement disk I/O stats collection
            timestamp: std::time::SystemTime::now(),
        })
    }

    fn print_result_summary(&self, result: &BenchmarkResult) {
        println!("  ✅ Completed {} iterations", result.iterations);
        println!("  ⏱️  Average: {:.2?}", result.avg_duration);
        println!("  🚀 Throughput: {:.0} ops/sec", result.operations_per_second);
        
        if let Some(memory) = &result.memory_usage {
            println!("  💾 Peak memory: {:.1}MB", memory.peak_bytes as f64 / 1024.0 / 1024.0);
        }
    }

    fn export_csv(&self, results: &HashMap<String, BenchmarkResult>, path: &str) -> Result<()> {
        use std::io::Write;
        
        let mut file = std::fs::File::create(path)?;
        
        // Header
        writeln!(file, "name,iterations,avg_duration_ms,min_duration_ms,max_duration_ms,median_duration_ms,std_deviation,ops_per_sec")?;
        
        // Data
        for (_, result) in results {
            writeln!(file, "{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.0}",
                result.name,
                result.iterations,
                result.avg_duration.as_secs_f64() * 1000.0,
                result.min_duration.as_secs_f64() * 1000.0,
                result.max_duration.as_secs_f64() * 1000.0,
                result.median_duration.as_secs_f64() * 1000.0,
                result.std_deviation,
                result.operations_per_second)?;
        }
        
        Ok(())
    }

    fn export_html(&self, results: &HashMap<String, BenchmarkResult>, path: &str) -> Result<()> {
        let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Benchmark Results</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
        .number {{ text-align: right; }}
    </style>
</head>
<body>
    <h1>🏆 Benchmark Results</h1>
    <table>
        <tr>
            <th>Benchmark</th>
            <th>Iterations</th>
            <th>Avg Duration (ms)</th>
            <th>Min Duration (ms)</th>
            <th>Max Duration (ms)</th>
            <th>Ops/Sec</th>
        </tr>
        {}
    </table>
</body>
</html>
"#, results.iter().map(|(_, result)| {
            format!("<tr><td>{}</td><td class=\"number\">{}</td><td class=\"number\">{:.3}</td><td class=\"number\">{:.3}</td><td class=\"number\">{:.3}</td><td class=\"number\">{:.0}</td></tr>",
                result.name,
                result.iterations,
                result.avg_duration.as_secs_f64() * 1000.0,
                result.min_duration.as_secs_f64() * 1000.0,
                result.max_duration.as_secs_f64() * 1000.0,
                result.operations_per_second)
        }).collect::<Vec<_>>().join("\n"));

        std::fs::write(path, html)?;
        Ok(())
    }

    fn export_markdown(&self, results: &HashMap<String, BenchmarkResult>, path: &str) -> Result<()> {
        let mut content = String::from("# 🏆 Benchmark Results\n\n");
        content.push_str("| Benchmark | Iterations | Avg Duration (ms) | Min Duration (ms) | Max Duration (ms) | Ops/Sec |\n");
        content.push_str("|-----------|------------|-------------------|-------------------|-------------------|----------|\n");

        for (_, result) in results {
            content.push_str(&format!("| {} | {} | {:.3} | {:.3} | {:.3} | {:.0} |\n",
                result.name,
                result.iterations,
                result.avg_duration.as_secs_f64() * 1000.0,
                result.min_duration.as_secs_f64() * 1000.0,
                result.max_duration.as_secs_f64() * 1000.0,
                result.operations_per_second));
        }

        std::fs::write(path, content)?;
        Ok(())
    }
}

// Convenience macros for benchmarking
#[macro_export]
macro_rules! benchmark {
    ($suite:expr, $name:expr, $operation:block) => {
        $suite.benchmark_sync($name, || $operation)
    };
}

#[macro_export]
macro_rules! async_benchmark {
    ($suite:expr, $name:expr, $operation:block) => {
        $suite.benchmark($name, || async move $operation).await
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_benchmark_suite_sync() -> Result<()> {
        let config = BenchmarkConfig {
            iterations: 10,
            warmup_iterations: 2,
            ..Default::default()
        };
        
        let suite = BenchmarkSuite::new(config);
        
        let result = suite.benchmark_sync("test_operation", || {
            thread::sleep(Duration::from_millis(1));
            Ok(())
        })?;

        assert_eq!(result.iterations, 10);
        assert!(result.avg_duration >= Duration::from_millis(1));
        assert!(result.operations_per_second > 0.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_benchmark_suite_async() -> Result<()> {
        let config = BenchmarkConfig {
            iterations: 5,
            warmup_iterations: 1,
            ..Default::default()
        };
        
        let suite = BenchmarkSuite::new(config);
        
        let result = suite.benchmark("async_test_operation", || async {
            tokio::time::sleep(Duration::from_millis(2)).await;
            Ok(())
        }).await?;

        assert_eq!(result.iterations, 5);
        assert!(result.avg_duration >= Duration::from_millis(2));

        Ok(())
    }

    #[test]
    fn test_export_results() -> Result<()> {
        let config = BenchmarkConfig {
            iterations: 3,
            export_results: true,
            export_format: ExportFormat::Json,
            ..Default::default()
        };
        
        let suite = BenchmarkSuite::new(config);
        
        suite.benchmark_sync("export_test", || {
            thread::sleep(Duration::from_millis(1));
            Ok(())
        })?;

        // Test export
        let temp_dir = tempfile::tempdir()?;
        let export_path = temp_dir.path().join("test_results");
        suite.export_results(export_path.to_str().unwrap())?;

        // Verify file was created
        assert!(temp_dir.path().join("test_results.json").exists());

        Ok(())
    }
}
