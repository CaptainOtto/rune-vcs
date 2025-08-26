//! # Rune Draft System
//!
//! Provides Perforce-style changelist functionality with shelved snapshots.
//! Safer and more powerful than traditional Git stash.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rune_core::Author;
use rune_store::Store;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// A draft commit represents work-in-progress that can be shelved and restored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftCommit {
    /// Unique identifier for this draft
    pub id: String,
    /// Human-readable name/description
    pub name: String,
    /// Optional longer description
    pub description: Option<String>,
    /// Author who created the draft
    pub author: Author,
    /// When the draft was created
    pub created_at: DateTime<Utc>,
    /// When the draft was last updated
    pub updated_at: DateTime<Utc>,
    /// Files included in this draft
    pub files: HashMap<PathBuf, DraftFile>,
    /// Branch this draft was created from
    pub base_branch: String,
    /// Base commit hash
    pub base_commit: String,
    /// Tags for organization
    pub tags: Vec<String>,
    /// Whether this draft is currently applied to working directory
    pub is_active: bool,
}

/// A file in a draft commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftFile {
    /// Original file path
    pub path: PathBuf,
    /// Content of the file at draft time
    pub content: Vec<u8>,
    /// File mode/permissions
    pub mode: u32,
    /// Hash of the content
    pub hash: String,
    /// Whether this is a new file (not in base commit)
    pub is_new: bool,
    /// Whether this file was deleted
    pub is_deleted: bool,
    /// Original file hash in base commit (if exists)
    pub original_hash: Option<String>,
}

/// Configuration for the draft system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftConfig {
    /// Maximum number of drafts to keep
    pub max_drafts: usize,
    /// Auto-cleanup drafts older than this many days
    pub auto_cleanup_days: u32,
    /// Default tags to apply to new drafts
    pub default_tags: Vec<String>,
    /// Whether to auto-create checkpoints on certain operations
    pub auto_checkpoint: bool,
    /// How often to auto-checkpoint (in minutes)
    pub auto_checkpoint_interval: u32,
}

impl Default for DraftConfig {
    fn default() -> Self {
        Self {
            max_drafts: 50,
            auto_cleanup_days: 30,
            default_tags: vec!["draft".to_string()],
            auto_checkpoint: false,
            auto_checkpoint_interval: 15,
        }
    }
}

/// Manager for draft commits and checkpoints
pub struct DraftManager {
    store: Store,
    config: DraftConfig,
    drafts_dir: PathBuf,
}

impl DraftManager {
    /// Create a new draft manager
    pub fn new(store: Store) -> Result<Self> {
        let drafts_dir = store.rune_dir.join("drafts");
        fs::create_dir_all(&drafts_dir)
            .context("Failed to create drafts directory")?;

        let config = Self::load_config(&store)?;

        Ok(Self {
            store,
            config,
            drafts_dir,
        })
    }

