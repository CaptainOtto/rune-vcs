use anyhow::Result;
use memmap2::{Mmap, MmapOptions};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub enable_memory_mapping: bool,
    pub mmap_threshold_bytes: usize,
    pub buffer_size: usize,
    pub max_open_files: usize,
    pub enable_read_ahead: bool,
    pub read_ahead_size: usize,
    pub enable_write_behind: bool,
    pub write_behind_delay_ms: u64,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            enable_memory_mapping: true,
            mmap_threshold_bytes: 64 * 1024, // 64KB
            buffer_size: 8 * 1024, // 8KB
            max_open_files: 1000,
            enable_read_ahead: true,
            read_ahead_size: 64 * 1024, // 64KB
            enable_write_behind: true,
            write_behind_delay_ms: 100,
        }
    }
}

pub struct MemoryOptimizer {
    config: MemoryConfig,
    mmap_cache: Arc<RwLock<HashMap<PathBuf, Arc<Mmap>>>>,
    file_handles: Arc<Mutex<lru::LruCache<PathBuf, File>>>,
    read_ahead_cache: Arc<RwLock<HashMap<PathBuf, Vec<u8>>>>,
    write_behind_queue: Arc<Mutex<HashMap<PathBuf, Vec<u8>>>>,
    metrics: Arc<RwLock<MemoryMetrics>>,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryMetrics {
    pub mmap_hits: u64,
    pub mmap_misses: u64,
    pub read_ahead_hits: u64,
    pub read_ahead_misses: u64,
    pub write_behind_flushes: u64,
    pub total_bytes_read: u64,
    pub total_bytes_written: u64,
    pub file_handles_opened: u64,
    pub file_handles_closed: u64,
    pub memory_mapped_files: usize,
    pub peak_memory_usage: usize,
}

impl MemoryMetrics {
    pub fn mmap_hit_ratio(&self) -> f64 {
        let total = self.mmap_hits + self.mmap_misses;
        if total == 0 {
            0.0
        } else {
            self.mmap_hits as f64 / total as f64
        }
    }

    pub fn read_ahead_hit_ratio(&self) -> f64 {
        let total = self.read_ahead_hits + self.read_ahead_misses;
        if total == 0 {
            0.0
        } else {
            self.read_ahead_hits as f64 / total as f64
        }
    }
}

impl MemoryOptimizer {
    pub fn new(config: MemoryConfig) -> Result<Self> {
        let max_files = std::num::NonZeroUsize::new(config.max_open_files).unwrap();
        
        Ok(Self {
            config,
            mmap_cache: Arc::new(RwLock::new(HashMap::new())),
            file_handles: Arc::new(Mutex::new(lru::LruCache::new(max_files))),
            read_ahead_cache: Arc::new(RwLock::new(HashMap::new())),
            write_behind_queue: Arc::new(Mutex::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(MemoryMetrics::default())),
        })
    }

    pub fn read_file_optimized<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>> {
        let path = path.as_ref().to_path_buf();
        let file_size = std::fs::metadata(&path)?.len() as usize;

        // Check read-ahead cache first
        if let Some(data) = self.check_read_ahead_cache(&path) {
            self.metrics.write().unwrap().read_ahead_hits += 1;
            return Ok(data);
        }
        self.metrics.write().unwrap().read_ahead_misses += 1;

        // Use memory mapping for large files
        if file_size >= self.config.mmap_threshold_bytes && self.config.enable_memory_mapping {
            return self.read_with_mmap(&path);
        }

        // Use buffered reading for smaller files
        self.read_with_buffer(&path)
    }

    pub fn write_file_optimized<P: AsRef<Path>>(&self, path: P, data: &[u8]) -> Result<()> {
        let path = path.as_ref().to_path_buf();

        if self.config.enable_write_behind {
            // Queue for write-behind
            self.queue_write_behind(path, data.to_vec())?;
        } else {
            // Write immediately
            self.write_immediately(&path, data)?;
        }

        self.metrics.write().unwrap().total_bytes_written += data.len() as u64;
        Ok(())
    }

    pub fn read_range_optimized<P: AsRef<Path>>(
        &self,
        path: P,
        offset: u64,
        length: usize,
    ) -> Result<Vec<u8>> {
        let path = path.as_ref().to_path_buf();
        let file_size = std::fs::metadata(&path)?.len();

        // Use memory mapping for range reads on large files
        if file_size >= self.config.mmap_threshold_bytes as u64 && self.config.enable_memory_mapping {
            return self.read_range_with_mmap(&path, offset, length);
        }

        // Use seek + read for smaller files
        self.read_range_with_seek(&path, offset, length)
    }

