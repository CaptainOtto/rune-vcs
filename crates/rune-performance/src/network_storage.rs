// Network and Storage optimization module for Rune VCS
// Provides delta compression v2.0, streaming transfers, and bandwidth management

use anyhow::Result;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::path::Path;
use std::io::Write;
use flate2::{Compression, write::GzEncoder};
use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};

/// Network and storage optimization engine
pub struct NetworkStorageEngine {
    /// Compression settings
    compression_level: Compression,
    /// Bandwidth limiter (bytes per second)
    bandwidth_limit: Option<u64>,
    /// Transfer statistics
    stats: Arc<Mutex<TransferStats>>,
    /// Delta compression cache
    delta_cache: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    /// Chunk size for streaming transfers
    chunk_size: usize,
}

/// Transfer and compression statistics
#[derive(Debug, Clone, Default)]
pub struct TransferStats {
    pub total_bytes_compressed: u64,
    pub total_bytes_original: u64,
    pub compression_ratio: f64,
    pub transfer_speed_mbps: f64,
    pub chunks_processed: u64,
    pub delta_cache_hits: u64,
    pub bandwidth_savings_mb: f64,
}

/// Delta compression result
#[derive(Debug)]
pub struct DeltaResult {
    pub compressed_data: Vec<u8>,
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f64,
    pub delta_type: DeltaType,
}

/// Type of delta compression applied
#[derive(Debug, Clone)]
pub enum DeltaType {
    /// Simple gzip compression
    Gzip,
    /// Binary delta compression
    BinaryDelta,
    /// Text-based delta compression
    TextDelta,
    /// No compression (file too small or incompressible)
    None,
}

/// Streaming transfer configuration
#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub chunk_size: usize,
    pub buffer_size: usize,
    pub enable_compression: bool,
    pub bandwidth_limit_mbps: Option<f64>,
    pub enable_progress: bool,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            chunk_size: 64 * 1024, // 64KB chunks
            buffer_size: 1024 * 1024, // 1MB buffer
            enable_compression: true,
            bandwidth_limit_mbps: None,
            enable_progress: true,
        }
    }
}

impl NetworkStorageEngine {
    /// Create a new network storage optimization engine
    pub fn new() -> Self {
        Self {
            compression_level: Compression::default(),
            bandwidth_limit: None,
            stats: Arc::new(Mutex::new(TransferStats::default())),
            delta_cache: Arc::new(Mutex::new(HashMap::new())),
            chunk_size: 64 * 1024, // 64KB chunks
        }
    }

    /// Create with custom bandwidth limit (bytes per second)
    pub fn with_bandwidth_limit(bandwidth_mbps: f64) -> Self {
        let mut engine = Self::new();
        engine.bandwidth_limit = Some((bandwidth_mbps * 1024.0 * 1024.0) as u64);
        engine
    }

    /// Set compression level (0-9, where 9 is maximum compression)
    pub fn set_compression_level(&mut self, level: u32) {
        self.compression_level = Compression::new(level.min(9));
    }

    /// Delta compression v2.0 with improved algorithms
    pub fn delta_compress_v2(&self, file_path: &Path, reference_data: Option<&[u8]>) -> Result<DeltaResult> {
        let start = Instant::now();
        let data = std::fs::read(file_path)?;
        let original_size = data.len();

        // Determine the best compression strategy based on file type and size
        let delta_result = if original_size < 1024 {
            // Small files: no compression
            DeltaResult {
                compressed_data: data.clone(),
                original_size,
                compressed_size: original_size,
                compression_ratio: 1.0,
                delta_type: DeltaType::None,
            }
        } else if let Some(reference) = reference_data {
            // Binary delta compression when reference data is available
            self.binary_delta_compress(&data, reference)?
        } else if self.is_text_file(&data) {
            // Text-based delta compression
            self.text_delta_compress(&data)?
        } else {
            // Generic gzip compression
            self.gzip_compress(&data)?
        };

        // Update statistics
        let mut stats = self.stats.lock().unwrap();
        stats.total_bytes_original += original_size as u64;
        stats.total_bytes_compressed += delta_result.compressed_size as u64;
        stats.compression_ratio = stats.total_bytes_compressed as f64 / stats.total_bytes_original as f64;

        println!("🗜️  Delta compression v2.0: {} → {} ({:.1}% reduction) in {:.2}ms", 
                 self.format_bytes(original_size),
                 self.format_bytes(delta_result.compressed_size),
                 (1.0 - delta_result.compression_ratio) * 100.0,
                 start.elapsed().as_millis());

        Ok(delta_result)
    }

