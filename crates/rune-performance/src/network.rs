use anyhow::Result;
use std::collections::HashMap;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::net::{TcpSocket, UdpSocket};
use tokio::time::timeout;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub connection_pool_size: usize,
    pub max_concurrent_connections: usize,
    pub connection_timeout_ms: u64,
    pub read_timeout_ms: u64,
    pub write_timeout_ms: u64,
    pub keepalive_enabled: bool,
    pub keepalive_interval_ms: u64,
    pub tcp_nodelay: bool,
    pub buffer_size: usize,
    pub enable_compression: bool,
    pub compression_threshold: usize,
    pub retry_attempts: usize,
    pub retry_delay_ms: u64,
    pub bandwidth_limit_mbps: Option<f64>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            connection_pool_size: 100,
            max_concurrent_connections: 1000,
            connection_timeout_ms: 5000,
            read_timeout_ms: 30000,
            write_timeout_ms: 30000,
            keepalive_enabled: true,
            keepalive_interval_ms: 60000,
            tcp_nodelay: true,
            buffer_size: 64 * 1024, // 64KB
            enable_compression: true,
            compression_threshold: 1024, // 1KB
            retry_attempts: 3,
            retry_delay_ms: 1000,
            bandwidth_limit_mbps: None,
        }
    }
}

pub struct NetworkOptimizer {
    config: NetworkConfig,
    connection_pool: Arc<Mutex<HashMap<SocketAddr, Vec<TcpStream>>>>,
    active_connections: Arc<RwLock<HashMap<SocketAddr, usize>>>,
    metrics: Arc<RwLock<NetworkMetrics>>,
    bandwidth_limiter: Arc<Mutex<BandwidthLimiter>>,
}

#[derive(Debug, Clone, Default)]
pub struct NetworkMetrics {
    pub total_connections: u64,
    pub active_connections: usize,
    pub connection_pool_hits: u64,
    pub connection_pool_misses: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub compression_ratio: f64,
    pub retry_count: u64,
    pub timeouts: u64,
    pub errors: u64,
    pub average_latency_ms: f64,
    pub peak_bandwidth_mbps: f64,
}

impl NetworkMetrics {
    pub fn connection_pool_hit_ratio(&self) -> f64 {
        let total = self.connection_pool_hits + self.connection_pool_misses;
        if total == 0 {
            0.0
        } else {
            self.connection_pool_hits as f64 / total as f64
        }
    }

    pub fn effective_bandwidth_mbps(&self) -> f64 {
        let total_bytes = self.bytes_sent + self.bytes_received;
        // This is a simplified calculation - in practice you'd track time windows
        total_bytes as f64 * 8.0 / (1024.0 * 1024.0)
    }
}

struct BandwidthLimiter {
    limit_mbps: Option<f64>,
    last_reset: Instant,
    bytes_this_second: u64,
}

impl BandwidthLimiter {
    fn new(limit_mbps: Option<f64>) -> Self {
        Self {
            limit_mbps,
            last_reset: Instant::now(),
            bytes_this_second: 0,
        }
    }

    fn check_and_limit(&mut self, bytes: usize) -> Option<Duration> {
        if let Some(limit) = self.limit_mbps {
            let now = Instant::now();
            
            // Reset counter every second
            if now.duration_since(self.last_reset) >= Duration::from_secs(1) {
                self.bytes_this_second = 0;
                self.last_reset = now;
            }

            let limit_bytes_per_second = (limit * 1024.0 * 1024.0 / 8.0) as u64;
            self.bytes_this_second += bytes as u64;

            if self.bytes_this_second > limit_bytes_per_second {
                // Calculate delay needed
                let excess_bytes = self.bytes_this_second - limit_bytes_per_second;
                let delay_ms = (excess_bytes as f64 / (limit_bytes_per_second as f64)) * 1000.0;
                return Some(Duration::from_millis(delay_ms as u64));
            }
        }
        None
    }
}

pub struct OptimizedConnection {
    stream: TcpStream,
    addr: SocketAddr,
    config: NetworkConfig,
    metrics: Arc<RwLock<NetworkMetrics>>,
    compression_buffer: Vec<u8>,
    last_activity: Instant,
}

