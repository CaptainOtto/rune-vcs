use crate::intelligence::{IntelligentFileAnalyzer, ProjectType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockingConfig {
    pub enabled: bool,
    pub auto_lock_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub lock_timeout_hours: u64,
    pub require_message: bool,
    pub allow_force_unlock: bool,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub file_type: DetectedFileType,
}

#[derive(Debug, Clone)]
pub enum DetectedFileType {
    SourceCode(String),  // Language
    GameAsset(String),   // Asset type (3D Model, Texture, etc.)
    Document(String),    // Document type
    BinaryAsset(String), // Type description
    Media(MediaType),
    Configuration,
    Dependency,
    Documentation,
    DesignAsset,
    DataFile,
    Other,
}

#[derive(Debug, Clone)]
pub enum MediaType {
    Image,
    Video,
    Audio,
    Model3D,
    Font,
}

#[derive(Debug, Clone)]
pub enum LockStatus {
    Unlocked,
}

pub struct LockManager {
    config: LockingConfig,
}

impl Default for LockingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_lock_patterns: vec![
                "*.uasset".to_string(),
                "*.umap".to_string(),
                "*.blend".to_string(),
                "*.fbx".to_string(),
                "*.psd".to_string(),
                "*.ai".to_string(),
            ],
            exclude_patterns: vec![
                "*.tmp".to_string(),
                "*.log".to_string(),
                "*.cache".to_string(),
                "node_modules/**".to_string(),
                "target/**".to_string(),
                ".git/**".to_string(),
            ],
            lock_timeout_hours: 24,
            require_message: false,
            allow_force_unlock: true,
        }
    }
}

impl LockManager {
    pub fn new(config: LockingConfig) -> Self {
        Self { config }
    }

    pub fn detect_project_type(&self, repo_path: &str) -> Result<ProjectType, std::io::Error> {
        let analyzer = IntelligentFileAnalyzer::new();
        analyzer.detect_project_type(repo_path)
    }

    pub fn analyze_files(&self, file_paths: &[String]) -> HashMap<String, FileInfo> {
        let mut results = HashMap::new();

        for path in file_paths {
            let file_type = self.detect_file_type(path);

            results.insert(path.clone(), FileInfo { file_type });
        }

        results
    }

    pub fn should_lock(&self, file_path: &str) -> bool {
        if !self.config.enabled {
            return false;
        }

        let path = Path::new(file_path);
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        // Check exclude patterns first
        for pattern in &self.config.exclude_patterns {
            if self.matches_pattern(file_path, pattern) {
                return false;
            }
        }

        // Check auto-lock patterns
        for pattern in &self.config.auto_lock_patterns {
            if self.matches_pattern(file_path, pattern) {
                return true;
            }
        }

        // Auto-lock for source code and important assets
        let file_type = self.detect_file_type(file_path);
        matches!(
            file_type,
            DetectedFileType::SourceCode(_)
                | DetectedFileType::GameAsset(_)
                | DetectedFileType::DesignAsset
        )
    }