    /// Binary delta compression using XOR-based algorithm
    fn binary_delta_compress(&self, data: &[u8], reference: &[u8]) -> Result<DeltaResult> {
        let mut compressed = Vec::new();
        let min_len = data.len().min(reference.len());
        
        // Simple XOR-based delta compression
        for i in 0..min_len {
            compressed.push(data[i] ^ reference[i]);
        }
        
        // Append remaining bytes if data is longer
        if data.len() > min_len {
            compressed.extend_from_slice(&data[min_len..]);
        }
        
        // Apply gzip to the delta
        let final_compressed = self.apply_gzip(&compressed)?;
        
        Ok(DeltaResult {
            compressed_data: final_compressed.clone(),
            original_size: data.len(),
            compressed_size: final_compressed.len(),
            compression_ratio: final_compressed.len() as f64 / data.len() as f64,
            delta_type: DeltaType::BinaryDelta,
        })
    }

    /// Text-based delta compression with line-based diffing
    fn text_delta_compress(&self, data: &[u8]) -> Result<DeltaResult> {
        // Convert to string and apply text-based compression
        let text = String::from_utf8_lossy(data);
        let lines: Vec<&str> = text.lines().collect();
        
        // Simple line deduplication and compression
        let mut unique_lines = Vec::new();
        let mut line_map = HashMap::new();
        let mut indices = Vec::new();
        
        for line in lines {
            if let Some(&index) = line_map.get(line) {
                indices.push(index);
            } else {
                let index = unique_lines.len();
                unique_lines.push(line);
                line_map.insert(line, index);
                indices.push(index);
            }
        }
        
        // Serialize the compressed representation
        let mut compressed = Vec::new();
        
        // Write unique lines
        for line in unique_lines {
            compressed.extend_from_slice(&(line.len() as u32).to_le_bytes());
            compressed.extend_from_slice(line.as_bytes());
        }
        
        // Write indices
        for index in indices {
            compressed.extend_from_slice(&(index as u32).to_le_bytes());
        }
        
        // Apply final gzip compression
        let final_compressed = self.apply_gzip(&compressed)?;
        
        Ok(DeltaResult {
            compressed_data: final_compressed.clone(),
            original_size: data.len(),
            compressed_size: final_compressed.len(),
            compression_ratio: final_compressed.len() as f64 / data.len() as f64,
            delta_type: DeltaType::TextDelta,
        })
    }

    /// Standard gzip compression
    fn gzip_compress(&self, data: &[u8]) -> Result<DeltaResult> {
        let compressed = self.apply_gzip(data)?;
        
        Ok(DeltaResult {
            compressed_data: compressed.clone(),
            original_size: data.len(),
            compressed_size: compressed.len(),
            compression_ratio: compressed.len() as f64 / data.len() as f64,
            delta_type: DeltaType::Gzip,
        })
    }

