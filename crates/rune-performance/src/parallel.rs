use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use flume::{Receiver, Sender};
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct ParallelConfig {
    pub max_threads: usize,
    pub chunk_size: usize,
    pub queue_size: usize,
    pub enable_work_stealing: bool,
    pub thread_stack_size: Option<usize>,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            max_threads: num_cpus::get(),
            chunk_size: 1000,
            queue_size: 10000,
            enable_work_stealing: true,
            thread_stack_size: Some(2 * 1024 * 1024), // 2MB stack
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskMetrics {
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub total_processing_time: std::time::Duration,
    pub average_task_time: std::time::Duration,
    pub threads_used: usize,
}

pub struct ParallelExecutor {
    config: ParallelConfig,
    thread_pool: Arc<rayon::ThreadPool>,
    metrics: Arc<RwLock<TaskMetrics>>,
    task_sender: Sender<Box<dyn FnOnce() + Send + 'static>>,
    task_receiver: Receiver<Box<dyn FnOnce() + Send + 'static>>,
}

impl ParallelExecutor {
    pub fn new(config: ParallelConfig) -> Result<Self> {
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(config.max_threads)
            .stack_size(config.thread_stack_size.unwrap_or(2 * 1024 * 1024))
            .build()?;

        let (task_sender, task_receiver) = flume::bounded(config.queue_size);

        Ok(Self {
            config,
            thread_pool: Arc::new(thread_pool),
            metrics: Arc::new(RwLock::new(TaskMetrics {
                tasks_completed: 0,
                tasks_failed: 0,
                total_processing_time: std::time::Duration::ZERO,
                average_task_time: std::time::Duration::ZERO,
                threads_used: 0,
            })),
            task_sender,
            task_receiver,
        })
    }

    pub fn execute_parallel<T, F, R>(&self, items: Vec<T>, operation: F) -> Result<Vec<R>>
    where
        T: Send + Sync,
        F: Fn(&T) -> Result<R> + Send + Sync,
        R: Send,
    {
        let start_time = std::time::Instant::now();
        info!("Starting parallel execution of {} items", items.len());

        let results: Result<Vec<_>, _> = self.thread_pool.install(|| {
            items
                .par_chunks(self.config.chunk_size)
                .map(|chunk| {
                    chunk
                        .iter()
                        .map(&operation)
                        .collect::<Result<Vec<_>, _>>()
                })
                .collect::<Result<Vec<_>, _>>()
                .map(|chunks| chunks.into_iter().flatten().collect())
        });

        let processing_time = start_time.elapsed();
        self.update_metrics(items.len(), processing_time, results.is_ok());

        info!(
            "Parallel execution completed in {:?} for {} items", 
            processing_time, 
            items.len()
        );

        results
    }

    pub fn execute_parallel_diff(&self, file_pairs: Vec<(PathBuf, PathBuf)>) -> Result<Vec<FileDiff>> {
        debug!("Computing diffs for {} file pairs", file_pairs.len());
        
        self.execute_parallel(file_pairs, |pair| {
            let (old_path, new_path) = pair;
            self.compute_file_diff(old_path, new_path)
        })
    }

    pub fn execute_parallel_hash(&self, files: Vec<PathBuf>) -> Result<Vec<FileHash>> {
        debug!("Computing hashes for {} files", files.len());
        
        self.execute_parallel(files, |file_path| {
            self.compute_file_hash(file_path)
        })
    }

    pub fn execute_parallel_compression(&self, files: Vec<PathBuf>) -> Result<Vec<CompressionResult>> {
        debug!("Compressing {} files", files.len());
        
        self.execute_parallel(files, |file_path| {
            self.compress_file(file_path)
        })
    }

    pub fn spawn_background_task<F>(&self, task: F) -> Result<()>
    where
        F: FnOnce() + Send + 'static,
    {
        self.task_sender.send(Box::new(task))?;
        Ok(())
    }

    pub fn process_background_tasks(&self) -> Result<()> {
        while let Ok(task) = self.task_receiver.try_recv() {
            self.thread_pool.spawn(move || {
                task();
            });
        }
        Ok(())
    }

    fn compute_file_diff(&self, old_path: &Path, new_path: &Path) -> Result<FileDiff> {
        let old_content = std::fs::read_to_string(old_path)?;
        let new_content = std::fs::read_to_string(new_path)?;
        
        // Simplified diff computation - in real implementation, use more sophisticated algorithm
        let lines_added = new_content.lines().count().saturating_sub(old_content.lines().count());
        let lines_removed = old_content.lines().count().saturating_sub(new_content.lines().count());
        
        Ok(FileDiff {
            old_path: old_path.to_path_buf(),
            new_path: new_path.to_path_buf(),
            lines_added,
            lines_removed,
            is_binary: false,
        })
    }

    fn compute_file_hash(&self, file_path: &Path) -> Result<FileHash> {
        let content = std::fs::read(file_path)?;
        let hash = blake3::hash(&content);
        
        Ok(FileHash {
            path: file_path.to_path_buf(),
            hash: hash.to_hex().to_string(),
            size: content.len(),
        })
    }

    fn compress_file(&self, file_path: &Path) -> Result<CompressionResult> {
        let content = std::fs::read(file_path)?;
        let original_size = content.len();
        
        let compressed = zstd::bulk::compress(&content, 3)?;
        let compressed_size = compressed.len();
        
        let compression_ratio = if original_size > 0 {
            compressed_size as f64 / original_size as f64
        } else {
            1.0
        };

        Ok(CompressionResult {
            path: file_path.to_path_buf(),
            original_size,
            compressed_size,
            compression_ratio,
            compressed_data: compressed,
        })
    }

    fn update_metrics(&self, task_count: usize, processing_time: std::time::Duration, success: bool) {
        let mut metrics = self.metrics.write();
        
        if success {
            metrics.tasks_completed += task_count as u64;
        } else {
            metrics.tasks_failed += task_count as u64;
        }
        
        metrics.total_processing_time += processing_time;
        let total_tasks = metrics.tasks_completed + metrics.tasks_failed;
        
        if total_tasks > 0 {
            metrics.average_task_time = metrics.total_processing_time / total_tasks as u32;
        }
        
        metrics.threads_used = self.config.max_threads;
    }

    pub fn get_metrics(&self) -> TaskMetrics {
        self.metrics.read().clone()
    }

    pub fn reset_metrics(&self) {
        let mut metrics = self.metrics.write();
        *metrics = TaskMetrics {
            tasks_completed: 0,
            tasks_failed: 0,
            total_processing_time: std::time::Duration::ZERO,
            average_task_time: std::time::Duration::ZERO,
            threads_used: 0,
        };
    }
}

#[derive(Debug, Clone)]
pub struct FileDiff {
    pub old_path: PathBuf,
    pub new_path: PathBuf,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub is_binary: bool,
}

#[derive(Debug, Clone)]
pub struct FileHash {
    pub path: PathBuf,
    pub hash: String,
    pub size: usize,
}

#[derive(Debug, Clone)]
pub struct CompressionResult {
    pub path: PathBuf,
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f64,
    pub compressed_data: Vec<u8>,
}

// Utility functions for parallel processing
pub fn parallel_map<T, R, F>(items: Vec<T>, func: F) -> Vec<R>
where
    T: Send + Sync,
    R: Send,
    F: Fn(T) -> R + Send + Sync,
{
    items.into_par_iter().map(func).collect()
}

pub fn parallel_filter_map<T, R, F>(items: Vec<T>, func: F) -> Vec<R>
where
    T: Send + Sync,
    R: Send,
    F: Fn(T) -> Option<R> + Send + Sync,
{
    items.into_par_iter().filter_map(func).collect()
}

pub fn parallel_reduce<T, F, R>(items: Vec<T>, identity: R, func: F) -> R
where
    T: Send + Sync,
    R: Send + Clone,
    F: Fn(R, T) -> R + Send + Sync,
{
    items.into_par_iter().fold(|| identity.clone(), func).reduce(|| identity, |a, b| func(a.clone(), b.clone()))
}

// Benchmarking utilities
pub struct PerformanceBenchmark {
    name: String,
    iterations: usize,
    warmup_iterations: usize,
}

impl PerformanceBenchmark {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            iterations: 1000,
            warmup_iterations: 100,
        }
    }

    pub fn with_iterations(mut self, iterations: usize) -> Self {
        self.iterations = iterations;
        self
    }

    pub fn with_warmup(mut self, warmup: usize) -> Self {
        self.warmup_iterations = warmup;
        self
    }

    pub fn run<F>(&self, mut operation: F) -> BenchmarkResult
    where
        F: FnMut() -> (),
    {
        // Warmup
        for _ in 0..self.warmup_iterations {
            operation();
        }

        let start_time = std::time::Instant::now();
        
        for _ in 0..self.iterations {
            operation();
        }
        
        let total_time = start_time.elapsed();
        let avg_time = total_time / self.iterations as u32;

        BenchmarkResult {
            name: self.name.clone(),
            iterations: self.iterations,
            total_time,
            average_time: avg_time,
            operations_per_second: if avg_time.as_nanos() > 0 {
                1_000_000_000.0 / avg_time.as_nanos() as f64
            } else {
                f64::INFINITY
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: usize,
    pub total_time: std::time::Duration,
    pub average_time: std::time::Duration,
    pub operations_per_second: f64,
}

impl BenchmarkResult {
    pub fn print_results(&self) {
        println!("Benchmark: {}", self.name);
        println!("  Iterations: {}", self.iterations);
        println!("  Total time: {:?}", self.total_time);
        println!("  Average time: {:?}", self.average_time);
        println!("  Operations/sec: {:.2}", self.operations_per_second);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parallel_executor_creation() {
        let config = ParallelConfig::default();
        let executor = ParallelExecutor::new(config).unwrap();
        assert!(executor.get_metrics().tasks_completed == 0);
    }

    #[test]
    fn test_parallel_map() {
        let numbers = vec![1, 2, 3, 4, 5];
        let results = parallel_map(numbers, |x| x * 2);
        assert_eq!(results, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_parallel_hash_computation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "test content").unwrap();

        let executor = ParallelExecutor::new(ParallelConfig::default()).unwrap();
        let results = executor.execute_parallel_hash(vec![file_path.clone()]).unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, file_path);
        assert!(!results[0].hash.is_empty());
    }

    #[test]
    fn test_benchmark() {
        let benchmark = PerformanceBenchmark::new("test_operation")
            .with_iterations(10)
            .with_warmup(2);

        let result = benchmark.run(|| {
            // Simulate some work
            std::thread::sleep(std::time::Duration::from_millis(1));
        });

        assert_eq!(result.name, "test_operation");
        assert_eq!(result.iterations, 10);
        assert!(result.total_time.as_millis() >= 10);
    }
}
