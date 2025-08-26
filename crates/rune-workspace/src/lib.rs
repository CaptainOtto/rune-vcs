use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::{fs, time::SystemTime};

/// Virtual workspace configuration for sparse checkout and monorepo management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub name: String,
    pub root_path: PathBuf,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub virtual_roots: HashMap<String, VirtualRoot>,
    pub performance_limits: PerformanceLimits,
    pub created_at: SystemTime,
    pub last_updated: SystemTime,
}

/// Virtual root definition for monorepo sub-workspaces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualRoot {
    pub name: String,
    pub path: PathBuf,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub dependencies: Vec<String>,
    pub active: bool,
    pub auto_include_deps: bool,
}

/// Performance limits and guardrails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceLimits {
    pub max_file_size_mb: u64,
    pub max_files_per_commit: usize,
    pub max_binary_files_per_commit: usize,
    pub warn_file_size_mb: u64,
    pub blocked_extensions: Vec<String>,
    pub tracked_extensions: Vec<String>,
}

impl Default for PerformanceLimits {
    fn default() -> Self {
        Self {
            max_file_size_mb: 100,
            max_files_per_commit: 1000,
            max_binary_files_per_commit: 50,
            warn_file_size_mb: 10,
            blocked_extensions: vec![
                ".exe".to_string(),
                ".dll".to_string(),
                ".so".to_string(),
                ".dylib".to_string(),
                ".a".to_string(),
                ".lib".to_string(),
                ".obj".to_string(),
                ".o".to_string(),
                ".class".to_string(),
                ".jar".to_string(),
                ".war".to_string(),
                ".ear".to_string(),
            ],
            tracked_extensions: vec![
                ".rs".to_string(),
                ".toml".to_string(),
                ".json".to_string(),
                ".yaml".to_string(),
                ".yml".to_string(),
                ".md".to_string(),
                ".txt".to_string(),
                ".py".to_string(),
                ".js".to_string(),
                ".ts".to_string(),
                ".go".to_string(),
                ".c".to_string(),
                ".cpp".to_string(),
                ".h".to_string(),
                ".hpp".to_string(),
            ],
        }
    }
}

/// Workspace manager for virtual workspaces and sparse checkout
pub struct WorkspaceManager {
    pub config: WorkspaceConfig,
    pub cache_dir: PathBuf,
}

impl WorkspaceManager {
    /// Create a new workspace manager
    pub fn new(root_path: PathBuf, name: String) -> Result<Self> {
        let cache_dir = root_path.join(".rune").join("workspace");
        fs::create_dir_all(&cache_dir)?;

        let config = WorkspaceConfig {
            name,
            root_path: root_path.clone(),
            include_patterns: vec!["*".to_string()],
            exclude_patterns: vec![],
            virtual_roots: HashMap::new(),
            performance_limits: PerformanceLimits::default(),
            created_at: SystemTime::now(),
            last_updated: SystemTime::now(),
        };

        Ok(Self { config, cache_dir })
    }

    /// Load existing workspace configuration
    pub fn load(root_path: PathBuf) -> Result<Self> {
        let cache_dir = root_path.join(".rune").join("workspace");
        let config_path = cache_dir.join("config.json");

        if !config_path.exists() {
            anyhow::bail!("No workspace configuration found at {}", config_path.display());
        }

        let config_data = fs::read_to_string(&config_path)?;
        let config: WorkspaceConfig = serde_json::from_str(&config_data)?;

        Ok(Self { config, cache_dir })
    }

    /// Save workspace configuration
    pub fn save(&mut self) -> Result<()> {
        self.config.last_updated = SystemTime::now();
        let config_path = self.cache_dir.join("config.json");
        let config_data = serde_json::to_string_pretty(&self.config)?;
        fs::write(&config_path, config_data)?;
        Ok(())
    }

    /// Add a virtual root to the workspace
    pub fn add_virtual_root(&mut self, name: String, path: PathBuf, patterns: Vec<String>) -> Result<()> {
        let virtual_root = VirtualRoot {
            name: name.clone(),
            path: path.clone(),
            include_patterns: patterns,
            exclude_patterns: vec![],
            dependencies: vec![],
            active: true,
            auto_include_deps: true,
        };

        self.config.virtual_roots.insert(name.clone(), virtual_root);
        self.save()?;

        println!("✓ Added virtual root '{}' at path: {}", name, path.display());
        Ok(())
    }