    /// Apply gzip compression to data
    fn apply_gzip(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = GzEncoder::new(Vec::new(), self.compression_level);
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    /// Check if data appears to be a text file
    fn is_text_file(&self, data: &[u8]) -> bool {
        // Simple heuristic: check if most bytes are printable ASCII
        let text_bytes = data.iter()
            .take(1024) // Sample first 1KB
            .filter(|&&b| b.is_ascii() && (b.is_ascii_graphic() || b.is_ascii_whitespace()))
            .count();
        
        text_bytes as f64 / data.len().min(1024) as f64 > 0.8
    }

    /// Streaming data transfer with chunking and bandwidth limiting
    pub async fn stream_transfer<R: AsyncRead + Unpin, W: AsyncWrite + Unpin>(
        &self,
        mut reader: R,
        mut writer: W,
        config: StreamConfig,
    ) -> Result<TransferStats> {
        let start = Instant::now();
        let mut buffer = vec![0u8; config.chunk_size];
        let mut total_bytes = 0u64;
        let mut chunks_processed = 0u64;
        
        // Bandwidth limiting setup
        let bandwidth_delay = if let Some(limit_mbps) = config.bandwidth_limit_mbps {
            let bytes_per_second = (limit_mbps * 1024.0 * 1024.0) as u64;
            let delay_per_chunk = Duration::from_nanos(
                (config.chunk_size as u64 * 1_000_000_000) / bytes_per_second
            );
            Some(delay_per_chunk)
        } else {
            None
        };
        
        loop {
            // Read chunk
            let bytes_read = reader.read(&mut buffer).await?;
            if bytes_read == 0 {
                break; // EOF
            }
            
            // Optionally compress chunk
            let chunk_data = if config.enable_compression {
                self.apply_gzip(&buffer[..bytes_read])?
            } else {
                buffer[..bytes_read].to_vec()
            };
            
            // Write chunk
            writer.write_all(&chunk_data).await?;
            
            total_bytes += bytes_read as u64;
            chunks_processed += 1;
            
            // Apply bandwidth limiting
            if let Some(delay) = bandwidth_delay {
                tokio::time::sleep(delay).await;
            }
            
            // Progress reporting
            if config.enable_progress && chunks_processed % 100 == 0 {
                let elapsed = start.elapsed();
                let speed_mbps = (total_bytes as f64 / (1024.0 * 1024.0)) / elapsed.as_secs_f64();
                println!("📡 Streaming: {} transferred, {:.1} MB/s", 
                         self.format_bytes(total_bytes as usize), speed_mbps);
            }
        }
        
        writer.flush().await?;
        
        let elapsed = start.elapsed();
        let speed_mbps = (total_bytes as f64 / (1024.0 * 1024.0)) / elapsed.as_secs_f64();
        
        println!("✅ Stream complete: {} in {:.2}s ({:.1} MB/s)", 
                 self.format_bytes(total_bytes as usize), 
                 elapsed.as_secs_f64(), 
                 speed_mbps);
        
        Ok(TransferStats {
            total_bytes_original: total_bytes,
            total_bytes_compressed: total_bytes, // Simplified for this example
            compression_ratio: 1.0,
            transfer_speed_mbps: speed_mbps,
            chunks_processed,
            delta_cache_hits: 0,
            bandwidth_savings_mb: 0.0,
        })
    }

    /// Chunked upload simulation
    pub fn chunked_upload(&self, file_path: &Path, chunk_size: usize) -> Result<Vec<Vec<u8>>> {
        let data = std::fs::read(file_path)?;
        let mut chunks = Vec::new();
        
        for chunk in data.chunks(chunk_size) {
            // Compress each chunk individually
            let compressed_chunk = self.apply_gzip(chunk)?;
            chunks.push(compressed_chunk);
        }
        
        let total_original = data.len();
        let total_compressed: usize = chunks.iter().map(|c| c.len()).sum();
        let compression_ratio = total_compressed as f64 / total_original as f64;
        
        println!("📦 Chunked upload: {} → {} chunks ({:.1}% compression)", 
                 self.format_bytes(total_original),
                 chunks.len(),
                 (1.0 - compression_ratio) * 100.0);
        
        Ok(chunks)
    }

    /// Get current transfer statistics
    pub fn get_stats(&self) -> TransferStats {
        self.stats.lock().unwrap().clone()
    }

    /// Format bytes in human-readable format
    fn format_bytes(&self, bytes: usize) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        format!("{:.1}{}", size, UNITS[unit_index])
    }

    /// Print comprehensive network and storage performance summary
    pub fn print_performance_summary(&self) {
        let stats = self.get_stats();
        
        println!("\n🌐 Network & Storage Performance Summary");
        println!("  🗜️  Total compressed: {} → {} ({:.1}% reduction)", 
                 self.format_bytes(stats.total_bytes_original as usize),
                 self.format_bytes(stats.total_bytes_compressed as usize),
                 (1.0 - stats.compression_ratio) * 100.0);
        println!("  📡 Transfer speed: {:.1} MB/s", stats.transfer_speed_mbps);
        println!("  📦 Chunks processed: {}", stats.chunks_processed);
        println!("  💾 Cache hits: {}", stats.delta_cache_hits);
        
        if let Some(limit) = self.bandwidth_limit {
            println!("  🚦 Bandwidth limit: {:.1} MB/s", limit as f64 / (1024.0 * 1024.0));
        }
    }

    /// Clear all caches and reset statistics
    pub fn clear_caches(&self) -> Result<()> {
        {
            let mut delta_cache = self.delta_cache.lock().unwrap();
            delta_cache.clear();
        }
        
        {
            let mut stats = self.stats.lock().unwrap();
            *stats = TransferStats::default();
        }
        
        println!("🧹 Network storage caches cleared");
        Ok(())
    }
}

impl Default for NetworkStorageEngine {
    fn default() -> Self {
        Self::new()
    }
}