impl OptimizedConnection {
    pub async fn connect(addr: SocketAddr, config: NetworkConfig) -> Result<Self> {
        let socket = TcpSocket::new_v4()?;
        
        // Configure socket options
        socket.set_nodelay(config.tcp_nodelay)?;
        if config.keepalive_enabled {
            socket.set_keepalive(true)?;
        }

        // Connect with timeout
        let connect_timeout = Duration::from_millis(config.connection_timeout_ms);
        let stream = timeout(connect_timeout, socket.connect(addr)).await??;
        
        // Convert to std TcpStream for synchronous operations
        let std_stream = stream.into_std()?;

        Ok(Self {
            stream: std_stream,
            addr,
            config: config.clone(),
            metrics: Arc::new(RwLock::new(NetworkMetrics::default())),
            compression_buffer: Vec::with_capacity(config.buffer_size),
            last_activity: Instant::now(),
        })
    }

    pub fn send_data(&mut self, data: &[u8]) -> Result<usize> {
        let data_to_send = if self.config.enable_compression && data.len() > self.config.compression_threshold {
            self.compress_data(data)?
        } else {
            data.to_vec()
        };

        let bytes_sent = self.send_with_retry(&data_to_send)?;
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.bytes_sent += bytes_sent as u64;
            if data.len() != data_to_send.len() {
                metrics.compression_ratio = data_to_send.len() as f64 / data.len() as f64;
            }
        }

        self.last_activity = Instant::now();
        Ok(bytes_sent)
    }

    pub fn receive_data(&mut self, buffer: &mut [u8]) -> Result<usize> {
        use std::io::Read;
        
        let bytes_received = self.receive_with_retry(buffer)?;
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.bytes_received += bytes_received as u64;
        }

        self.last_activity = Instant::now();
        Ok(bytes_received)
    }

    pub fn is_alive(&self) -> bool {
        // Check if connection is still active
        self.last_activity.elapsed() < Duration::from_millis(self.config.keepalive_interval_ms * 2)
    }

    fn compress_data(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        use std::io::Write;
        
        self.compression_buffer.clear();
        let mut encoder = flate2::write::GzEncoder::new(&mut self.compression_buffer, flate2::Compression::fast());
        encoder.write_all(data)?;
        encoder.finish()?;
        
        Ok(self.compression_buffer.clone())
    }

    fn send_with_retry(&mut self, data: &[u8]) -> Result<usize> {
        use std::io::Write;
        
        let mut attempts = 0;
        let max_attempts = self.config.retry_attempts;
        let retry_delay = Duration::from_millis(self.config.retry_delay_ms);

        while attempts < max_attempts {
            match self.stream.write(data) {
                Ok(bytes_sent) => {
                    if attempts > 0 {
                        self.metrics.write().unwrap().retry_count += 1;
                    }
                    return Ok(bytes_sent);
                }
                Err(e) if attempts < max_attempts - 1 => {
                    attempts += 1;
                    std::thread::sleep(retry_delay);
                    continue;
                }
                Err(e) => {
                    self.metrics.write().unwrap().errors += 1;
                    return Err(e.into());
                }
            }
        }

        unreachable!()
    }

    fn receive_with_retry(&mut self, buffer: &mut [u8]) -> Result<usize> {
        use std::io::Read;
        
        let mut attempts = 0;
        let max_attempts = self.config.retry_attempts;
        let retry_delay = Duration::from_millis(self.config.retry_delay_ms);

        while attempts < max_attempts {
            match self.stream.read(buffer) {
                Ok(bytes_received) => {
                    if attempts > 0 {
                        self.metrics.write().unwrap().retry_count += 1;
                    }
                    return Ok(bytes_received);
                }
                Err(e) if attempts < max_attempts - 1 => {
                    attempts += 1;
                    std::thread::sleep(retry_delay);
                    continue;
                }
                Err(e) => {
                    self.metrics.write().unwrap().errors += 1;
                    return Err(e.into());
                }
            }
        }

        unreachable!()
    }
}