    pub fn stream_reader<P: AsRef<Path>>(&self, path: P) -> Result<OptimizedReader> {
        let path = path.as_ref().to_path_buf();
        let file = self.get_file_handle(&path)?;
        
        Ok(OptimizedReader::new(
            file,
            self.config.buffer_size,
            self.config.enable_read_ahead,
            self.config.read_ahead_size,
        ))
    }

    pub fn stream_writer<P: AsRef<Path>>(&self, path: P) -> Result<OptimizedWriter> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        
        Ok(OptimizedWriter::new(
            file,
            self.config.buffer_size,
            self.config.enable_write_behind,
            self.config.write_behind_delay_ms,
        ))
    }

    pub fn flush_write_behind(&self) -> Result<()> {
        let queue = {
            let mut queue = self.write_behind_queue.lock();
            std::mem::take(&mut *queue)
        };

        for (path, data) in queue {
            self.write_immediately(&path, &data)?;
            self.metrics.write().unwrap().write_behind_flushes += 1;
        }

        Ok(())
    }

    pub fn clear_caches(&self) {
        self.mmap_cache.write().unwrap().clear();
        self.read_ahead_cache.write().unwrap().clear();
        self.file_handles.lock().clear();
        self.write_behind_queue.lock().clear();
    }

    pub fn get_metrics(&self) -> MemoryMetrics {
        self.metrics.read().unwrap().clone()
    }

    pub fn optimize_memory_usage(&self) -> Result<()> {
        // Clear least recently used items from caches
        self.trim_caches();
        
        // Force garbage collection of unused memory maps
        self.cleanup_unused_mmaps();
        
        // Flush any pending writes
        self.flush_write_behind()?;
        
        Ok(())
    }

