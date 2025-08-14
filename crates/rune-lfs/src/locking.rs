use anyhow::Result;
use std::{collections::HashMap, path::PathBuf, fs};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockingConfig {
    pub auto_lfs_size_mb: u64,
    pub lock_patterns: Vec<String>,
    pub lfs_patterns: Vec<String>,
    pub binary_patterns: Vec<String>,
    pub text_patterns: Vec<String>,
}

impl Default for LockingConfig {
    fn default() -> Self {
        Self {
            auto_lfs_size_mb: 100,
            lock_patterns: vec![
                "*.blend".to_string(),
                "*.psd".to_string(),
                "*.ai".to_string(),
                "*.sketch".to_string(),
                "*.fig".to_string(),
                "*.afdesign".to_string(),
                "*.afphoto".to_string(),
                "*.afpub".to_string(),
                "*.3ds".to_string(),
                "*.max".to_string(),
                "*.maya".to_string(),
                "*.fbx".to_string(),
                "*.obj".to_string(),
                "*.dae".to_string(),
                "*.stl".to_string(),
                "*.ply".to_string(),
                "*.x3d".to_string(),
                "*.unity".to_string(),
                "*.unitypackage".to_string(),
                "*.uasset".to_string(),
                "*.umap".to_string(),
            ],
            lfs_patterns: vec![
                "*.zip".to_string(),
                "*.rar".to_string(),
                "*.7z".to_string(),
                "*.tar.gz".to_string(),
                "*.tar.bz2".to_string(),
                "*.tar.xz".to_string(),
                "*.gz".to_string(),
                "*.bz2".to_string(),
                "*.xz".to_string(),
                "*.dmg".to_string(),
                "*.iso".to_string(),
                "*.img".to_string(),
                "*.vdi".to_string(),
                "*.vmdk".to_string(),
                "*.qcow2".to_string(),
                "*.exe".to_string(),
                "*.msi".to_string(),
                "*.app".to_string(),
                "*.deb".to_string(),
                "*.rpm".to_string(),
                "*.pkg".to_string(),
                "*.bin".to_string(),
                "*.so".to_string(),
                "*.dll".to_string(),
                "*.dylib".to_string(),
                "*.a".to_string(),
                "*.lib".to_string(),
                "*.mp3".to_string(),
                "*.wav".to_string(),
                "*.flac".to_string(),
                "*.aac".to_string(),
                "*.ogg".to_string(),
                "*.m4a".to_string(),
                "*.wma".to_string(),
                "*.mp4".to_string(),
                "*.avi".to_string(),
                "*.mkv".to_string(),
                "*.mov".to_string(),
                "*.wmv".to_string(),
                "*.flv".to_string(),
                "*.webm".to_string(),
                "*.m4v".to_string(),
                "*.3gp".to_string(),
                "*.jpg".to_string(),
                "*.jpeg".to_string(),
                "*.png".to_string(),
                "*.gif".to_string(),
                "*.bmp".to_string(),
                "*.tiff".to_string(),
                "*.tga".to_string(),
                "*.webp".to_string(),
                "*.ico".to_string(),
                "*.svg".to_string(),
                "*.eps".to_string(),
                "*.raw".to_string(),
                "*.cr2".to_string(),
                "*.nef".to_string(),
                "*.arw".to_string(),
                "*.dng".to_string(),
                "*.pdf".to_string(),
                "*.doc".to_string(),
                "*.docx".to_string(),
                "*.xls".to_string(),
                "*.xlsx".to_string(),
                "*.ppt".to_string(),
                "*.pptx".to_string(),
                "*.odt".to_string(),
                "*.ods".to_string(),
                "*.odp".to_string(),
                "*.rtf".to_string(),
            ],
            binary_patterns: vec![
                "*.exe".to_string(),
                "*.dll".to_string(),
                "*.so".to_string(),
                "*.dylib".to_string(),
                "*.a".to_string(),
                "*.lib".to_string(),
                "*.o".to_string(),
                "*.obj".to_string(),
                "*.class".to_string(),
                "*.jar".to_string(),
                "*.war".to_string(),
                "*.ear".to_string(),
                "*.pyc".to_string(),
                "*.pyo".to_string(),
                "*.pyd".to_string(),
                "*.rlib".to_string(),
                "*.wasm".to_string(),
            ],
            text_patterns: vec![
                "*.txt".to_string(),
                "*.md".to_string(),
                "*.rst".to_string(),
                "*.tex".to_string(),
                "*.adoc".to_string(),
                "*.org".to_string(),
                "*.yaml".to_string(),
                "*.yml".to_string(),
                "*.json".to_string(),
                "*.xml".to_string(),
                "*.toml".to_string(),
                "*.ini".to_string(),
                "*.cfg".to_string(),
                "*.conf".to_string(),
                "*.config".to_string(),
                "*.properties".to_string(),
                "*.env".to_string(),
                "*.log".to_string(),
                "*.csv".to_string(),
                "*.tsv".to_string(),
                "*.sql".to_string(),
                "*.html".to_string(),
                "*.htm".to_string(),
                "*.css".to_string(),
                "*.scss".to_string(),
                "*.sass".to_string(),
                "*.less".to_string(),
                "*.js".to_string(),
                "*.ts".to_string(),
                "*.jsx".to_string(),
                "*.tsx".to_string(),
                "*.vue".to_string(),
                "*.svelte".to_string(),
                "*.py".to_string(),
                "*.rs".to_string(),
                "*.go".to_string(),
                "*.java".to_string(),
                "*.c".to_string(),
                "*.cpp".to_string(),
                "*.cc".to_string(),
                "*.cxx".to_string(),
                "*.h".to_string(),
                "*.hpp".to_string(),
                "*.hh".to_string(),
                "*.hxx".to_string(),
                "*.cs".to_string(),
                "*.php".to_string(),
                "*.rb".to_string(),
                "*.pl".to_string(),
                "*.pm".to_string(),
                "*.sh".to_string(),
                "*.bash".to_string(),
                "*.zsh".to_string(),
                "*.fish".to_string(),
                "*.ps1".to_string(),
                "*.bat".to_string(),
                "*.cmd".to_string(),
                "*.dockerfile".to_string(),
                "Dockerfile".to_string(),
                "Makefile".to_string(),
                "makefile".to_string(),
                "*.mk".to_string(),
                "*.cmake".to_string(),
                "CMakeLists.txt".to_string(),
                "*.gradle".to_string(),
                "*.maven".to_string(),
                "*.pom".to_string(),
                "*.sbt".to_string(),
                "*.lock".to_string(),
                "*.gitignore".to_string(),
                "*.gitattributes".to_string(),
                "*.editorconfig".to_string(),
                "*.prettierrc".to_string(),
                "*.eslintrc".to_string(),
                "*.babelrc".to_string(),
                "*.npmrc".to_string(),
                "*.yarnrc".to_string(),
                "*.nvmrc".to_string(),
                "*.rvmrc".to_string(),
                "*.ruby-version".to_string(),
                "*.python-version".to_string(),
                "*.node-version".to_string(),
                "*.terraform".to_string(),
                "*.tf".to_string(),
                "*.tfvars".to_string(),
                "*.hcl".to_string(),
                "*.nomad".to_string(),
                "*.consul".to_string(),
                "*.vault".to_string(),
                "*.k8s".to_string(),
                "*.kubernetes".to_string(),
                "*.yml".to_string(),
                "*.yaml".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub enum DetectedFileType {
    Text,
    Binary,
    Archive,
    Media,
    Document,
    Code,
    Config,
    Lock,
    LargeBinary,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct FileAnalysis {
    pub file_type: DetectedFileType,
    pub should_lock: bool,
    pub should_lfs: bool,
    pub size_mb: f64,
    pub reason: String,
    pub recommendations: Vec<String>,
}

pub struct LockManager {
    pub config: LockingConfig,
}

impl LockManager {
    pub fn new() -> Self {
        Self {
            config: LockingConfig::default(),
        }
    }

    pub fn load_config(&mut self, repo_root: &PathBuf) -> Result<()> {
        let config_path = repo_root.join(".rune").join("locking.json");
        if config_path.exists() {
            let content = fs::read_to_string(config_path)?;
            self.config = serde_json::from_str(&content)?;
        }
        Ok(())
    }

    pub fn save_config(&self, repo_root: &PathBuf) -> Result<()> {
        let config_dir = repo_root.join(".rune");
        fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("locking.json");
        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    pub fn analyze_file(&self, file_path: &PathBuf) -> Result<FileAnalysis> {
        let file_name = file_path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        let file_size = file_path.metadata()
            .map(|m| m.len() as f64 / (1024.0 * 1024.0))
            .unwrap_or(0.0);

        let file_type = self.detect_file_type(file_name);
        let should_lock = self.should_file_be_locked(file_name);
        let should_lfs = self.should_file_use_lfs(file_name, file_size);

        let mut recommendations = Vec::new();
        let reason = match (&file_type, should_lock, should_lfs) {
            (DetectedFileType::Binary, true, true) => {
                recommendations.push("Consider using both file locking and LFS for this binary file".to_string());
                "Large binary file requiring both locking and LFS".to_string()
            },
            (DetectedFileType::Binary, true, false) => {
                recommendations.push("This binary file should be locked to prevent conflicts".to_string());
                "Binary file requiring locking".to_string()
            },
            (DetectedFileType::Binary, false, true) => {
                recommendations.push("This large binary file should use LFS for efficient storage".to_string());
                "Large binary file suitable for LFS".to_string()
            },
            (DetectedFileType::Binary, false, false) => {
                "Small binary file".to_string()
            },
            (DetectedFileType::Media, _, true) => {
                recommendations.push("Media files should typically use LFS for efficient storage".to_string());
                if file_size > 50.0 {
                    recommendations.push("Consider locking this large media file to prevent conflicts".to_string());
                }
                "Media file suitable for LFS".to_string()
            },
            (DetectedFileType::Media, _, false) => {
                "Small media file".to_string()
            },
            (DetectedFileType::Archive, _, true) => {
                recommendations.push("Archive files should use LFS due to their size".to_string());
                recommendations.push("Consider locking archives to prevent accidental modifications".to_string());
                "Archive file suitable for LFS and locking".to_string()
            },
            (DetectedFileType::Archive, _, false) => {
                "Small archive file".to_string()
            },
            (DetectedFileType::Document, _, _) => {
                if file_size > 10.0 {
                    recommendations.push("Large document files may benefit from LFS".to_string());
                }
                "Document file".to_string()
            },
            (DetectedFileType::Code, _, _) => {
                if file_size > 5.0 {
                    recommendations.push("Unusually large code file - consider refactoring or using LFS".to_string());
                }
                "Source code file".to_string()
            },
            (DetectedFileType::Config, _, _) => {
                "Configuration file".to_string()
            },
            (DetectedFileType::Text, _, _) => {
                if file_size > 1.0 {
                    recommendations.push("Large text file may benefit from LFS".to_string());
                }
                "Text file".to_string()
            },
            (DetectedFileType::Lock, _, _) => {
                recommendations.push("Lock files should typically be ignored by version control".to_string());
                "Lock file".to_string()
            },
            (DetectedFileType::LargeBinary, _, _) => {
                recommendations.push("Large binary file should use both LFS and locking".to_string());
                "Large binary file requiring special handling".to_string()
            },
            (DetectedFileType::Unknown, _, _) => {
                if should_lock {
                    recommendations.push("This file type should be locked to prevent conflicts".to_string());
                }
                if should_lfs {
                    recommendations.push("Consider using LFS for this file".to_string());
                }
                if file_size > 10.0 {
                    recommendations.push("Unknown large file type - consider LFS".to_string());
                }
                "Unknown file type".to_string()
            },
        };

        // Add general locking recommendations if not already covered
        if should_lock && recommendations.is_empty() {
            recommendations.push("This file should be locked to prevent conflicts during editing".to_string());
        }
        if should_lfs && recommendations.is_empty() {
            recommendations.push("This file should use LFS for efficient storage".to_string());
        }

        Ok(FileAnalysis {
            file_type,
            should_lock,
            should_lfs,
            size_mb: file_size,
            reason,
            recommendations,
        })
    }

    fn detect_file_type(&self, file_name: &str) -> DetectedFileType {
        let lower_name = file_name.to_lowercase();
        
        // Check code files first (before text patterns that might overlap)
        if lower_name.ends_with(".rs") || lower_name.ends_with(".py") ||
           lower_name.ends_with(".js") || lower_name.ends_with(".ts") ||
           lower_name.ends_with(".java") || lower_name.ends_with(".c") ||
           lower_name.ends_with(".cpp") || lower_name.ends_with(".go") {
            return DetectedFileType::Code;
        }

        // Config files (before text patterns)
        if lower_name.ends_with(".json") || lower_name.ends_with(".yaml") ||
           lower_name.ends_with(".toml") || lower_name.ends_with(".ini") ||
           lower_name.ends_with(".config") || lower_name.ends_with(".cfg") {
            return DetectedFileType::Config;
        }

        // Lock files
        if lower_name.contains(".lock") || lower_name == "cargo.lock" ||
           lower_name == "package-lock.json" || lower_name == "yarn.lock" {
            return DetectedFileType::Lock;
        }

        // Check binary patterns
        if self.matches_patterns(&lower_name, &self.config.binary_patterns) {
            return DetectedFileType::Binary;
        }

        // Media files
        if lower_name.ends_with(".mp3") || lower_name.ends_with(".mp4") || 
           lower_name.ends_with(".avi") || lower_name.ends_with(".wav") ||
           lower_name.ends_with(".jpg") || lower_name.ends_with(".png") ||
           lower_name.ends_with(".gif") || lower_name.ends_with(".svg") {
            return DetectedFileType::Media;
        }

        // Archive files
        if lower_name.ends_with(".zip") || lower_name.ends_with(".tar.gz") ||
           lower_name.ends_with(".rar") || lower_name.ends_with(".7z") {
            return DetectedFileType::Archive;
        }

        // Document files
        if lower_name.ends_with(".pdf") || lower_name.ends_with(".doc") ||
           lower_name.ends_with(".docx") || lower_name.ends_with(".xls") ||
           lower_name.ends_with(".ppt") || lower_name.ends_with(".odt") {
            return DetectedFileType::Document;
        }

        // Check text patterns last (catch-all for remaining text files)
        if self.matches_patterns(&lower_name, &self.config.text_patterns) {
            return DetectedFileType::Text;
        }

        DetectedFileType::Unknown
    }

    fn should_file_be_locked(&self, file_name: &str) -> bool {
        self.matches_patterns(&file_name.to_lowercase(), &self.config.lock_patterns)
    }

    fn should_file_use_lfs(&self, file_name: &str, size_mb: f64) -> bool {
        // Auto-LFS for large files
        if size_mb > self.config.auto_lfs_size_mb as f64 {
            return true;
        }

        // Pattern-based LFS
        self.matches_patterns(&file_name.to_lowercase(), &self.config.lfs_patterns)
    }

    fn matches_patterns(&self, file_name: &str, patterns: &[String]) -> bool {
        patterns.iter().any(|pattern| {
            // Simple glob matching - could be enhanced with proper glob library
            if pattern.contains('*') {
                let pattern_parts: Vec<&str> = pattern.split('*').collect();
                if pattern_parts.len() == 2 && pattern_parts[0].is_empty() {
                    // Pattern like "*.ext"
                    file_name.ends_with(pattern_parts[1])
                } else if pattern_parts.len() == 2 && pattern_parts[1].is_empty() {
                    // Pattern like "prefix*"
                    file_name.starts_with(pattern_parts[0])
                } else {
                    // More complex patterns - basic implementation
                    file_name.contains(pattern_parts[0]) && file_name.contains(pattern_parts[1])
                }
            } else {
                file_name == pattern
            }
        })
    }

    pub fn get_lock_status(&self, repo_root: &PathBuf) -> Result<HashMap<String, bool>> {
        let locks_dir = repo_root.join(".rune").join("locks");
        let mut status = HashMap::new();

        if locks_dir.exists() {
            for entry in fs::read_dir(locks_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    if let Some(file_name) = entry.file_name().to_str() {
                        if file_name.ends_with(".lock") {
                            let locked_file = file_name.trim_end_matches(".lock");
                            status.insert(locked_file.to_string(), true);
                        }
                    }
                }
            }
        }

        Ok(status)
    }

    pub fn lock_file(&self, repo_root: &PathBuf, file_path: &str, user: &str) -> Result<()> {
        let locks_dir = repo_root.join(".rune").join("locks");
        fs::create_dir_all(&locks_dir)?;
        
        let lock_file = locks_dir.join(format!("{}.lock", file_path.replace('/', "_")));
        let lock_info = serde_json::json!({
            "file": file_path,
            "user": user,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "machine": std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string())
        });
        
        fs::write(lock_file, serde_json::to_string_pretty(&lock_info)?)?;
        Ok(())
    }

    pub fn unlock_file(&self, repo_root: &PathBuf, file_path: &str) -> Result<()> {
        let locks_dir = repo_root.join(".rune").join("locks");
        let lock_file = locks_dir.join(format!("{}.lock", file_path.replace('/', "_")));
        
        if lock_file.exists() {
            fs::remove_file(lock_file)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_lock_manager_creation() {
        let manager = LockManager::new();
        assert!(!manager.config.lock_patterns.is_empty());
        assert!(!manager.config.lfs_patterns.is_empty());
    }

    #[test]
    fn test_file_type_detection() {
        let manager = LockManager::new();
        
        assert!(matches!(manager.detect_file_type("test.rs"), DetectedFileType::Code));
        assert!(matches!(manager.detect_file_type("config.json"), DetectedFileType::Config));
        assert!(matches!(manager.detect_file_type("image.png"), DetectedFileType::Media));
        assert!(matches!(manager.detect_file_type("archive.zip"), DetectedFileType::Archive));
        assert!(matches!(manager.detect_file_type("document.pdf"), DetectedFileType::Document));
        assert!(matches!(manager.detect_file_type("Cargo.lock"), DetectedFileType::Lock));
    }

    #[test]
    fn test_pattern_matching() {
        let manager = LockManager::new();
        
        assert!(manager.matches_patterns("test.blend", &manager.config.lock_patterns));
        assert!(manager.matches_patterns("video.mp4", &manager.config.lfs_patterns));
        assert!(manager.matches_patterns("binary.exe", &manager.config.binary_patterns));
        assert!(manager.matches_patterns("source.rs", &manager.config.text_patterns));
    }

    #[test]
    fn test_lfs_size_threshold() {
        let manager = LockManager::new();
        
        // Should use LFS for large files regardless of type
        assert!(manager.should_file_use_lfs("largefile.txt", 200.0));
        
        // Should use LFS for pattern matches even if small
        assert!(manager.should_file_use_lfs("video.mp4", 5.0));
        
        // Should not use LFS for small text files not in patterns
        assert!(!manager.should_file_use_lfs("small.unknown", 1.0));
    }

    #[test]
    fn test_file_analysis() {
        let manager = LockManager::new();
        
        // Create a temporary file for testing
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join("test.blend");
        std::fs::write(&test_file, b"test content").unwrap();
        
        let analysis = manager.analyze_file(&test_file).unwrap();
        assert!(analysis.should_lock);
        assert!(!analysis.recommendations.is_empty());
        
        // Clean up
        std::fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_config_serialization() {
        let config = LockingConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: LockingConfig = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(config.auto_lfs_size_mb, deserialized.auto_lfs_size_mb);
        assert_eq!(config.lock_patterns.len(), deserialized.lock_patterns.len());
        assert_eq!(config.lfs_patterns.len(), deserialized.lfs_patterns.len());
    }
}