    pub fn get_lfs_suggestions(&self, repo_path: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        if let Ok(entries) = fs::read_dir(repo_path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() && metadata.len() > 100_000_000 {
                        // 100MB
                        if let Some(path_str) = entry.path().to_str() {
                            suggestions.push(path_str.to_string());
                        }
                    }
                }
            }
        }

        suggestions
    }

    pub fn get_lock_status(
        &self,
        _repo_path: &str,
        file_paths: &[String],
    ) -> HashMap<String, LockStatus> {
        let mut status = HashMap::new();

        for path in file_paths {
            status.insert(path.clone(), LockStatus::Unlocked);
        }

        status
    }

    fn detect_file_type(&self, file_path: &str) -> DetectedFileType {
        let path = Path::new(file_path);
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            // Source code
            "rs" => DetectedFileType::SourceCode("Rust".to_string()),
            "py" => DetectedFileType::SourceCode("Python".to_string()),
            "js" | "ts" => DetectedFileType::SourceCode("JavaScript/TypeScript".to_string()),
            "cpp" | "cc" | "cxx" | "c" => DetectedFileType::SourceCode("C/C++".to_string()),
            "java" => DetectedFileType::SourceCode("Java".to_string()),
            "go" => DetectedFileType::SourceCode("Go".to_string()),
            "swift" => DetectedFileType::SourceCode("Swift".to_string()),
            "kt" => DetectedFileType::SourceCode("Kotlin".to_string()),
            "cs" => DetectedFileType::SourceCode("C#".to_string()),

            // Game assets
            "uasset" | "umap" => DetectedFileType::GameAsset("Unreal Engine Asset".to_string()),
            "prefab" | "unity" => DetectedFileType::GameAsset("Unity Asset".to_string()),
            "tscn" | "tres" => DetectedFileType::GameAsset("Godot Scene/Resource".to_string()),
            "blend" => DetectedFileType::GameAsset("Blender File".to_string()),
            "fbx" | "obj" | "dae" | "3ds" => DetectedFileType::GameAsset("3D Model".to_string()),

            // Design assets
            "psd" | "psb" => DetectedFileType::DesignAsset,
            "ai" | "eps" => DetectedFileType::DesignAsset,
            "sketch" => DetectedFileType::DesignAsset,
            "fig" | "figma" => DetectedFileType::DesignAsset,
            "xd" => DetectedFileType::DesignAsset,

            // Media
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tiff" | "webp" => {
                DetectedFileType::Media(MediaType::Image)
            }
            "mp4" | "avi" | "mov" | "wmv" | "mkv" | "webm" => {
                DetectedFileType::Media(MediaType::Video)
            }
            "mp3" | "wav" | "ogg" | "flac" | "aac" => DetectedFileType::Media(MediaType::Audio),
            "ttf" | "otf" | "woff" | "woff2" => DetectedFileType::Media(MediaType::Font),

            // Documents
            "md" | "txt" => DetectedFileType::Document("Text".to_string()),
            "pdf" => DetectedFileType::Document("PDF".to_string()),
            "doc" | "docx" => DetectedFileType::Document("Word Document".to_string()),
            "ppt" | "pptx" => DetectedFileType::Document("PowerPoint".to_string()),
            "xls" | "xlsx" => DetectedFileType::Document("Excel".to_string()),

            // Configuration
            "json" | "yaml" | "yml" | "toml" | "ini" | "cfg" | "conf" => {
                DetectedFileType::Configuration
            }

            // Data files
            "csv" | "tsv" | "xml" | "sql" => DetectedFileType::DataFile,
            "db" | "sqlite" | "sqlite3" => DetectedFileType::DataFile,

            // Binary assets
            "exe" | "dll" | "so" | "dylib" | "lib" | "a" => {
                DetectedFileType::BinaryAsset("Executable/Library".to_string())
            }
            "zip" | "tar" | "gz" | "rar" | "7z" => {
                DetectedFileType::BinaryAsset("Archive".to_string())
            }

            _ => DetectedFileType::Other,
        }
    }

    fn matches_pattern(&self, file_path: &str, pattern: &str) -> bool {
        // Simple pattern matching - could be enhanced with proper glob matching
        if pattern.contains('*') {
            let pattern_without_star = pattern.replace('*', "");
            file_path.contains(&pattern_without_star)
        } else {
            file_path.ends_with(pattern)
        }
    }

    pub fn print_project_analysis(&self, repo_path: &str) {
        println!("🔍 Analyzing project structure...");

        match self.detect_project_type(repo_path) {
            Ok(project_type) => {
                println!("📋 Project Type: {:?}", project_type);

                match project_type {
                    ProjectType::UnrealEngine => {
                        println!("🎮 Unreal Engine project detected");
                        println!("   - Recommended: Lock .uasset and .umap files");
                        println!("   - Consider: LFS for large assets (>100MB)");
                    }
                    ProjectType::Unity => {
                        println!("🎮 Unity project detected");
                        println!("   - Recommended: Lock .prefab and scene files");
                        println!("   - Consider: LFS for large textures and models");
                    }
                    ProjectType::NextJS | ProjectType::React => {
                        println!("🌐 Web project detected");
                        println!("   - Recommended: Lock component files during development");
                        println!("   - Consider: LFS for large media assets");
                    }
                    ProjectType::Rust => {
                        println!("🦀 Rust project detected");
                        println!("   - Recommended: Lock .rs files during active development");
                        println!("   - Exclude: target/ directory");
                    }
                    _ => {
                        println!("📁 Generic project type");
                        println!("   - Default locking strategies will be applied");
                    }
                }
            }
            Err(e) => {
                println!("⚠️  Could not detect project type: {}", e);
            }
        }
    }
}