    fn read_with_mmap(&self, path: &Path) -> Result<Vec<u8>> {
        // Check if already memory mapped
        if let Some(mmap) = self.mmap_cache.read().unwrap().get(path) {
            self.metrics.write().unwrap().mmap_hits += 1;
            return Ok(mmap.as_ref().to_vec());
        }

        self.metrics.write().unwrap().mmap_misses += 1;

        // Create new memory map
        let file = File::open(path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        let data = mmap.as_ref().to_vec();
        
        // Cache the memory map
        self.mmap_cache.write().unwrap().insert(path.to_path_buf(), Arc::new(mmap));
        self.metrics.write().unwrap().memory_mapped_files += 1;

        Ok(data)
    }

    fn read_with_buffer(&self, path: &Path) -> Result<Vec<u8>> {
        let file = self.get_file_handle(path)?;
        let mut reader = BufReader::with_capacity(self.config.buffer_size, file);
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        self.metrics.write().unwrap().total_bytes_read += data.len() as u64;

        // Trigger read-ahead if enabled
        if self.config.enable_read_ahead {
            self.trigger_read_ahead(path);
        }

        Ok(data)
    }

    fn read_range_with_mmap(&self, path: &Path, offset: u64, length: usize) -> Result<Vec<u8>> {
        // Check if already memory mapped
        if let Some(mmap) = self.mmap_cache.read().unwrap().get(path) {
            let start = offset as usize;
            let end = std::cmp::min(start + length, mmap.len());
            return Ok(mmap[start..end].to_vec());
        }

        // Create new memory map
        let file = File::open(path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        
        let start = offset as usize;
        let end = std::cmp::min(start + length, mmap.len());
        let data = mmap[start..end].to_vec();

        // Cache the memory map
        self.mmap_cache.write().unwrap().insert(path.to_path_buf(), Arc::new(mmap));

        Ok(data)
    }

    fn read_range_with_seek(&self, path: &Path, offset: u64, length: usize) -> Result<Vec<u8>> {
        let mut file = self.get_file_handle(path)?;
        file.seek(SeekFrom::Start(offset))?;
        
        let mut buffer = vec![0u8; length];
        let bytes_read = file.read(&mut buffer)?;
        buffer.truncate(bytes_read);

        self.metrics.write().unwrap().total_bytes_read += bytes_read as u64;
        Ok(buffer)
    }

    fn get_file_handle(&self, path: &Path) -> Result<File> {
        let path_buf = path.to_path_buf();
        
        // Check cache first
        if let Some(file) = self.file_handles.lock().get(&path_buf) {
            // Clone the file handle (this creates a new handle to the same file)
            return Ok(file.try_clone()?);
        }

        // Open new file
        let file = File::open(path)?;
        
        // Cache the handle
        if let Some(_evicted) = self.file_handles.lock().push(path_buf, file.try_clone()?) {
            self.metrics.write().unwrap().file_handles_closed += 1;
        }
        self.metrics.write().unwrap().file_handles_opened += 1;

        Ok(file)
    }

    fn write_immediately(&self, path: &Path, data: &[u8]) -> Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        
        let mut writer = BufWriter::with_capacity(self.config.buffer_size, file);
        writer.write_all(data)?;
        writer.flush()?;

        Ok(())
    }

    fn queue_write_behind(&self, path: PathBuf, data: Vec<u8>) -> Result<()> {
        self.write_behind_queue.lock().insert(path, data);
        
        // TODO: Implement background flush timer
        // For now, we'll flush immediately if queue is getting large
        if self.write_behind_queue.lock().len() > 100 {
            self.flush_write_behind()?;
        }

        Ok(())
    }

    fn check_read_ahead_cache(&self, path: &Path) -> Option<Vec<u8>> {
        self.read_ahead_cache.read().unwrap().get(path).cloned()
    }

    fn trigger_read_ahead(&self, _path: &Path) {
        // TODO: Implement intelligent read-ahead based on access patterns
        // This could prefetch related files or upcoming data
    }

    fn trim_caches(&self) {
        // Keep only most recently used items in caches
        let mut mmap_cache = self.mmap_cache.write().unwrap();
        if mmap_cache.len() > 100 {
            // Simple strategy: remove half of the entries
            let keys_to_remove: Vec<_> = mmap_cache.keys().take(50).cloned().collect();
            for key in keys_to_remove {
                mmap_cache.remove(&key);
            }
        }

        let mut read_ahead = self.read_ahead_cache.write().unwrap();
        if read_ahead.len() > 50 {
            let keys_to_remove: Vec<_> = read_ahead.keys().take(25).cloned().collect();
            for key in keys_to_remove {
                read_ahead.remove(&key);
            }
        }
    }

    fn cleanup_unused_mmaps(&self) {
        // Remove memory maps that are only referenced by the cache
        let mut cache = self.mmap_cache.write().unwrap();
        cache.retain(|_, mmap| Arc::strong_count(mmap) > 1);
    }
}

pub struct OptimizedReader {
    inner: BufReader<File>,
    read_ahead_enabled: bool,
    read_ahead_size: usize,
    read_ahead_buffer: Vec<u8>,
    read_ahead_pos: usize,
}

impl OptimizedReader {
    fn new(file: File, buffer_size: usize, read_ahead: bool, read_ahead_size: usize) -> Self {
        Self {
            inner: BufReader::with_capacity(buffer_size, file),
            read_ahead_enabled: read_ahead,
            read_ahead_size,
            read_ahead_buffer: Vec::new(),
            read_ahead_pos: 0,
        }
    }

    pub fn read_exact_optimized(&mut self, buf: &mut [u8]) -> io::Result<()> {
        if self.read_ahead_enabled {
            self.ensure_read_ahead_buffer(buf.len())?;
            
            if self.read_ahead_pos + buf.len() <= self.read_ahead_buffer.len() {
                buf.copy_from_slice(&self.read_ahead_buffer[self.read_ahead_pos..self.read_ahead_pos + buf.len()]);
                self.read_ahead_pos += buf.len();
                return Ok(());
            }
        }

        self.inner.read_exact(buf)
    }