impl NetworkOptimizer {
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            bandwidth_limiter: Arc::new(Mutex::new(BandwidthLimiter::new(config.bandwidth_limit_mbps))),
            config,
            connection_pool: Arc::new(Mutex::new(HashMap::new())),
            active_connections: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(NetworkMetrics::default())),
        }
    }

    pub async fn get_connection(&self, addr: SocketAddr) -> Result<OptimizedConnection> {
        // Check if we can reuse a pooled connection
        if let Some(stream) = self.get_pooled_connection(&addr) {
            self.metrics.write().unwrap().connection_pool_hits += 1;
            return Ok(OptimizedConnection {
                stream,
                addr,
                config: self.config.clone(),
                metrics: self.metrics.clone(),
                compression_buffer: Vec::with_capacity(self.config.buffer_size),
                last_activity: Instant::now(),
            });
        }

        self.metrics.write().unwrap().connection_pool_misses += 1;

        // Check connection limits
        let active_count = self.active_connections.read().unwrap()
            .values().sum::<usize>();
        
        if active_count >= self.config.max_concurrent_connections {
            return Err(anyhow::anyhow!("Maximum concurrent connections reached"));
        }

        // Create new connection
        let connection = OptimizedConnection::connect(addr, self.config.clone()).await?;
        
        // Update active connections count
        *self.active_connections.write().unwrap()
            .entry(addr).or_insert(0) += 1;
        
        self.metrics.write().unwrap().total_connections += 1;
        
        Ok(connection)
    }

    pub fn return_connection(&self, connection: OptimizedConnection) {
        if connection.is_alive() && 
           self.get_pool_size(&connection.addr) < self.config.connection_pool_size {
            // Return connection to pool
            self.connection_pool.lock()
                .entry(connection.addr)
                .or_default()
                .push(connection.stream);
        }

        // Decrease active count
        if let Some(count) = self.active_connections.write().unwrap().get_mut(&connection.addr) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }

    pub async fn send_with_optimization(
        &self,
        addr: SocketAddr,
        data: &[u8],
    ) -> Result<usize> {
        // Apply bandwidth limiting
        {
            let mut limiter_guard = self.bandwidth_limiter.lock();
            if let Some(delay) = limiter_guard.check_and_limit(data.len()) {
                drop(limiter_guard); // Release lock before await
                tokio::time::sleep(delay).await;
            }
        }

        let mut connection = self.get_connection(addr).await?;
        let result = connection.send_data(data);
        self.return_connection(connection);
        
        result
    }

    pub async fn receive_with_optimization(
        &self,
        addr: SocketAddr,
        buffer: &mut [u8],
    ) -> Result<usize> {
        let mut connection = self.get_connection(addr).await?;
        let result = connection.receive_data(buffer);
        self.return_connection(connection);
        
        result
    }

    pub fn cleanup_stale_connections(&self) {
        let mut pool = self.connection_pool.lock();
        
        // Remove stale connections from pool
        pool.retain(|_, connections| {
            connections.retain(|stream| {
                // Simple check - in practice you'd need more sophisticated detection
                stream.peer_addr().is_ok()
            });
            !connections.is_empty()
        });
    }

    pub fn get_metrics(&self) -> NetworkMetrics {
        let mut metrics = self.metrics.read().unwrap().clone();
        metrics.active_connections = self.active_connections.read().unwrap()
            .values().sum::<usize>();
        metrics
    }

    pub fn set_bandwidth_limit(&self, limit_mbps: Option<f64>) {
        self.bandwidth_limiter.lock().limit_mbps = limit_mbps;
    }

    fn get_pooled_connection(&self, addr: &SocketAddr) -> Option<TcpStream> {
        self.connection_pool.lock()
            .get_mut(addr)?
            .pop()
    }

    fn get_pool_size(&self, addr: &SocketAddr) -> usize {
        self.connection_pool.lock()
            .get(addr)
            .map(|pool| pool.len())
            .unwrap_or(0)
    }
}

// UDP optimization for faster, connectionless operations
pub struct UdpOptimizer {
    config: NetworkConfig,
    socket: UdpSocket,
    metrics: Arc<RwLock<NetworkMetrics>>,
    bandwidth_limiter: Arc<Mutex<BandwidthLimiter>>,
}