    /// Remove a virtual root
    pub fn remove_virtual_root(&mut self, name: &str) -> Result<()> {
        if self.config.virtual_roots.remove(name).is_some() {
            self.save()?;
            println!("✓ Removed virtual root '{}'", name);
        } else {
            anyhow::bail!("Virtual root '{}' not found", name);
        }
        Ok(())
    }

    /// Activate/deactivate a virtual root
    pub fn set_virtual_root_active(&mut self, name: &str, active: bool) -> Result<()> {
        if let Some(root) = self.config.virtual_roots.get_mut(name) {
            root.active = active;
            self.save()?;
            println!("✓ Virtual root '{}' {}", name, if active { "activated" } else { "deactivated" });
        } else {
            anyhow::bail!("Virtual root '{}' not found", name);
        }
        Ok(())
    }

    /// Get all files that should be included in the current workspace view
    pub fn get_workspace_files(&self) -> Result<HashSet<PathBuf>> {
        let mut included_files = HashSet::new();

        // Process active virtual roots
        for (name, root) in &self.config.virtual_roots {
            if !root.active {
                continue;
            }

            println!("📁 Processing virtual root: {}", name);
            let root_files = self.get_virtual_root_files(root)?;
            included_files.extend(root_files);
        }

        // Apply global include/exclude patterns
        let filtered_files = self.apply_global_patterns(included_files)?;

        Ok(filtered_files)
    }

    /// Get files for a specific virtual root
    fn get_virtual_root_files(&self, root: &VirtualRoot) -> Result<HashSet<PathBuf>> {
        let mut files = HashSet::new();
        let full_root_path = self.config.root_path.join(&root.path);

        if !full_root_path.exists() {
            return Ok(files);
        }

        for entry in walkdir::WalkDir::new(&full_root_path) {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }

            let file_path = entry.path();
            let relative_path = file_path.strip_prefix(&self.config.root_path)?;

            // Check include patterns
            let included = if root.include_patterns.is_empty() {
                true
            } else {
                root.include_patterns.iter().any(|pattern| {
                    glob::Pattern::new(pattern)
                        .map(|p| p.matches_path(relative_path))
                        .unwrap_or(false)
                })
            };

            // Check exclude patterns
            let excluded = root.exclude_patterns.iter().any(|pattern| {
                glob::Pattern::new(pattern)
                    .map(|p| p.matches_path(relative_path))
                    .unwrap_or(false)
            });

            if included && !excluded {
                files.insert(relative_path.to_path_buf());
            }
        }