    fn ensure_read_ahead_buffer(&mut self, needed: usize) -> io::Result<()> {
        if self.read_ahead_buffer.len() - self.read_ahead_pos < needed {
            // Shift remaining data to beginning and read more
            if self.read_ahead_pos > 0 {
                self.read_ahead_buffer.drain(0..self.read_ahead_pos);
                self.read_ahead_pos = 0;
            }

            let additional_needed = std::cmp::max(needed, self.read_ahead_size);
            let mut additional_buffer = vec![0u8; additional_needed];
            let bytes_read = self.inner.read(&mut additional_buffer)?;
            additional_buffer.truncate(bytes_read);
            self.read_ahead_buffer.extend(additional_buffer);
        }
        Ok(())
    }
}

impl Read for OptimizedReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.read_ahead_enabled && !self.read_ahead_buffer.is_empty() {
            let available = self.read_ahead_buffer.len() - self.read_ahead_pos;
            if available > 0 {
                let to_copy = std::cmp::min(buf.len(), available);
                buf[..to_copy].copy_from_slice(&self.read_ahead_buffer[self.read_ahead_pos..self.read_ahead_pos + to_copy]);
                self.read_ahead_pos += to_copy;
                return Ok(to_copy);
            }
        }

        self.inner.read(buf)
    }
}

pub struct OptimizedWriter {
    inner: BufWriter<File>,
    write_behind_enabled: bool,
    write_behind_delay: Duration,
    last_write: Instant,
}

impl OptimizedWriter {
    fn new(file: File, buffer_size: usize, write_behind: bool, delay_ms: u64) -> Self {
        Self {
            inner: BufWriter::with_capacity(buffer_size, file),
            write_behind_enabled: write_behind,
            write_behind_delay: Duration::from_millis(delay_ms),
            last_write: Instant::now(),
        }
    }

    pub fn write_optimized(&mut self, buf: &[u8]) -> io::Result<usize> {
        let result = self.inner.write(buf)?;
        self.last_write = Instant::now();

        if !self.write_behind_enabled {
            self.inner.flush()?;
        }

        Ok(result)
    }

    pub fn should_flush(&self) -> bool {
        !self.write_behind_enabled || 
        self.last_write.elapsed() >= self.write_behind_delay
    }
}

impl Write for OptimizedWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_optimized(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_memory_optimizer_basic() -> Result<()> {
        let config = MemoryConfig::default();
        let optimizer = MemoryOptimizer::new(config)?;

        // Create test file
        let mut temp_file = NamedTempFile::new()?;
        let test_data = b"Hello, World! This is a test file.";
        temp_file.write_all(test_data)?;

        // Test optimized read
        let data = optimizer.read_file_optimized(temp_file.path())?;
        assert_eq!(data, test_data);

        // Test metrics
        let metrics = optimizer.get_metrics();
        assert!(metrics.total_bytes_read > 0);

        Ok(())
    }

    #[test]
    fn test_memory_mapping() -> Result<()> {
        let mut config = MemoryConfig::default();
        config.mmap_threshold_bytes = 10; // Low threshold for testing
        
        let optimizer = MemoryOptimizer::new(config)?;

        // Create test file larger than threshold
        let mut temp_file = NamedTempFile::new()?;
        let test_data = b"This is a larger test file that should trigger memory mapping.";
        temp_file.write_all(test_data)?;

        // Test memory mapped read
        let data = optimizer.read_file_optimized(temp_file.path())?;
        assert_eq!(data, test_data);

        // Should have used memory mapping
        let metrics = optimizer.get_metrics();
        assert!(metrics.memory_mapped_files > 0);

        Ok(())
    }

    #[test]
    fn test_range_reading() -> Result<()> {
        let config = MemoryConfig::default();
        let optimizer = MemoryOptimizer::new(config)?;

        // Create test file
        let mut temp_file = NamedTempFile::new()?;
        let test_data = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        temp_file.write_all(test_data)?;

        // Test range read
        let range_data = optimizer.read_range_optimized(temp_file.path(), 10, 10)?;
        assert_eq!(range_data, b"ABCDEFGHIJ");

        Ok(())
    }

    #[test]
    fn test_optimized_streams() -> Result<()> {
        let config = MemoryConfig::default();
        let optimizer = MemoryOptimizer::new(config)?;

        // Create test file
        let mut temp_file = NamedTempFile::new()?;
        let test_data = b"Stream test data for reading and writing";
        temp_file.write_all(test_data)?;

        // Test optimized reader
        let mut reader = optimizer.stream_reader(temp_file.path())?;
        let mut buffer = vec![0u8; test_data.len()];
        reader.read_exact_optimized(&mut buffer)?;
        assert_eq!(buffer, test_data);

        // Test optimized writer
        let temp_write_file = NamedTempFile::new()?;
        let mut writer = optimizer.stream_writer(temp_write_file.path())?;
        writer.write_optimized(test_data)?;
        writer.flush()?;

        Ok(())
    }
}