impl UdpOptimizer {
    pub async fn new(bind_addr: SocketAddr, config: NetworkConfig) -> Result<Self> {
        let socket = UdpSocket::bind(bind_addr).await?;
        
        Ok(Self {
            socket,
            config: config.clone(),
            metrics: Arc::new(RwLock::new(NetworkMetrics::default())),
            bandwidth_limiter: Arc::new(Mutex::new(BandwidthLimiter::new(config.bandwidth_limit_mbps))),
        })
    }

    pub async fn send_to(&self, data: &[u8], addr: SocketAddr) -> Result<usize> {
        // Apply bandwidth limiting
        if let Some(delay) = self.bandwidth_limiter.lock().check_and_limit(data.len()) {
            tokio::time::sleep(delay).await;
        }

        let bytes_sent = self.socket.send_to(data, addr).await?;
        
        self.metrics.write().unwrap().bytes_sent += bytes_sent as u64;
        Ok(bytes_sent)
    }

    pub async fn receive_from(&self, buffer: &mut [u8]) -> Result<(usize, SocketAddr)> {
        let (bytes_received, addr) = self.socket.recv_from(buffer).await?;
        
        self.metrics.write().unwrap().bytes_received += bytes_received as u64;
        Ok((bytes_received, addr))
    }

    pub fn get_metrics(&self) -> NetworkMetrics {
        self.metrics.read().unwrap().clone()
    }
}

// High-level network operations
pub struct NetworkOperations {
    tcp_optimizer: NetworkOptimizer,
    config: NetworkConfig,
}

impl NetworkOperations {
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            tcp_optimizer: NetworkOptimizer::new(config.clone()),
            config,
        }
    }

    pub async fn bulk_transfer(
        &self,
        addr: SocketAddr,
        data: &[u8],
        chunk_size: usize,
    ) -> Result<()> {
        let chunks = data.chunks(chunk_size);
        let total_chunks = chunks.len();
        
        for (i, chunk) in chunks.enumerate() {
            self.tcp_optimizer.send_with_optimization(addr, chunk).await?;
            
            // Progress reporting could go here
            if i % 100 == 0 {
                println!("Transferred {}/{} chunks", i + 1, total_chunks);
            }
        }

        Ok(())
    }

    pub async fn parallel_transfer(
        self: Arc<Self>,
        targets: Vec<(SocketAddr, Vec<u8>)>,
        max_concurrent: usize,
    ) -> Result<Vec<Result<usize>>> {
        use tokio::task;
        
        let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent));
        let mut handles = Vec::new();

        for (addr, data) in targets {
            let optimizer = self.clone();
            let permit = semaphore.clone().acquire_owned().await?;
            
            let handle = task::spawn(async move {
                let _permit = permit; // Keep permit until task completes
                optimizer.tcp_optimizer.send_with_optimization(addr, &data).await
            });
            
            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await?);
        }

        Ok(results)
    }

    pub fn get_combined_metrics(&self) -> NetworkMetrics {
        self.tcp_optimizer.get_metrics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_network_optimizer_basic() {
        let config = NetworkConfig::default();
        let optimizer = NetworkOptimizer::new(config);

        // Test connection limiting
        let metrics = optimizer.get_metrics();
        assert_eq!(metrics.total_connections, 0);
    }

    #[tokio::test]
    async fn test_bandwidth_limiter() {
        let mut limiter = BandwidthLimiter::new(Some(1.0)); // 1 Mbps limit
        
        // Small transfer should not be limited
        assert!(limiter.check_and_limit(1024).is_none());
        
        // Large transfer should be limited
        assert!(limiter.check_and_limit(1024 * 1024).is_some());
    }

    #[tokio::test]
    async fn test_udp_optimizer() -> Result<()> {
        let config = NetworkConfig::default();
        let bind_addr = "127.0.0.1:0".parse().unwrap();
        
        let udp_optimizer = UdpOptimizer::new(bind_addr, config).await?;
        let metrics = udp_optimizer.get_metrics();
        assert_eq!(metrics.bytes_sent, 0);
        assert_eq!(metrics.bytes_received, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_network_operations() {
        let config = NetworkConfig::default();
        let net_ops = NetworkOperations::new(config);
        
        let metrics = net_ops.get_combined_metrics();
        assert_eq!(metrics.total_connections, 0);
    }
}