        Ok(files)
    }

    /// Apply global include/exclude patterns
    fn apply_global_patterns(&self, files: HashSet<PathBuf>) -> Result<HashSet<PathBuf>> {
        let mut filtered_files = HashSet::new();

        for file_path in files {
            // Check global include patterns
            let included = self.config.include_patterns.iter().any(|pattern| {
                glob::Pattern::new(pattern)
                    .map(|p| p.matches_path(&file_path))
                    .unwrap_or(false)
            });

            // Check global exclude patterns
            let excluded = self.config.exclude_patterns.iter().any(|pattern| {
                glob::Pattern::new(pattern)
                    .map(|p| p.matches_path(&file_path))
                    .unwrap_or(false)
            });

            if included && !excluded {
                filtered_files.insert(file_path);
            }
        }

        Ok(filtered_files)
    }

    /// Check if a file meets performance guardrails
    pub fn check_performance_limits(&self, file_path: &Path) -> Result<PerformanceCheck> {
        let full_path = self.config.root_path.join(file_path);
        
        if !full_path.exists() {
            return Ok(PerformanceCheck::NotFound);
        }

        let metadata = fs::metadata(&full_path)?;
        let file_size_mb = metadata.len() / (1024 * 1024);

        // Check file size limits
        if file_size_mb > self.config.performance_limits.max_file_size_mb {
            return Ok(PerformanceCheck::TooLarge {
                size_mb: file_size_mb,
                limit_mb: self.config.performance_limits.max_file_size_mb,
            });
        }

        if file_size_mb > self.config.performance_limits.warn_file_size_mb {
            return Ok(PerformanceCheck::Warning {
                size_mb: file_size_mb,
                warn_limit_mb: self.config.performance_limits.warn_file_size_mb,
            });
        }

        // Check file extension
        if let Some(extension) = file_path.extension().and_then(|e| e.to_str()) {
            let ext_with_dot = format!(".{}", extension);
            
            if self.config.performance_limits.blocked_extensions.contains(&ext_with_dot) {
                return Ok(PerformanceCheck::BlockedExtension {
                    extension: ext_with_dot,
                });
            }
        }

        Ok(PerformanceCheck::Ok)
    }

    /// Validate a set of files for commit
    pub fn validate_commit_files(&self, files: &[PathBuf]) -> Result<CommitValidation> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut binary_count = 0;

        // Check file count limit
        if files.len() > self.config.performance_limits.max_files_per_commit {
            errors.push(format!(
                "Too many files in commit: {} (limit: {})",
                files.len(),
                self.config.performance_limits.max_files_per_commit
            ));
        }

        // Check each file
        for file_path in files {
            match self.check_performance_limits(file_path)? {
                PerformanceCheck::TooLarge { size_mb, limit_mb } => {
                    errors.push(format!(
                        "File too large: {} ({} MB, limit: {} MB)",
                        file_path.display(),
                        size_mb,
                        limit_mb
                    ));
                }
                PerformanceCheck::Warning { size_mb, warn_limit_mb } => {
                    warnings.push(format!(
                        "Large file warning: {} ({} MB, warning threshold: {} MB)",
                        file_path.display(),
                        size_mb,
                        warn_limit_mb
                    ));
                }
                PerformanceCheck::BlockedExtension { extension } => {
                    errors.push(format!(
                        "Blocked file type: {} (extension: {})",
                        file_path.display(),
                        extension
                    ));
                }
                PerformanceCheck::NotFound => {
                    warnings.push(format!("File not found: {}", file_path.display()));
                }
                PerformanceCheck::Ok => {}
            }

            // Count binary files
            if self.is_likely_binary(file_path)? {
                binary_count += 1;
            }
        }

        // Check binary file limit
        if binary_count > self.config.performance_limits.max_binary_files_per_commit {
            errors.push(format!(
                "Too many binary files in commit: {} (limit: {})",
                binary_count,
                self.config.performance_limits.max_binary_files_per_commit
            ));
        }

        Ok(CommitValidation {
            valid: errors.is_empty(),
            warnings,
            errors,
            file_count: files.len(),
            binary_count,
        })
    }

    /// Check if a file is likely binary
    fn is_likely_binary(&self, file_path: &Path) -> Result<bool> {
        if let Some(extension) = file_path.extension().and_then(|e| e.to_str()) {
            let ext_with_dot = format!(".{}", extension);
            
            // Check if it's a known text extension
            if self.config.performance_limits.tracked_extensions.contains(&ext_with_dot) {
                return Ok(false);
            }
            
            // Check if it's a known binary extension
            if self.config.performance_limits.blocked_extensions.contains(&ext_with_dot) {
                return Ok(true);
            }
        }

        // For unknown extensions, check file content
        let full_path = self.config.root_path.join(file_path);
        if full_path.exists() {
            let sample = fs::read(&full_path)?;
            let sample_size = std::cmp::min(sample.len(), 8192);
            
            // Check for null bytes (common binary indicator)
            let null_count = sample[..sample_size].iter().filter(|&&b| b == 0).count();
            let null_ratio = null_count as f64 / sample_size as f64;
            
            Ok(null_ratio > 0.01) // More than 1% null bytes suggests binary
        } else {
            Ok(false)
        }
    }

    /// List all virtual roots
    pub fn list_virtual_roots(&self) -> Vec<(&String, &VirtualRoot)> {
        self.config.virtual_roots.iter().collect()
    }

    /// Add global include pattern
    pub fn add_include_pattern(&mut self, pattern: String) -> Result<()> {
        if !self.config.include_patterns.contains(&pattern) {
            self.config.include_patterns.push(pattern.clone());
            self.save()?;
            println!("✓ Added include pattern: {}", pattern);
        }
        Ok(())
    }

    /// Add global exclude pattern
    pub fn add_exclude_pattern(&mut self, pattern: String) -> Result<()> {
        if !self.config.exclude_patterns.contains(&pattern) {
            self.config.exclude_patterns.push(pattern.clone());
            self.save()?;
            println!("✓ Added exclude pattern: {}", pattern);
        }
        Ok(())
    }

    /// Update performance limits
    pub fn update_performance_limits(&mut self, limits: PerformanceLimits) -> Result<()> {
        self.config.performance_limits = limits;
        self.save()?;
        println!("✓ Updated performance limits");
        Ok(())
    }
}