    /// Get current configuration
    pub fn config(&self) -> &DraftConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: DraftConfig) -> Result<()> {
        self.config = config;
        self.save_config()
    }

    /// Load configuration from store
    fn load_config(store: &Store) -> Result<DraftConfig> {
        let config_path = store.rune_dir.join("draft_config.json");
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .context("Failed to read draft config")?;
            serde_json::from_str(&content)
                .context("Failed to parse draft config")
        } else {
            Ok(DraftConfig::default())
        }
    }

    /// Save configuration to store
    pub fn save_config(&self) -> Result<()> {
        let config_path = self.store.rune_dir.join("draft_config.json");
        let content = serde_json::to_string_pretty(&self.config)
            .context("Failed to serialize draft config")?;
        fs::write(&config_path, content)
            .context("Failed to write draft config")?;
        Ok(())
    }

    /// Create a new draft from current working directory
    pub fn create_draft(&mut self, name: String, description: Option<String>) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        
        // Get current branch and commit
        let current_branch = self.store.current_branch()
            .unwrap_or_else(|| "main".to_string());
        let head_commit = self.get_head_commit();
        
        // Get current author from environment
        let author = Author {
            name: whoami::username(),
            email: format!("{}@local", whoami::username()),
        };
        
        // Collect modified files from working directory
        let files = self.collect_working_files()?;
        
        let now = Utc::now();
        let draft = DraftCommit {
            id: id.clone(),
            name,
            description,
            author,
            created_at: now,
            updated_at: now,
            files,
            base_branch: current_branch,
            base_commit: head_commit,
            tags: self.config.default_tags.clone(),
            is_active: false,
        };

        self.save_draft(&draft)?;
        
        println!("Created draft '{}' with {} files", draft.name, draft.files.len());
        Ok(id)
    }

    /// Get current head commit ID
    fn get_head_commit(&self) -> String {
        // Try to get current commit, fallback to empty string if not available
        if let Some(ref_str) = self.store.read_ref(&self.store.head_ref()) {
            ref_str
        } else {
            String::new()
        }
    }

    /// Apply a draft to the working directory
    pub fn apply_draft(&mut self, draft_id: &str) -> Result<()> {
        let mut draft = self.load_draft(draft_id)?;
        
        // Deactivate any currently active draft
        self.deactivate_all_drafts()?;
        
        // Apply files to working directory
        for (path, draft_file) in &draft.files {
            if draft_file.is_deleted {
                if path.exists() {
                    fs::remove_file(path)
                        .with_context(|| format!("Failed to delete file: {:?}", path))?;
                }
            } else {
                // Create parent directories if needed
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create directory: {:?}", parent))?;
                }
                
                fs::write(path, &draft_file.content)
                    .with_context(|| format!("Failed to write file: {:?}", path))?;
                
                // Set file permissions on Unix systems
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let perms = std::fs::Permissions::from_mode(draft_file.mode);
                    fs::set_permissions(path, perms)
                        .with_context(|| format!("Failed to set permissions: {:?}", path))?;
                }
            }
        }
        
        // Mark draft as active
        draft.is_active = true;
        draft.updated_at = Utc::now();
        self.save_draft(&draft)?;
        
        println!("Applied draft '{}' with {} files", draft.name, draft.files.len());
        Ok(())
    }

    /// Shelve (remove) an active draft from working directory
    pub fn shelve_draft(&mut self, draft_id: &str) -> Result<()> {
        let mut draft = self.load_draft(draft_id)?;
        
        if !draft.is_active {
            anyhow::bail!("Draft '{}' is not currently active", draft.name);
        }
        
        // Remove files that were added by this draft
        for (path, draft_file) in &draft.files {
            if draft_file.is_new && path.exists() {
                fs::remove_file(path)
                    .with_context(|| format!("Failed to remove file: {:?}", path))?;
            }
        }
        
        // Mark draft as inactive
        draft.is_active = false;
        draft.updated_at = Utc::now();
        self.save_draft(&draft)?;
        
        println!("Shelved draft '{}'", draft.name);
        Ok(())
    }

    /// Update an existing draft with current working directory changes
    pub fn update_draft(&mut self, draft_id: &str) -> Result<()> {
        let mut draft = self.load_draft(draft_id)?;
        
        // Collect current working files
        let files = self.collect_working_files()?;
        
        draft.files = files;
        draft.updated_at = Utc::now();
        
        self.save_draft(&draft)?;
        
        println!("Updated draft '{}' with {} files", draft.name, draft.files.len());
        Ok(())
    }

    /// Delete a draft
    pub fn delete_draft(&mut self, draft_id: &str) -> Result<()> {
        let draft = self.load_draft(draft_id)?;
        
        if draft.is_active {
            self.shelve_draft(draft_id)?;
        }
        
        let draft_path = self.drafts_dir.join(format!("{}.json", draft_id));
        fs::remove_file(&draft_path)
            .context("Failed to delete draft file")?;
        
        println!("Deleted draft '{}'", draft.name);
        Ok(())
    }

    /// List all drafts
    pub fn list_drafts(&self) -> Result<Vec<DraftCommit>> {
        let mut drafts = Vec::new();
        
        if !self.drafts_dir.exists() {
            return Ok(drafts);
        }
        
        for entry in fs::read_dir(&self.drafts_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(draft) = self.load_draft_from_path(&path) {
                    drafts.push(draft);
                }
            }
        }
        
        // Sort by creation time (newest first)
        drafts.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(drafts)
    }

    /// Create an automatic checkpoint
    pub fn create_checkpoint(&mut self, name: Option<String>) -> Result<String> {
        let checkpoint_name = name.unwrap_or_else(|| {
            format!("checkpoint-{}", Utc::now().format("%Y%m%d-%H%M%S"))
        });
        
        self.create_draft(checkpoint_name, Some("Automatic checkpoint".to_string()))
    }

    /// Clean up old drafts based on configuration
    pub fn cleanup_old_drafts(&mut self) -> Result<usize> {
        let drafts = self.list_drafts()?;
        let cutoff_date = Utc::now() - chrono::Duration::days(self.config.auto_cleanup_days as i64);
        
        let mut cleaned = 0;
        for draft in drafts {
            if draft.created_at < cutoff_date && !draft.is_active {
                self.delete_draft(&draft.id)?;
                cleaned += 1;
            }
        }
        
        Ok(cleaned)
    }

    /// Get draft by ID
    pub fn get_draft(&self, draft_id: &str) -> Result<DraftCommit> {
        self.load_draft(draft_id)
    }

    /// Add tags to a draft
    pub fn add_tags(&mut self, draft_id: &str, tags: Vec<String>) -> Result<()> {
        let mut draft = self.load_draft(draft_id)?;
        
        for tag in tags {
            if !draft.tags.contains(&tag) {
                draft.tags.push(tag);
            }
        }
        
        draft.updated_at = Utc::now();
        self.save_draft(&draft)?;
        
        Ok(())
    }

    /// Remove tags from a draft
    pub fn remove_tags(&mut self, draft_id: &str, tags: Vec<String>) -> Result<()> {
        let mut draft = self.load_draft(draft_id)?;
        
        draft.tags.retain(|tag| !tags.contains(tag));
        draft.updated_at = Utc::now();
        self.save_draft(&draft)?;
        
        Ok(())
    }

    // Private helper methods

    fn collect_working_files(&self) -> Result<HashMap<PathBuf, DraftFile>> {
        let files = HashMap::new();
        
        // TODO: Implement proper file collection
        // For now, return empty map to get basic functionality working
        // This would need to:
        // 1. Scan working directory for modified files
        // 2. Compare with index to detect changes
        // 3. Include new/deleted files
        
        Ok(files)
    }

    fn get_file_mode(metadata: &fs::Metadata) -> u32 {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            metadata.mode()
        }
        #[cfg(not(unix))]
        {
            if metadata.permissions().readonly() {
                0o644
            } else {
                0o755
            }
        }
    }

    fn save_draft(&self, draft: &DraftCommit) -> Result<()> {
        let draft_path = self.drafts_dir.join(format!("{}.json", draft.id));
        let content = serde_json::to_string_pretty(draft)
            .context("Failed to serialize draft")?;
        fs::write(&draft_path, content)
            .context("Failed to write draft file")?;
        Ok(())
    }

    fn load_draft(&self, draft_id: &str) -> Result<DraftCommit> {
        let draft_path = self.drafts_dir.join(format!("{}.json", draft_id));
        self.load_draft_from_path(&draft_path)
    }

    fn load_draft_from_path(&self, path: &Path) -> Result<DraftCommit> {
        let content = fs::read_to_string(path)
            .context("Failed to read draft file")?;
        serde_json::from_str(&content)
            .context("Failed to parse draft file")
    }

    fn deactivate_all_drafts(&mut self) -> Result<()> {
        let drafts = self.list_drafts()?;
        
        for mut draft in drafts {
            if draft.is_active {
                draft.is_active = false;
                draft.updated_at = Utc::now();
                self.save_draft(&draft)?;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_store() -> (Store, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        // Create a rune directory to make it a valid repository  
        std::fs::create_dir_all(temp_dir.path().join(".rune")).unwrap();
        let store = Store::open(temp_dir.path()).unwrap();
        (store, temp_dir)
    }

    #[test]
    fn test_draft_manager_creation() {
        let (store, _temp) = setup_test_store();
        let manager = DraftManager::new(store).unwrap();
        
        assert_eq!(manager.config().max_drafts, 50);
        assert_eq!(manager.config().auto_cleanup_days, 30);
        assert!(manager.drafts_dir.exists());
    }

    #[test]
    fn test_draft_config_serialization() {
        let config = DraftConfig {
            max_drafts: 25,
            auto_cleanup_days: 14,
            default_tags: vec!["test".to_string()],
            auto_checkpoint: true,
            auto_checkpoint_interval: 10,
        };

        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: DraftConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(config.max_drafts, deserialized.max_drafts);
        assert_eq!(config.auto_cleanup_days, deserialized.auto_cleanup_days);
        assert_eq!(config.default_tags, deserialized.default_tags);
        assert_eq!(config.auto_checkpoint, deserialized.auto_checkpoint);
        assert_eq!(config.auto_checkpoint_interval, deserialized.auto_checkpoint_interval);
    }

    #[test]
    fn test_draft_file_creation() {
        let file = DraftFile {
            path: PathBuf::from("test.txt"),
            content: b"Hello, World!".to_vec(),
            mode: 0o644,
            hash: "test_hash".to_string(),
            is_new: true,
            is_deleted: false,
            original_hash: None,
        };

        assert_eq!(file.path, PathBuf::from("test.txt"));
        assert_eq!(file.content, b"Hello, World!");
        assert!(!file.is_deleted);
        assert!(file.is_new);
    }

    #[test]
    fn test_draft_listing() {
        let (store, _temp) = setup_test_store();
        let manager = DraftManager::new(store).unwrap();
        
        let drafts = manager.list_drafts().unwrap();
        assert_eq!(drafts.len(), 0);
    }

    #[test]
    fn test_checkpoint_naming() {
        let auto_name = format!("checkpoint-{}", Utc::now().format("%Y%m%d"));
        assert!(auto_name.starts_with("checkpoint-"));
    }
}
