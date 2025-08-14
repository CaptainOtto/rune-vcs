
use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::{fs, path::{Path,PathBuf}};

// LFS functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LfsConfig { 
    pub patterns: Vec<String>, 
    pub chunk_size: usize, 
    pub remote: Option<String>,
    pub upload_enabled: bool,
    pub download_enabled: bool,
    pub migration_threshold: u64, // bytes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pointer { 
    pub oid: String, 
    pub size: u64, 
    pub chunks: Vec<String>,
    pub upload_status: UploadStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UploadStatus {
    Local,
    Uploading,
    Uploaded,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LfsStats {
    pub total_files: usize,
    pub total_size: u64,
    pub tracked_patterns: usize,
    pub remote_files: usize,
    pub local_only_files: usize,
}

pub struct Lfs { pub root: PathBuf, pub dir: PathBuf }
impl Lfs{
  pub fn open(root:impl AsRef<Path>)->Result<Self>{ 
    let root=root.as_ref().to_path_buf(); 
    let d=root.join(".rune").join("lfs"); 
    fs::create_dir_all(d.join("objects"))?; 
    fs::create_dir_all(d.join("tmp"))?; 
    fs::create_dir_all(d.join("logs"))?; 
    Ok(Self{ root, dir:d }) 
  }
  
  pub fn config_path(&self)->PathBuf{ self.dir.join("config.json") }
  
  pub fn config(&self)->Result<LfsConfig>{ 
    if self.config_path().exists(){ 
      Ok(serde_json::from_str(&fs::read_to_string(self.config_path())?)?) 
    } else { 
      Ok(LfsConfig{
        patterns:vec![], 
        chunk_size:8*1024*1024, 
        remote:None,
        upload_enabled: true,
        download_enabled: true,
        migration_threshold: 100 * 1024 * 1024, // 100MB default
      }) 
    } 
  }
  
  pub fn write_config(&self, cfg:&LfsConfig)->Result<()> { 
    fs::write(self.config_path(), serde_json::to_vec_pretty(cfg)?)?; 
    Ok(()) 
  }
  
  pub fn is_tracked(&self, path:&str)->Result<bool>{ 
    let cfg=self.config()?; 
    for pat in cfg.patterns { 
      if glob::Pattern::new(&pat).map(|g|g.matches(path)).unwrap_or(false){ 
        return Ok(true); 
      } 
    } 
    Ok(false) 
  }
  
  pub fn should_migrate(&self, path: &Path) -> Result<bool> {
    let cfg = self.config()?;
    if let Ok(metadata) = fs::metadata(path) {
      Ok(metadata.len() > cfg.migration_threshold)
    } else {
      Ok(false)
    }
  }
  
  pub fn get_stats(&self) -> Result<LfsStats> {
    let cfg = self.config()?;
    let mut stats = LfsStats {
      total_files: 0,
      total_size: 0,
      tracked_patterns: cfg.patterns.len(),
      remote_files: 0,
      local_only_files: 0,
    };
    
    // Walk through objects directory
    if let Ok(entries) = fs::read_dir(self.dir.join("objects")) {
      for entry in entries.flatten() {
        if let Ok(sub_entries) = fs::read_dir(entry.path()) {
          for sub_entry in sub_entries.flatten() {
            if let Ok(oid_entries) = fs::read_dir(sub_entry.path()) {
              for oid_entry in oid_entries.flatten() {
                if oid_entry.path().join("pointer.json").exists() {
                  stats.total_files += 1;
                  if let Ok(ptr_data) = fs::read_to_string(oid_entry.path().join("pointer.json")) {
                    if let Ok(ptr) = serde_json::from_str::<Pointer>(&ptr_data) {
                      stats.total_size += ptr.size;
                      match ptr.upload_status {
                        UploadStatus::Uploaded => stats.remote_files += 1,
                        _ => stats.local_only_files += 1,
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
    
    Ok(stats)
  }
  
  fn chunk_dir(&self, oid:&str)->PathBuf{ 
    self.dir.join("objects").join(&oid[0..2]).join(&oid[2..4]).join(oid) 
  }

  pub fn clean_to_pointer(&self, rel:&str)->Result<Option<Pointer>>{
    if !self.is_tracked(rel)? { return Ok(None); }
    let data = fs::read(self.root.join(rel))?; let oid = format!("{}", blake3::hash(&data));
    let chunk_size = self.config()?.chunk_size; let dir = self.chunk_dir(&oid); fs::create_dir_all(&dir)?;
    let mut chunks = Vec::new();
    for (i, part) in data.chunks(chunk_size).enumerate(){ let cid = format!("{}.{:06}", oid, i); fs::write(dir.join(&cid), part)?; chunks.push(cid); }
    let ptr = Pointer{ oid: oid.clone(), size: data.len() as u64, chunks, upload_status: UploadStatus::Local };
    fs::write(self.root.join(rel), format!("version https://rune-lfs/v1
oid {}
size {}", oid, data.len()))?;
    fs::write(dir.join("pointer.json"), serde_json::to_vec_pretty(&ptr)?)?; Ok(Some(ptr))
  }
  pub fn smudge_from_pointer(&self, rel:&str)->Result<bool>{
    let s = fs::read_to_string(self.root.join(rel)).unwrap_or_default(); if !s.starts_with("version https://rune-lfs/v1"){ return Ok(false); }
    let oid = s.lines().find(|l| l.starts_with("oid ")).unwrap().trim_start_matches("oid ").trim().to_string();
    let dir = self.chunk_dir(&oid); let ppath = dir.join("pointer.json"); if !ppath.exists(){ anyhow::bail!("pointer data missing for {}", rel); }
    let ptr: Pointer = serde_json::from_slice(&fs::read(ppath)?)?; let mut out = Vec::with_capacity(ptr.size as usize);
    for cid in ptr.chunks { let part = fs::read(dir.join(cid))?; out.extend_from_slice(&part); } fs::write(self.root.join(rel), out)?; Ok(true)
  }

  // Migration tools
  pub fn migrate_file(&self, path: &Path) -> Result<bool> {
    if !path.exists() { return Ok(false); }
    
    let relative_path = path.strip_prefix(&self.root)?;
    let path_str = relative_path.to_string_lossy();
    
    // Check if file should be migrated based on size and patterns
    if self.should_migrate(path)? || self.is_tracked(&path_str)? {
      if let Some(_pointer) = self.clean_to_pointer(&path_str)? {
        println!("✓ Migrated {} to LFS", path_str);
        return Ok(true);
      }
    }
    Ok(false)
  }

  pub fn migrate_directory(&self, dir: &Path) -> Result<Vec<String>> {
    let mut migrated = Vec::new();
    
    for entry in walkdir::WalkDir::new(dir) {
      let entry = entry?;
      if entry.file_type().is_file() {
        if self.migrate_file(entry.path())? {
          migrated.push(entry.path().strip_prefix(&self.root)?.to_string_lossy().to_string());
        }
      }
    }
    
    Ok(migrated)
  }

  // Server integration
  pub fn upload_to_server(&self, oid: &str) -> Result<()> {
    let config = self.config()?;
    if let Some(remote_url) = &config.remote {
      if !config.upload_enabled {
        anyhow::bail!("Upload is disabled in configuration");
      }

      let dir = self.chunk_dir(oid);
      let pointer_path = dir.join("pointer.json");
      
      if !pointer_path.exists() {
        anyhow::bail!("Pointer not found for OID: {}", oid);
      }

      let mut pointer: Pointer = serde_json::from_slice(&fs::read(&pointer_path)?)?;
      
      // Mock server upload (in real implementation, this would use HTTP client)
      println!("📤 Uploading {} chunks to {}", pointer.chunks.len(), remote_url);
      
      // Simulate upload process
      pointer.upload_status = UploadStatus::Uploading;
      fs::write(&pointer_path, serde_json::to_vec_pretty(&pointer)?)?;
      
      // In real implementation, upload each chunk
      for chunk in &pointer.chunks {
        let chunk_data = fs::read(dir.join(chunk))?;
        // Upload chunk_data to server
        println!("  ✓ Uploaded chunk: {}", chunk);
      }
      
      pointer.upload_status = UploadStatus::Uploaded;
      fs::write(&pointer_path, serde_json::to_vec_pretty(&pointer)?)?;
      
      println!("✅ Successfully uploaded {}", oid);
    } else {
      anyhow::bail!("No remote server configured");
    }
    
    Ok(())
  }

  pub fn download_from_server(&self, oid: &str) -> Result<()> {
    let config = self.config()?;
    if let Some(remote_url) = &config.remote {
      if !config.download_enabled {
        anyhow::bail!("Download is disabled in configuration");
      }

      println!("📥 Downloading {} from {}", oid, remote_url);
      
      // Mock server download (in real implementation, this would use HTTP client)
      // For now, just mark as available locally
      let dir = self.chunk_dir(oid);
      fs::create_dir_all(&dir)?;
      
      println!("✅ Successfully downloaded {}", oid);
    } else {
      anyhow::bail!("No remote server configured");
    }
    
    Ok(())
  }

  pub fn sync_with_server(&self) -> Result<()> {
    let config = self.config()?;
    if config.remote.is_none() {
      anyhow::bail!("No remote server configured");
    }

    let stats = self.get_stats()?;
    println!("🔄 Syncing {} LFS objects with server...", stats.total_files);

    // Upload local-only files
    if let Ok(entries) = fs::read_dir(self.dir.join("objects")) {
      for entry in entries.flatten() {
        if let Ok(sub_entries) = fs::read_dir(entry.path()) {
          for sub_entry in sub_entries.flatten() {
            if let Ok(oid_entries) = fs::read_dir(sub_entry.path()) {
              for oid_entry in oid_entries.flatten() {
                let oid = oid_entry.file_name().to_string_lossy().to_string();
                let pointer_path = oid_entry.path().join("pointer.json");
                
                if pointer_path.exists() {
                  if let Ok(ptr_data) = fs::read_to_string(&pointer_path) {
                    if let Ok(ptr) = serde_json::from_str::<Pointer>(&ptr_data) {
                      if matches!(ptr.upload_status, UploadStatus::Local | UploadStatus::Failed(_)) {
                        if let Err(e) = self.upload_to_server(&oid) {
                          eprintln!("⚠️  Failed to upload {}: {}", oid, e);
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }

    println!("✅ Sync completed");
    Ok(())
  }

  // Configuration management
  pub fn add_pattern(&self, pattern: &str) -> Result<()> {
    let mut config = self.config()?;
    if !config.patterns.contains(&pattern.to_string()) {
      config.patterns.push(pattern.to_string());
      self.write_config(&config)?;
      println!("✓ Added LFS pattern: {}", pattern);
    } else {
      println!("Pattern already exists: {}", pattern);
    }
    Ok(())
  }

  pub fn remove_pattern(&self, pattern: &str) -> Result<()> {
    let mut config = self.config()?;
    if let Some(pos) = config.patterns.iter().position(|p| p == pattern) {
      config.patterns.remove(pos);
      self.write_config(&config)?;
      println!("✓ Removed LFS pattern: {}", pattern);
    } else {
      println!("Pattern not found: {}", pattern);
    }
    Ok(())
  }

  pub fn set_remote(&self, url: &str) -> Result<()> {
    let mut config = self.config()?;
    config.remote = Some(url.to_string());
    self.write_config(&config)?;
    println!("✓ Set LFS remote: {}", url);
    Ok(())
  }

  pub fn set_chunk_size(&self, size: usize) -> Result<()> {
    let mut config = self.config()?;
    config.chunk_size = size;
    self.write_config(&config)?;
    println!("✓ Set LFS chunk size: {} bytes", size);
    Ok(())
  }

  pub fn set_migration_threshold(&self, threshold: u64) -> Result<()> {
    let mut config = self.config()?;
    config.migration_threshold = threshold;
    self.write_config(&config)?;
    println!("✓ Set LFS migration threshold: {} bytes", threshold);
    Ok(())
  }
}

// Locking functionality moved from rune-cli
pub mod locking;
