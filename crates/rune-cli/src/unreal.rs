use colored::*;
use std::path::Path;
use std::fs;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::intelligence::{IntelligentFileAnalyzer, FileAnalysis};
use crate::style::Style;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LockingConfig {
    pub enabled: bool,
    pub project_type: Option<ProjectType>,
    pub lock_management: LockManagement,
    pub file_handling: FileHandling,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProjectType {
    GameDevelopment,    // Unreal, Unity, Godot, etc.
    WebDevelopment,     // React, Angular, Vue, etc.
    MobileApp,          // iOS, Android
    DesktopApp,         // Electron, native apps
    DataScience,        // Jupyter notebooks, datasets
    DesignAssets,       // Figma, Photoshop, video files
    Documentation,      // Wikis, books, technical docs
    General,            // Generic project
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LockManagement {
    pub intelligent_locking: bool,
    pub auto_unlock_on_branch: bool,
    pub release_lock_cleanup: bool,
    pub timeout_hours: u32,
    pub lock_inheritance: LockInheritance,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum LockInheritance {
    None,           // No lock inheritance between branches
    Smart,          // Intelligent inheritance based on file type and patterns
    Explicit,       // Manual inheritance control
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileHandling {
    pub binary_file_detection: bool,
    pub large_file_lfs: bool,
    pub conflict_prevention: bool,
    pub metadata_cleanup: bool,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub file_type: DetectedFileType,
    pub dependencies: Vec<String>,
    pub size_mb: f64,
    pub lock_status: LockStatus,
    pub merge_strategy: MergeStrategy,
    pub conflict_risk: f64,
}

#[derive(Debug, Clone)]
pub enum DetectedFileType {
    // Development
    SourceCode(String),      // Language
    Configuration,
    Database,
    
    // Assets
    BinaryAsset(String),     // Type description
    Media(MediaType),
    Document(DocumentType),
    
    // Data
    Dataset,
    Notebook,
    
    // Build/Deploy
    BuildArtifact,
    Dependency,
    
    Unknown,
}

#[derive(Debug, Clone)]
pub enum MediaType {
    Image, Video, Audio, Model3D, Texture, Font,
}

#[derive(Debug, Clone)]
pub enum DocumentType {
    Text, Markdown, PDF, Presentation, Spreadsheet,
}

#[derive(Debug, Clone)]
pub enum LockStatus {
    Unlocked,
    LockedByUser(String),
    LockedByBranch(String),
    SmartLocked(SmartLockReason),
}

#[derive(Debug, Clone)]
pub enum SmartLockReason {
    ActiveDevelopment,
    PendingRelease,
    ConflictPrevention,
    DependencyProtection,
    LargeFileOperation,
    TeamCoordination,
}

#[derive(Debug, Clone)]
pub enum MergeStrategy {
    TextMerge,           // Standard text-based merge
    BinaryReplace,       // Treat as binary, no merge
    AssistedMerge,       // Use external tools
    IntelligentMerge,    // AI-assisted merge
    ManualOnly,          // Require manual resolution
}

pub struct LockManager {
    config: LockingConfig,
    analyzer: IntelligentFileAnalyzer,
    locks: HashMap<String, LockStatus>,
}

impl Default for LockingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            project_type: None,
            lock_management: LockManagement {
                intelligent_locking: true,
                auto_unlock_on_branch: true,
                release_lock_cleanup: true,
                timeout_hours: 24,
                lock_inheritance: LockInheritance::Smart,
            },
            file_handling: FileHandling {
                binary_file_detection: true,
                large_file_lfs: true,
                conflict_prevention: true,
                metadata_cleanup: true,
            },
        }
    }
}

impl LockManager {
    pub fn new() -> Self {
        Self {
            config: LockingConfig::default(),
            analyzer: IntelligentFileAnalyzer::new(),
            locks: HashMap::new(),
        }
    }

    pub fn detect_project_type(&mut self, path: &str) -> Result<Option<ProjectType>, std::io::Error> {
        if let Ok(entries) = fs::read_dir(path) {
            let files: Vec<_> = entries
                .filter_map(|entry| entry.ok())
                .collect();

            // Game Development
            if files.iter().any(|f| {
                let name = f.file_name().to_string_lossy().to_lowercase();
                name.ends_with(".uproject") || name.ends_with(".unity") || 
                name.contains("godot") || name == "assets" || name == "content"
            }) {
                self.config.project_type = Some(ProjectType::GameDevelopment);
                self.config.enabled = true;
                println!("{} Game development project detected!", "🎮".blue());
                return Ok(Some(ProjectType::GameDevelopment));
            }

            // Web Development
            if files.iter().any(|f| {
                let name = f.file_name().to_string_lossy().to_lowercase();
                name == "package.json" || name == "angular.json" || 
                name == "vue.config.js" || name == "webpack.config.js"
            }) {
                self.config.project_type = Some(ProjectType::WebDevelopment);
                self.config.enabled = true;
                println!("{} Web development project detected!", "🌐".blue());
                return Ok(Some(ProjectType::WebDevelopment));
            }

            // Mobile Development
            if files.iter().any(|f| {
                let name = f.file_name().to_string_lossy().to_lowercase();
                name.ends_with(".xcodeproj") || name == "build.gradle" || 
                name == "pubspec.yaml" || name == "app.json"
            }) {
                self.config.project_type = Some(ProjectType::MobileApp);
                self.config.enabled = true;
                println!("{} Mobile app project detected!", "📱".blue());
                return Ok(Some(ProjectType::MobileApp));
            }

            // Data Science
            if files.iter().any(|f| {
                let name = f.file_name().to_string_lossy().to_lowercase();
                name.ends_with(".ipynb") || name == "requirements.txt" || 
                name == "environment.yml" || name.contains("dataset")
            }) {
                self.config.project_type = Some(ProjectType::DataScience);
                self.config.enabled = true;
                println!("{} Data science project detected!", "📊".blue());
                return Ok(Some(ProjectType::DataScience));
            }

            // Design Assets
            if files.iter().any(|f| {
                let ext = f.path().extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                matches!(ext.as_str(), "psd" | "ai" | "sketch" | "fig" | "xd" | "blend" | "ma" | "mb")
            }) {
                self.config.project_type = Some(ProjectType::DesignAssets);
                self.config.enabled = true;
                println!("{} Design asset project detected!", "🎨".blue());
                return Ok(Some(ProjectType::DesignAssets));
            }

            // Documentation
            if files.iter().filter(|f| {
                let ext = f.path().extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                matches!(ext.as_str(), "md" | "rst" | "tex" | "adoc")
            }).count() > 5 {
                self.config.project_type = Some(ProjectType::Documentation);
                self.config.enabled = true;
                println!("{} Documentation project detected!", "📚".blue());
                return Ok(Some(ProjectType::Documentation));
            }
        }

        // Default to general if we have some files but no specific type
        self.config.project_type = Some(ProjectType::General);
        println!("{} General project - intelligent locking available", "📁".blue());
        Ok(Some(ProjectType::General))
    }

    pub fn analyze_file(&mut self, file_path: &str) -> Result<FileInfo, std::io::Error> {
        let path = Path::new(file_path);
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let file_type = self.detect_file_type(&extension, file_path);
        let metadata = fs::metadata(file_path)?;
        let size_mb = metadata.len() as f64 / 1024.0 / 1024.0;

        // Determine merge strategy based on file type and project type
        let merge_strategy = self.determine_merge_strategy(&file_type);

        // Calculate conflict risk based on file patterns
        let conflict_risk = self.calculate_conflict_risk(file_path, &file_type);

        // Check current lock status
        let lock_status = self.locks.get(file_path)
            .cloned()
            .unwrap_or(LockStatus::Unlocked);

        // Analyze dependencies
        let dependencies = self.analyze_dependencies(file_path, &file_type);

        Ok(FileInfo {
            file_type,
            dependencies,
            size_mb,
            lock_status,
            merge_strategy,
            conflict_risk,
        })
    }

    fn detect_file_type(&self, extension: &str, file_path: &str) -> DetectedFileType {
        match extension {
            // Source Code
            "rs" => DetectedFileType::SourceCode("Rust".to_string()),
            "py" => DetectedFileType::SourceCode("Python".to_string()),
            "js" | "ts" => DetectedFileType::SourceCode("JavaScript/TypeScript".to_string()),
            "go" => DetectedFileType::SourceCode("Go".to_string()),
            "java" | "kt" => DetectedFileType::SourceCode("JVM".to_string()),
            "c" | "cpp" | "cc" | "cxx" => DetectedFileType::SourceCode("C/C++".to_string()),
            "cs" => DetectedFileType::SourceCode("C#".to_string()),
            "swift" => DetectedFileType::SourceCode("Swift".to_string()),
            "dart" => DetectedFileType::SourceCode("Dart".to_string()),

            // Configuration
            "json" | "yaml" | "yml" | "toml" | "ini" | "conf" | "cfg" => DetectedFileType::Configuration,

            // Media
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tiff" | "webp" => DetectedFileType::Media(MediaType::Image),
            "mp4" | "avi" | "mov" | "mkv" | "webm" => DetectedFileType::Media(MediaType::Video),
            "mp3" | "wav" | "ogg" | "flac" | "aac" => DetectedFileType::Media(MediaType::Audio),
            "obj" | "fbx" | "dae" | "3ds" | "blend" => DetectedFileType::Media(MediaType::Model3D),
            "ttf" | "otf" | "woff" | "woff2" => DetectedFileType::Media(MediaType::Font),

            // Documents
            "md" | "rst" | "adoc" => DetectedFileType::Document(DocumentType::Markdown),
            "pdf" => DetectedFileType::Document(DocumentType::PDF),
            "pptx" | "ppt" | "odp" => DetectedFileType::Document(DocumentType::Presentation),
            "xlsx" | "xls" | "ods" | "csv" => DetectedFileType::Document(DocumentType::Spreadsheet),
            "txt" | "rtf" => DetectedFileType::Document(DocumentType::Text),

            // Data Science
            "ipynb" => DetectedFileType::Notebook,
            "csv" | "parquet" | "h5" | "hdf5" | "npz" => DetectedFileType::Dataset,

            // Databases
            "db" | "sqlite" | "sqlite3" => DetectedFileType::Database,

            // Build artifacts
            "exe" | "dll" | "so" | "dylib" | "jar" | "war" | "deb" | "rpm" => DetectedFileType::BuildArtifact,

            // Project-specific detection
            _ => self.detect_project_specific_type(file_path, extension),
        }
    }

    fn detect_project_specific_type(&self, file_path: &str, extension: &str) -> DetectedFileType {
        match &self.config.project_type {
            Some(ProjectType::GameDevelopment) => {
                match extension {
                    "uasset" | "umap" => DetectedFileType::BinaryAsset("Unreal Asset".to_string()),
                    "unity" => DetectedFileType::BinaryAsset("Unity Scene".to_string()),
                    "prefab" => DetectedFileType::BinaryAsset("Unity Prefab".to_string()),
                    "mat" => DetectedFileType::BinaryAsset("Material".to_string()),
                    _ => DetectedFileType::Unknown,
                }
            },
            Some(ProjectType::DesignAssets) => {
                match extension {
                    "psd" => DetectedFileType::BinaryAsset("Photoshop Document".to_string()),
                    "ai" => DetectedFileType::BinaryAsset("Illustrator File".to_string()),
                    "sketch" => DetectedFileType::BinaryAsset("Sketch File".to_string()),
                    "fig" => DetectedFileType::BinaryAsset("Figma File".to_string()),
                    "xd" => DetectedFileType::BinaryAsset("Adobe XD File".to_string()),
                    _ => DetectedFileType::Unknown,
                }
            },
            _ => DetectedFileType::Unknown,
        }
    }

    fn determine_merge_strategy(&self, file_type: &DetectedFileType) -> MergeStrategy {
        match file_type {
            DetectedFileType::SourceCode(_) => MergeStrategy::TextMerge,
            DetectedFileType::Configuration => MergeStrategy::IntelligentMerge,
            DetectedFileType::Document(DocumentType::Markdown) => MergeStrategy::TextMerge,
            DetectedFileType::Document(DocumentType::Text) => MergeStrategy::TextMerge,
            DetectedFileType::BinaryAsset(_) => MergeStrategy::BinaryReplace,
            DetectedFileType::Media(_) => MergeStrategy::BinaryReplace,
            DetectedFileType::Notebook => MergeStrategy::AssistedMerge,
            DetectedFileType::Database => MergeStrategy::ManualOnly,
            DetectedFileType::BuildArtifact => MergeStrategy::BinaryReplace,
            _ => MergeStrategy::TextMerge,
        }
    }

    fn calculate_conflict_risk(&self, file_path: &str, file_type: &DetectedFileType) -> f64 {
        let mut risk = 0.0;

        // Base risk by file type
        risk += match file_type {
            DetectedFileType::Configuration => 0.7,
            DetectedFileType::SourceCode(_) => 0.4,
            DetectedFileType::Document(_) => 0.2,
            DetectedFileType::BinaryAsset(_) => 0.9,
            DetectedFileType::Database => 0.8,
            _ => 0.3,
        };

        // Path-based risk factors
        let path_lower = file_path.to_lowercase();
        if path_lower.contains("config") || path_lower.contains("settings") {
            risk += 0.3;
        }
        if path_lower.contains("shared") || path_lower.contains("common") {
            risk += 0.2;
        }
        if path_lower.contains("main") || path_lower.contains("index") {
            risk += 0.2;
        }

        risk.min(1.0)
    }

    fn analyze_dependencies(&self, _file_path: &str, _file_type: &DetectedFileType) -> Vec<String> {
        // Placeholder for dependency analysis
        // Would be implemented based on file type and project structure
        vec![]
    }

    pub fn intelligent_lock(&mut self, file_path: &str, reason: SmartLockReason) -> Result<(), String> {
        if !self.config.lock_management.intelligent_locking {
            return Err("Intelligent locking is disabled".to_string());
        }

        let current_user = whoami::username();
        
        // Check if already locked by someone else
        if let Some(existing_lock) = self.locks.get(file_path) {
            match existing_lock {
                LockStatus::LockedByUser(user) if user != &current_user => {
                    return Err(format!("File is already locked by {}", user));
                },
                _ => {} // Can override our own locks or smart locks
            }
        }

        self.locks.insert(file_path.to_string(), LockStatus::SmartLocked(reason.clone()));
        
        println!("{} Smart lock applied: {}", "🔒".yellow(), Style::file_path(file_path));
        println!("  Reason: {:?}", reason);
        println!("  User: {}", current_user.cyan());
        
        Ok(())
    }

    pub fn release_lock(&mut self, file_path: &str, force: bool) -> Result<(), String> {
        let current_user = whoami::username();
        
        if let Some(lock_status) = self.locks.get(file_path) {
            match lock_status {
                LockStatus::LockedByUser(user) if user != &current_user && !force => {
                    return Err(format!("Cannot release lock owned by {}", user));
                },
                LockStatus::Unlocked => {
                    return Err("File is not locked".to_string());
                },
                _ => {
                    self.locks.remove(file_path);
                    println!("{} Lock released: {}", "🔓".green(), Style::file_path(file_path));
                }
            }
        } else {
            return Err("File is not locked".to_string());
        }
        
        Ok(())
    }

    pub fn branch_switch_cleanup(&mut self, old_branch: &str, new_branch: &str) -> Result<(), String> {
        if !self.config.lock_management.auto_unlock_on_branch {
            return Ok(());
        }

        let mut locks_to_release = Vec::new();
        let current_user = whoami::username();

        println!("{} Cleaning up locks for branch switch: {} → {}", 
                "🔄".blue(), old_branch.yellow(), new_branch.green());

        for (file_path, lock_status) in &self.locks {
            let should_release = match lock_status {
                LockStatus::LockedByUser(user) if user == &current_user => {
                    // Release our own locks based on inheritance setting
                    match self.config.lock_management.lock_inheritance {
                        LockInheritance::None => true,
                        LockInheritance::Smart => {
                            // Smart inheritance: keep locks on files that are likely to be shared
                            !self.is_likely_shared_file(file_path)
                        },
                        LockInheritance::Explicit => false, // Keep locks for explicit management
                    }
                },
                LockStatus::LockedByBranch(branch) if branch == old_branch => true,
                LockStatus::SmartLocked(SmartLockReason::ActiveDevelopment) => true,
                _ => false,
            };

            if should_release {
                locks_to_release.push(file_path.clone());
            }
        }

        for file_path in locks_to_release {
            self.locks.remove(&file_path);
            println!("  🔓 Released lock: {}", Style::file_path(&file_path));
        }

        Ok(())
    }

    pub fn release_cleanup(&mut self, release_tag: &str) -> Result<(), String> {
        if !self.config.lock_management.release_lock_cleanup {
            return Ok(());
        }

        println!("{} Performing release lock cleanup for: {}", 
                "🚀".green(), release_tag.bold());

        let mut locks_to_release = Vec::new();

        for (file_path, lock_status) in &self.locks {
            let should_release = match lock_status {
                LockStatus::SmartLocked(SmartLockReason::PendingRelease) => true,
                LockStatus::SmartLocked(SmartLockReason::ActiveDevelopment) => {
                    // Release development locks for stable files
                    self.is_stable_for_release(file_path)
                },
                _ => false,
            };

            if should_release {
                locks_to_release.push(file_path.clone());
            }
        }

        for file_path in locks_to_release {
            self.locks.remove(&file_path);
            println!("  🔓 Released release lock: {}", Style::file_path(&file_path));
        }

        println!("✨ Release cleanup complete!");
        Ok(())
    }

    fn is_likely_shared_file(&self, file_path: &str) -> bool {
        let path_lower = file_path.to_lowercase();
        
        // General shared patterns
        path_lower.contains("config") ||
        path_lower.contains("settings") ||
        path_lower.contains("shared") ||
        path_lower.contains("common") ||
        path_lower.contains("core") ||
        path_lower.contains("main") ||
        path_lower.contains("index") ||
        
        // Project-specific patterns
        match &self.config.project_type {
            Some(ProjectType::GameDevelopment) => {
                path_lower.contains("gamemode") ||
                path_lower.contains("playercontroller") ||
                path_lower.contains("gameinstance")
            },
            Some(ProjectType::WebDevelopment) => {
                path_lower.contains("package.json") ||
                path_lower.contains("webpack") ||
                path_lower.contains("app.") ||
                path_lower.contains("main.")
            },
            _ => false,
        }
    }

    fn is_stable_for_release(&self, _file_path: &str) -> bool {
        // Placeholder logic for determining if a file is stable for release
        // Could integrate with CI/CD status, testing results, etc.
        true
    }

    pub fn show_lock_status(&self) {
        if self.locks.is_empty() {
            println!("{} No active locks", "🔓".green());
            return;
        }

        println!("{} Active Locks", "🔒".yellow());
        for (file_path, lock_status) in &self.locks {
            let status_str = match lock_status {
                LockStatus::LockedByUser(user) => format!("Locked by {}", user.cyan()),
                LockStatus::LockedByBranch(branch) => format!("Branch lock: {}", branch.yellow()),
                LockStatus::SmartLocked(reason) => format!("Smart lock: {:?}", reason).blue().to_string(),
                LockStatus::Unlocked => "Unlocked".green().to_string(),
            };
            
            println!("  {} - {}", Style::file_path(file_path), status_str);
        }
    }

    pub fn suggest_lfs_candidates(&mut self, repo_path: &str) -> Result<Vec<String>, std::io::Error> {
        let mut candidates = Vec::new();
        
        if let Ok(entries) = fs::read_dir(repo_path) {
            for entry in entries.flatten() {
                if entry.metadata()?.is_file() {
                    let file_path = entry.path().to_string_lossy().to_string();
                    
                    if let Ok(file_info) = self.analyze_file(&file_path) {
                        // Suggest LFS for large files or specific types
                        let should_use_lfs = file_info.size_mb > 10.0 || 
                                           matches!(file_info.file_type, 
                                                   DetectedFileType::Media(_) | 
                                                   DetectedFileType::BinaryAsset(_) |
                                                   DetectedFileType::BuildArtifact |
                                                   DetectedFileType::Dataset);
                        
                        if should_use_lfs {
                            candidates.push(file_path);
                        }
                    }
                }
            }
        }
        
        Ok(candidates)
    }
}

// CLI Integration functions
pub fn handle_lock_command(cmd: LockCmd) -> anyhow::Result<()> {
    let mut manager = LockManager::new();
    
    match cmd {
        LockCmd::Detect => {
            if let Ok(current_dir) = std::env::current_dir() {
                let project_type = manager.detect_project_type(&current_dir.to_string_lossy())?;
                if project_type.is_none() {
                    println!("{} No specific project type detected - general locking available", "ℹ️".blue());
                } else {
                    println!("🚀 Intelligent file locking enabled for this project type!");
                }
            }
        },
        LockCmd::Lock { files, reason } => {
            let lock_reason = match reason.as_deref() {
                Some("development") => SmartLockReason::ActiveDevelopment,
                Some("release") => SmartLockReason::PendingRelease,
                Some("conflict") => SmartLockReason::ConflictPrevention,
                Some("dependency") => SmartLockReason::DependencyProtection,
                Some("large") => SmartLockReason::LargeFileOperation,
                Some("team") => SmartLockReason::TeamCoordination,
                _ => SmartLockReason::ActiveDevelopment,
            };
            
            for file in files {
                let file_str = file.to_string_lossy();
                match manager.intelligent_lock(&file_str, lock_reason.clone()) {
                    Ok(_) => {},
                    Err(e) => println!("❌ Failed to lock {}: {}", Style::file_path(&file_str), e.red()),
                }
            }
        },
        LockCmd::Unlock { files, force } => {
            for file in files {
                let file_str = file.to_string_lossy();
                match manager.release_lock(&file_str, force) {
                    Ok(_) => {},
                    Err(e) => println!("❌ Failed to unlock {}: {}", Style::file_path(&file_str), e.red()),
                }
            }
        },
        LockCmd::Status => {
            manager.show_lock_status();
        },
        LockCmd::CleanupBranch { old_branch, new_branch } => {
            manager.branch_switch_cleanup(&old_branch, &new_branch)?;
        },
        LockCmd::CleanupRelease { release_tag } => {
            manager.release_cleanup(&release_tag)?;
        },
        LockCmd::LfsSuggestions => {
            if let Ok(current_dir) = std::env::current_dir() {
                let candidates = manager.suggest_lfs_candidates(&current_dir.to_string_lossy())?;
                
                if candidates.is_empty() {
                    println!("{} No LFS candidates found", "✅".green());
                } else {
                    println!("{} LFS Candidates Found:", "📦".blue());
                    for candidate in candidates {
                        println!("  {}", Style::file_path(&candidate));
                    }
                    println!("\nRun 'git lfs track' for these files to improve performance");
                }
            }
        },
        LockCmd::Analyze { files } => {
            for file in files {
                let file_str = file.to_string_lossy();
                match manager.analyze_file(&file_str) {
                    Ok(info) => {
                        println!("\n{} Analysis: {}", "🔍".blue(), Style::file_path(&file_str));
                        println!("  Type: {:?}", info.file_type);
                        println!("  Size: {:.1} MB", info.size_mb);
                        println!("  Merge Strategy: {:?}", info.merge_strategy);
                        println!("  Conflict Risk: {:.0}%", info.conflict_risk * 100.0);
                        println!("  Lock Status: {:?}", info.lock_status);
                        
                        if info.conflict_risk > 0.7 {
                            println!("  {} High conflict risk - consider locking", "⚠️".yellow());
                        }
                        if info.size_mb > 10.0 {
                            println!("  {} Large file - consider LFS", "📦".blue());
                        }
                    },
                    Err(e) => println!("❌ Failed to analyze {}: {}", Style::file_path(&file_str), e),
                }
            }
        },
    }
    
    Ok(())
}

#[derive(clap::Subcommand, Debug)]
pub enum LockCmd {
    /// Auto-detect project type and enable intelligent locking
    Detect,
    /// Lock files with intelligent management
    Lock {
        files: Vec<std::path::PathBuf>,
        #[arg(long, help = "Lock reason: development, release, conflict, dependency, large, team")]
        reason: Option<String>,
    },
    /// Unlock files
    Unlock {
        files: Vec<std::path::PathBuf>,
        #[arg(long, help = "Force unlock even if owned by another user")]
        force: bool,
    },
    /// Show lock status
    Status,
    /// Cleanup locks when switching branches
    CleanupBranch {
        old_branch: String,
        new_branch: String,
    },
    /// Cleanup locks for release
    CleanupRelease {
        release_tag: String,
    },
    /// Show LFS suggestions for large/binary files
    LfsSuggestions,
    /// Analyze files for lock recommendations
    Analyze {
        files: Vec<std::path::PathBuf>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LockManagement {
    pub intelligent_locking: bool,
    pub auto_unlock_on_branch: bool,
    pub release_lock_cleanup: bool,
    pub timeout_hours: u32,
    pub lock_inheritance: LockInheritance,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum LockInheritance {
    None,           // No lock inheritance between branches
    Smart,          // Intelligent inheritance based on file type
    Explicit,       // Manual inheritance control
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetOptimization {
    pub binary_asset_handling: bool,
    pub blueprint_merge_assistance: bool,
    pub large_asset_lfs: bool,
    pub metadata_cleanup: bool,
}

#[derive(Debug, Clone)]
pub struct UnrealAssetInfo {
    pub asset_type: UnrealAssetType,
    pub dependencies: Vec<String>,
    pub size_mb: f64,
    pub lock_status: LockStatus,
    pub merge_strategy: MergeStrategy,
}

#[derive(Debug, Clone)]
pub enum UnrealAssetType {
    Blueprint,
    StaticMesh,
    Texture,
    Material,
    Animation,
    Sound,
    Map,
    DataAsset,
    Other(String),
}

#[derive(Debug, Clone)]
pub enum LockStatus {
    Unlocked,
    LockedByUser(String),
    LockedByBranch(String),
    SmartLocked(SmartLockReason),
}

#[derive(Debug, Clone)]
pub enum SmartLockReason {
    ActiveDevelopment,
    PendingRelease,
    ConflictPrevention,
    DependencyProtection,
}

#[derive(Debug, Clone)]
pub enum MergeStrategy {
    Binary,          // Treat as binary, no merge
    Assisted,        // Use Unreal's merge tools
    Intelligent,     // AI-assisted merge
    Manual,          // Require manual resolution
}

pub struct UnrealManager {
    integration: UnrealIntegration,
    analyzer: IntelligentFileAnalyzer,
    locks: HashMap<String, LockStatus>,
}

impl Default for UnrealIntegration {
    fn default() -> Self {
        Self {
            enabled: false,
            project_path: None,
            engine_version: None,
            lock_management: LockManagement {
                intelligent_locking: true,
                auto_unlock_on_branch: true,
                release_lock_cleanup: true,
                timeout_hours: 24,
                lock_inheritance: LockInheritance::Smart,
            },
            asset_optimization: AssetOptimization {
                binary_asset_handling: true,
                blueprint_merge_assistance: true,
                large_asset_lfs: true,
                metadata_cleanup: true,
            },
        }
    }
}

impl UnrealManager {
    pub fn new() -> Self {
        Self {
            integration: UnrealIntegration::default(),
            analyzer: IntelligentFileAnalyzer::new(),
            locks: HashMap::new(),
        }
    }

    pub fn detect_unreal_project(&mut self, path: &str) -> Result<bool, std::io::Error> {
        // Look for .uproject files
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "uproject" {
                        self.integration.enabled = true;
                        self.integration.project_path = Some(path.to_string());
                        
                        // Try to detect engine version
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            if let Ok(project_json) = serde_json::from_str::<serde_json::Value>(&content) {
                                if let Some(engine_version) = project_json.get("EngineAssociation") {
                                    self.integration.engine_version = engine_version.as_str().map(|s| s.to_string());
                                }
                            }
                        }
                        
                        println!("{} Unreal Engine project detected!", "🎮".blue());
                        println!("  Project: {}", Style::file_path(&entry.file_name().to_string_lossy()));
                        if let Some(version) = &self.integration.engine_version {
                            println!("  Engine Version: {}", version.cyan());
                        }
                        println!("  🚀 Revolutionary file management enabled!");
                        
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    pub fn analyze_unreal_asset(&mut self, file_path: &str) -> Result<UnrealAssetInfo, std::io::Error> {
        let path = Path::new(file_path);
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let asset_type = match extension.as_str() {
            "uasset" => {
                // Try to determine specific asset type from file content or name
                if file_path.contains("BP_") || file_path.contains("Blueprint") {
                    UnrealAssetType::Blueprint
                } else if file_path.contains("SM_") {
                    UnrealAssetType::StaticMesh
                } else if file_path.contains("T_") {
                    UnrealAssetType::Texture
                } else if file_path.contains("M_") || file_path.contains("MI_") {
                    UnrealAssetType::Material
                } else if file_path.contains("DA_") {
                    UnrealAssetType::DataAsset
                } else {
                    UnrealAssetType::Other("Asset".to_string())
                }
            },
            "umap" => UnrealAssetType::Map,
            "wav" | "ogg" => UnrealAssetType::Sound,
            "fbx" | "obj" => UnrealAssetType::StaticMesh,
            "png" | "jpg" | "tga" | "exr" => UnrealAssetType::Texture,
            _ => UnrealAssetType::Other(extension),
        };

        let metadata = fs::metadata(file_path)?;
        let size_mb = metadata.len() as f64 / 1024.0 / 1024.0;

        // Determine merge strategy based on asset type
        let merge_strategy = match asset_type {
            UnrealAssetType::Blueprint => MergeStrategy::Assisted,
            UnrealAssetType::Map => MergeStrategy::Assisted,
            UnrealAssetType::Material => MergeStrategy::Assisted,
            UnrealAssetType::DataAsset => MergeStrategy::Intelligent,
            _ => MergeStrategy::Binary,
        };

        // Check current lock status
        let lock_status = self.locks.get(file_path)
            .cloned()
            .unwrap_or(LockStatus::Unlocked);

        // Analyze dependencies (placeholder - would need UE integration)
        let dependencies = self.analyze_asset_dependencies(file_path);

        Ok(UnrealAssetInfo {
            asset_type,
            dependencies,
            size_mb,
            lock_status,
            merge_strategy,
        })
    }

    fn analyze_asset_dependencies(&self, _file_path: &str) -> Vec<String> {
        // Placeholder for dependency analysis
        // In a real implementation, this would parse .uasset files or use UE tools
        vec![]
    }

    pub fn intelligent_lock(&mut self, file_path: &str, reason: SmartLockReason) -> Result<(), String> {
        if !self.integration.lock_management.intelligent_locking {
            return Err("Intelligent locking is disabled".to_string());
        }

        let current_user = whoami::username();
        
        // Check if already locked by someone else
        if let Some(existing_lock) = self.locks.get(file_path) {
            match existing_lock {
                LockStatus::LockedByUser(user) if user != &current_user => {
                    return Err(format!("File is already locked by {}", user));
                },
                _ => {} // Can override our own locks or smart locks
            }
        }

        self.locks.insert(file_path.to_string(), LockStatus::SmartLocked(reason.clone()));
        
        println!("{} Smart lock applied: {}", "🔒".yellow(), Style::file_path(file_path));
        println!("  Reason: {:?}", reason);
        println!("  User: {}", current_user.cyan());
        
        Ok(())
    }

    pub fn release_lock(&mut self, file_path: &str, force: bool) -> Result<(), String> {
        let current_user = whoami::username();
        
        if let Some(lock_status) = self.locks.get(file_path) {
            match lock_status {
                LockStatus::LockedByUser(user) if user != &current_user && !force => {
                    return Err(format!("Cannot release lock owned by {}", user));
                },
                LockStatus::Unlocked => {
                    return Err("File is not locked".to_string());
                },
                _ => {
                    self.locks.remove(file_path);
                    println!("{} Lock released: {}", "🔓".green(), Style::file_path(file_path));
                }
            }
        } else {
            return Err("File is not locked".to_string());
        }
        
        Ok(())
    }

    pub fn branch_switch_cleanup(&mut self, old_branch: &str, new_branch: &str) -> Result<(), String> {
        if !self.integration.lock_management.auto_unlock_on_branch {
            return Ok(());
        }

        let mut locks_to_release = Vec::new();
        let current_user = whoami::username();

        println!("{} Cleaning up locks for branch switch: {} → {}", 
                "🔄".blue(), old_branch.yellow(), new_branch.green());

        for (file_path, lock_status) in &self.locks {
            let should_release = match lock_status {
                LockStatus::LockedByUser(user) if user == &current_user => {
                    // Release our own locks based on inheritance setting
                    match self.integration.lock_management.lock_inheritance {
                        LockInheritance::None => true,
                        LockInheritance::Smart => {
                            // Smart inheritance: keep locks on files that are likely to be edited
                            !self.is_likely_branch_shared_file(file_path)
                        },
                        LockInheritance::Explicit => false, // Keep locks for explicit management
                    }
                },
                LockStatus::LockedByBranch(branch) if branch == old_branch => true,
                LockStatus::SmartLocked(SmartLockReason::ActiveDevelopment) => true,
                _ => false,
            };

            if should_release {
                locks_to_release.push(file_path.clone());
            }
        }

        for file_path in locks_to_release {
            self.locks.remove(&file_path);
            println!("  🔓 Released lock: {}", Style::file_path(&file_path));
        }

        Ok(())
    }

    pub fn release_cleanup(&mut self, release_tag: &str) -> Result<(), String> {
        if !self.integration.lock_management.release_lock_cleanup {
            return Ok(());
        }

        println!("{} Performing release lock cleanup for: {}", 
                "🚀".green(), release_tag.bold());

        let mut locks_to_release = Vec::new();

        for (file_path, lock_status) in &self.locks {
            let should_release = match lock_status {
                LockStatus::SmartLocked(SmartLockReason::PendingRelease) => true,
                LockStatus::SmartLocked(SmartLockReason::ActiveDevelopment) => {
                    // Release development locks for stable assets
                    self.is_stable_for_release(file_path)
                },
                _ => false,
            };

            if should_release {
                locks_to_release.push(file_path.clone());
            }
        }

        for file_path in locks_to_release {
            self.locks.remove(&file_path);
            println!("  🔓 Released release lock: {}", Style::file_path(&file_path));
        }

        println!("✨ Release cleanup complete!");
        Ok(())
    }

    fn is_likely_branch_shared_file(&self, file_path: &str) -> bool {
        // Heuristics for files that are likely to be shared across branches
        let path_lower = file_path.to_lowercase();
        
        // Core game systems, settings, and shared assets
        path_lower.contains("config/") ||
        path_lower.contains("settings/") ||
        path_lower.contains("shared/") ||
        path_lower.contains("common/") ||
        path_lower.contains("core/") ||
        path_lower.contains("gamemode") ||
        path_lower.contains("playercontroller") ||
        path_lower.contains("gameinstance")
    }

    fn is_stable_for_release(&self, _file_path: &str) -> bool {
        // Placeholder logic for determining if a file is stable for release
        // Could integrate with CI/CD status, testing results, etc.
        true
    }

    pub fn show_lock_status(&self) {
        if self.locks.is_empty() {
            println!("{} No active locks", "🔓".green());
            return;
        }

        println!("{} Active Locks", "🔒".yellow());
        for (file_path, lock_status) in &self.locks {
            let status_str = match lock_status {
                LockStatus::LockedByUser(user) => format!("Locked by {}", user.cyan()),
                LockStatus::LockedByBranch(branch) => format!("Branch lock: {}", branch.yellow()),
                LockStatus::SmartLocked(reason) => format!("Smart lock: {:?}", reason).blue().to_string(),
                LockStatus::Unlocked => "Unlocked".green().to_string(),
            };
            
            println!("  {} - {}", Style::file_path(file_path), status_str);
        }
    }

    pub fn suggest_lfs_candidates(&mut self, repo_path: &str) -> Result<Vec<String>, std::io::Error> {
        let mut candidates = Vec::new();
        
        if let Ok(entries) = fs::read_dir(repo_path) {
            for entry in entries.flatten() {
                if entry.metadata()?.is_file() {
                    let file_path = entry.path().to_string_lossy().to_string();
                    
                    // Check if it's an Unreal asset
                    if let Ok(asset_info) = self.analyze_unreal_asset(&file_path) {
                        // Suggest LFS for large assets or specific types
                        let should_use_lfs = asset_info.size_mb > 10.0 || 
                                           matches!(asset_info.asset_type, 
                                                   UnrealAssetType::StaticMesh | 
                                                   UnrealAssetType::Texture | 
                                                   UnrealAssetType::Sound |
                                                   UnrealAssetType::Animation);
                        
                        if should_use_lfs {
                            candidates.push(file_path);
                        }
                    }
                }
            }
        }
        
        Ok(candidates)
    }
}

// CLI Integration functions
pub fn handle_unreal_command(cmd: UnrealCmd) -> anyhow::Result<()> {
    let mut manager = UnrealManager::new();
    
    match cmd {
        UnrealCmd::Detect => {
            if let Ok(current_dir) = std::env::current_dir() {
                let detected = manager.detect_unreal_project(&current_dir.to_string_lossy())?;
                if !detected {
                    println!("{} No Unreal Engine project detected in current directory", "ℹ️".blue());
                }
            }
        },
        UnrealCmd::Lock { files, reason } => {
            let lock_reason = match reason.as_deref() {
                Some("development") => SmartLockReason::ActiveDevelopment,
                Some("release") => SmartLockReason::PendingRelease,
                Some("conflict") => SmartLockReason::ConflictPrevention,
                Some("dependency") => SmartLockReason::DependencyProtection,
                _ => SmartLockReason::ActiveDevelopment,
            };
            
            for file in files {
                let file_str = file.to_string_lossy();
                match manager.intelligent_lock(&file_str, lock_reason.clone()) {
                    Ok(_) => {},
                    Err(e) => println!("❌ Failed to lock {}: {}", Style::file_path(&file_str), e.red()),
                }
            }
        },
        UnrealCmd::Unlock { files, force } => {
            for file in files {
                let file_str = file.to_string_lossy();
                match manager.release_lock(&file_str, force) {
                    Ok(_) => {},
                    Err(e) => println!("❌ Failed to unlock {}: {}", Style::file_path(&file_str), e.red()),
                }
            }
        },
        UnrealCmd::Status => {
            manager.show_lock_status();
        },
        UnrealCmd::CleanupBranch { old_branch, new_branch } => {
            manager.branch_switch_cleanup(&old_branch, &new_branch)?;
        },
        UnrealCmd::CleanupRelease { release_tag } => {
            manager.release_cleanup(&release_tag)?;
        },
        UnrealCmd::LfsSuggestions => {
            if let Ok(current_dir) = std::env::current_dir() {
                let candidates = manager.suggest_lfs_candidates(&current_dir.to_string_lossy())?;
                
                if candidates.is_empty() {
                    println!("{} No LFS candidates found", "✅".green());
                } else {
                    println!("{} LFS Candidates Found:", "📦".blue());
                    for candidate in candidates {
                        println!("  {}", Style::file_path(&candidate));
                    }
                    println!("\nRun 'git lfs track' for these files to improve performance");
                }
            }
        },
    }
    
    Ok(())
}

#[derive(clap::Subcommand, Debug)]
pub enum UnrealCmd {
    /// Detect Unreal Engine project in current directory
    Detect,
    /// Lock files with intelligent management
    Lock {
        files: Vec<std::path::PathBuf>,
        #[arg(long, help = "Lock reason: development, release, conflict, dependency")]
        reason: Option<String>,
    },
    /// Unlock files
    Unlock {
        files: Vec<std::path::PathBuf>,
        #[arg(long, help = "Force unlock even if owned by another user")]
        force: bool,
    },
    /// Show lock status
    Status,
    /// Cleanup locks when switching branches
    CleanupBranch {
        old_branch: String,
        new_branch: String,
    },
    /// Cleanup locks for release
    CleanupRelease {
        release_tag: String,
    },
    /// Show LFS suggestions for Unreal assets
    LfsSuggestions,
}