/// Result of performance check
#[derive(Debug, Clone)]
pub enum PerformanceCheck {
    Ok,
    Warning { size_mb: u64, warn_limit_mb: u64 },
    TooLarge { size_mb: u64, limit_mb: u64 },
    BlockedExtension { extension: String },
    NotFound,
}

/// Result of commit validation
#[derive(Debug, Clone)]
pub struct CommitValidation {
    pub valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub file_count: usize,
    pub binary_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_workspace_creation() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().to_path_buf();
        
        let workspace = WorkspaceManager::new(root_path.clone(), "test-workspace".to_string()).unwrap();
        
        assert_eq!(workspace.config.name, "test-workspace");
        assert_eq!(workspace.config.root_path, root_path);
        assert!(workspace.config.virtual_roots.is_empty());
    }

    #[test]
    fn test_virtual_root_management() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().to_path_buf();
        
        let mut workspace = WorkspaceManager::new(root_path, "test-workspace".to_string()).unwrap();
        
        // Add virtual root
        workspace.add_virtual_root(
            "frontend".to_string(),
            PathBuf::from("packages/frontend"),
            vec!["*.js".to_string(), "*.ts".to_string()],
        ).unwrap();
        
        assert_eq!(workspace.config.virtual_roots.len(), 1);
        assert!(workspace.config.virtual_roots.contains_key("frontend"));
        
        // Remove virtual root
        workspace.remove_virtual_root("frontend").unwrap();
        assert!(workspace.config.virtual_roots.is_empty());
    }

    #[test]
    fn test_performance_limits() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().to_path_buf();
        
        let workspace = WorkspaceManager::new(root_path.clone(), "test-workspace".to_string()).unwrap();
        
        // Create a test exe file
        let exe_file = root_path.join("test.exe");
        fs::write(&exe_file, "fake exe content").unwrap();
        
        // Test blocked extension
        let result = workspace.check_performance_limits(Path::new("test.exe")).unwrap();
        match result {
            PerformanceCheck::BlockedExtension { extension } => {
                assert_eq!(extension, ".exe");
            }
            _ => panic!("Expected blocked extension, got: {:?}", result),
        }
    }

    #[test]
    fn test_commit_validation() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().to_path_buf();
        
        let workspace = WorkspaceManager::new(root_path, "test-workspace".to_string()).unwrap();
        
        let files = vec![
            PathBuf::from("src/main.rs"),
            PathBuf::from("Cargo.toml"),
        ];
        
        let validation = workspace.validate_commit_files(&files).unwrap();
        assert!(validation.valid);
        assert_eq!(validation.file_count, 2);
    }

    #[test]
    fn test_binary_detection() {
        let temp_dir = TempDir::new().unwrap();
        let root_path = temp_dir.path().to_path_buf();
        
        let workspace = WorkspaceManager::new(root_path.clone(), "test-workspace".to_string()).unwrap();
        
        // Create a text file
        let text_file = root_path.join("test.txt");
        fs::write(&text_file, "Hello, world!").unwrap();
        
        let is_binary = workspace.is_likely_binary(Path::new("test.txt")).unwrap();
        assert!(!is_binary);
        
        // Create a binary file
        let binary_file = root_path.join("test.bin");
        fs::write(&binary_file, &[0u8, 1, 2, 3, 0, 0, 0, 255]).unwrap();
        
        let is_binary = workspace.is_likely_binary(Path::new("test.bin")).unwrap();
        assert!(is_binary);
    }
}
