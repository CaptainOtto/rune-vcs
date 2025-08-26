use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use colored::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceConfig {
    pub enabled: bool,
    pub lfs_threshold_mb: u64,
    pub features: IntelligenceFeatures,
    pub analysis_depth: AnalysisDepth,
    pub notification_level: NotificationLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceFeatures {
    pub security_analysis: bool,
    pub performance_insights: bool,
    pub predictive_modeling: bool,
    pub repository_health: bool,
    pub code_quality_assessment: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisDepth {
    Basic,
    Comprehensive,
    Advanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationLevel {
    Silent,
    Essential,
    Detailed,
    Verbose,
}

#[derive(Debug, Clone)]
pub struct FileAnalysis {
    pub suggested_lfs: bool,
    pub file_type: FileType,
    pub language: Language,
    pub size_bytes: u64,
    pub security_issues: Vec<SecurityIssue>,
    pub performance_impact: PerformanceImpact,
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FileType {
    SourceCode(Language),
    Binary,
    Text,
    Media,
    Archive,
    Configuration,
    Documentation,
    Unknown,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Language {
    Rust, Python, JavaScript, TypeScript, Go, C, Cpp, Java, Kotlin, Other(String),
}

#[derive(Debug, Clone)]
pub struct SecurityIssue {
    pub issue_type: String,
    pub severity: String,
    pub description: String,
    pub category: SecurityCategory,
}

#[derive(Debug, Clone)]
pub enum SecurityCategory {
    Credentials, Certificates, Configuration,
}

#[derive(Debug, Clone)]
pub struct PerformanceImpact {
    pub storage_efficiency: f64,
    pub compression_benefit: f64,
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub action: String,
    pub priority: String,
    pub category: SuggestionCategory,
}

#[derive(Debug, Clone)]
pub enum SuggestionCategory {
    Performance, Security, Organization,
}

#[derive(Debug, Clone)]
pub enum ProjectType {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    UnrealEngine,
    Unity,
    Godot,
    NextJS,
    React,
    Vue,
    Angular,
    iOS,
    Android,
    Flutter,
    ReactNative,
    DataScience,
    MachineLearning,
    Design,
    Documentation,
    Other(String),
}

#[derive(Debug, Clone)]
pub struct RepositoryHealth {
    pub overall_score: f64,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PredictiveInsight {
    pub insight: String,
    pub confidence: f64,
    pub category: InsightCategory,
    pub severity: InsightSeverity,
}

#[derive(Debug, Clone)]
pub enum InsightCategory {
    Security, Performance, Documentation, Legal,
}

#[derive(Debug, Clone)]
pub enum InsightSeverity {
    Medium, High,
}

pub struct IntelligentFileAnalyzer {
    pub config: IntelligenceConfig,
    cache: HashMap<String, FileAnalysis>,
}

impl Default for IntelligenceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            lfs_threshold_mb: 50,
            features: IntelligenceFeatures {
                security_analysis: true,
                performance_insights: true,
                predictive_modeling: true,
                repository_health: true,
                code_quality_assessment: true,
            },
            analysis_depth: AnalysisDepth::Comprehensive,
            notification_level: NotificationLevel::Essential,
        }
    }
}

impl Default for FileAnalysis {
    fn default() -> Self {
        Self {
            suggested_lfs: false,
            file_type: FileType::Unknown,
            language: Language::Other("Unknown".to_string()),
            size_bytes: 0,
            security_issues: vec![],
            performance_impact: PerformanceImpact {
                storage_efficiency: 1.0,
                compression_benefit: 0.0,
            },
            suggestions: vec![],
        }
    }
}

impl IntelligentFileAnalyzer {
    pub fn new() -> Self {
        Self {
            config: IntelligenceConfig::default(),
            cache: HashMap::new(),
        }
    }

    pub fn analyze_file(&mut self, file_path: &str) -> Result<FileAnalysis, std::io::Error> {
        if !self.config.enabled {
            return Ok(FileAnalysis::default());
        }

        // Check cache first
        if let Some(cached) = self.cache.get(file_path) {
            return Ok(cached.clone());
        }

        let metadata = fs::metadata(file_path)?;
        let file_size = metadata.len();
        let extension = Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        let file_type = self.detect_file_type(&extension, file_size, file_path);
        let language = self.detect_language(&extension);
        let contents = fs::read_to_string(file_path).unwrap_or_default();
        
        let security_issues = if self.config.features.security_analysis {
            self.analyze_security(file_path, &contents)
        } else {
            vec![]
        };

        let performance_impact = if self.config.features.performance_insights {
            self.analyze_performance_impact(file_size)
        } else {
            PerformanceImpact {
                storage_efficiency: 1.0,
                compression_benefit: 0.0,
            }
        };

        let suggestions = self.generate_suggestions(file_path, file_size, &security_issues);

        let analysis = FileAnalysis {
            suggested_lfs: file_size > self.config.lfs_threshold_mb * 1024 * 1024,
            file_type,
            language,
            size_bytes: file_size,
            security_issues,
            performance_impact,
            suggestions,
        };

        self.cache.insert(file_path.to_string(), analysis.clone());
        Ok(analysis)
    }

    pub fn detect_project_type(&self, repo_path: &str) -> Result<ProjectType, std::io::Error> {
        let path = Path::new(repo_path);
        
        // Check for various project indicators
        if path.join("Cargo.toml").exists() {
            return Ok(ProjectType::Rust);
        }
        if path.join("package.json").exists() {
            let content = fs::read_to_string(path.join("package.json"))?;
            if content.contains("\"next\"") {
                return Ok(ProjectType::NextJS);
            }
            if content.contains("\"react\"") {
                return Ok(ProjectType::React);
            }
            if content.contains("\"vue\"") {
                return Ok(ProjectType::Vue);
            }
            if content.contains("\"@angular\"") {
                return Ok(ProjectType::Angular);
            }
            return Ok(ProjectType::JavaScript);
        }
        if path.join("requirements.txt").exists() || path.join("pyproject.toml").exists() {
            return Ok(ProjectType::Python);
        }
        if path.join("go.mod").exists() {
            return Ok(ProjectType::Go);
        }
        
        // Game engine detection
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_lowercase();
            if file_name.ends_with(".uproject") {
                return Ok(ProjectType::UnrealEngine);
            }
            if file_name.ends_with(".unity") || file_name == "projectsettings" {
                return Ok(ProjectType::Unity);
            }
            if file_name == "project.godot" {
                return Ok(ProjectType::Godot);
            }
        }
        
        // Mobile development
        if path.join("ios").exists() && path.join("android").exists() {
            if path.join("pubspec.yaml").exists() {
                return Ok(ProjectType::Flutter);
            }
            return Ok(ProjectType::ReactNative);
        }
        if path.join("Podfile").exists() || path.join("ios").exists() {
            return Ok(ProjectType::iOS);
        }
        if path.join("build.gradle").exists() || path.join("android").exists() {
            return Ok(ProjectType::Android);
        }

        Ok(ProjectType::Other("Unknown".to_string()))
    }

    pub fn analyze_repository_health(&self, repo_path: &str) -> RepositoryHealth {
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();
        let mut score = 100.0;

        let path = Path::new(repo_path);
        
        // Check for README
        if !path.join("README.md").exists() && !path.join("README.txt").exists() {
            issues.push("No README file found".to_string());
            recommendations.push("Add a README.md file to document your project".to_string());
            score -= 20.0;
        }

        // Check for license
        if !path.join("LICENSE").exists() && !path.join("LICENSE.txt").exists() {
            issues.push("No license file found".to_string());
            recommendations.push("Add a LICENSE file to clarify usage rights".to_string());
            score -= 10.0;
        }

        // Check for gitignore
        if !path.join(".gitignore").exists() {
            issues.push("No .gitignore file found".to_string());
            recommendations.push("Add a .gitignore file to exclude unwanted files".to_string());
            score -= 15.0;
        }

        RepositoryHealth {
            overall_score: score.max(0.0),
            issues,
            recommendations,
        }
    }

    pub fn generate_insights(&self, repo_path: &str) -> Vec<PredictiveInsight> {
        let mut insights = Vec::new();
        let path = Path::new(repo_path);

        // Check for large files
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() && metadata.len() > 100_000_000 { // 100MB
                        insights.push(PredictiveInsight {
                            insight: format!("Large file detected: {} ({}MB)", 
                                entry.file_name().to_string_lossy(),
                                metadata.len() / 1024 / 1024),
                            confidence: 0.95,
                            category: InsightCategory::Performance,
                            severity: InsightSeverity::High,
                        });
                    }
                }
            }
        }

        insights
    }

    pub fn predictive_analysis(&self, repo_path: &str) -> Vec<PredictiveInsight> {
        self.generate_insights(repo_path)
    }

    fn detect_file_type(&self, extension: &str, file_size: u64, _file_path: &str) -> FileType {
        match extension {
            "rs" => FileType::SourceCode(Language::Rust),
            "py" => FileType::SourceCode(Language::Python),
            "js" => FileType::SourceCode(Language::JavaScript),
            "ts" => FileType::SourceCode(Language::TypeScript),
            "go" => FileType::SourceCode(Language::Go),
            "c" => FileType::SourceCode(Language::C),
            "cpp" | "cc" | "cxx" => FileType::SourceCode(Language::Cpp),
            "java" => FileType::SourceCode(Language::Java),
            "kt" => FileType::SourceCode(Language::Kotlin),
            "exe" | "dll" | "so" | "dylib" => FileType::Binary,
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" => FileType::Media,
            "mp4" | "avi" | "mov" | "wmv" => FileType::Media,
            "mp3" | "wav" | "ogg" | "flac" => FileType::Media,
            "zip" | "tar" | "gz" | "rar" | "7z" => FileType::Archive,
            "json" | "yaml" | "yml" | "toml" | "ini" | "cfg" => FileType::Configuration,
            "md" | "txt" | "doc" | "docx" | "pdf" => FileType::Documentation,
            _ => {
                if file_size > 10_000_000 { // Assume large files are binary
                    FileType::Binary
                } else {
                    FileType::Text
                }
            }
        }
    }

    fn detect_language(&self, extension: &str) -> Language {
        match extension {
            "rs" => Language::Rust,
            "py" => Language::Python,
            "js" => Language::JavaScript,
            "ts" => Language::TypeScript,
            "go" => Language::Go,
            "c" => Language::C,
            "cpp" | "cc" | "cxx" => Language::Cpp,
            "java" => Language::Java,
            "kt" => Language::Kotlin,
            _ => Language::Other(extension.to_string()),
        }
    }

    fn analyze_security(&self, file_path: &str, contents: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();
        let lower_path = file_path.to_lowercase();
        let lower_contents = contents.to_lowercase();

        // Check for credential files
        if lower_path.contains(".env") || lower_path.contains("secret") || lower_path.contains("password") {
            issues.push(SecurityIssue {
                issue_type: "Potential credential file".to_string(),
                severity: "High".to_string(),
                description: "File may contain sensitive credentials".to_string(),
                category: SecurityCategory::Credentials,
            });
        }

        // Check for hardcoded secrets in content
        if lower_contents.contains("password") || lower_contents.contains("api_key") || 
           lower_contents.contains("secret") || lower_contents.contains("token") {
            issues.push(SecurityIssue {
                issue_type: "Potential hardcoded secret".to_string(),
                severity: "High".to_string(),
                description: "Content may contain hardcoded secrets".to_string(),
                category: SecurityCategory::Credentials,
            });
        }

        issues
    }

    fn analyze_performance_impact(&self, file_size: u64) -> PerformanceImpact {
        let storage_efficiency = if file_size > 100_000_000 { // 100MB
            0.3
        } else if file_size > 10_000_000 { // 10MB
            0.7
        } else {
            1.0
        };

        PerformanceImpact {
            storage_efficiency,
            compression_benefit: if file_size > 1_000_000 { 0.6 } else { 0.1 },
        }
    }

    fn generate_suggestions(&self, file_path: &str, file_size: u64, security_issues: &[SecurityIssue]) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        if file_size > 50_000_000 { // 50MB
            suggestions.push(Suggestion {
                action: "Consider using Git LFS for this large file".to_string(),
                priority: "Medium".to_string(),
                category: SuggestionCategory::Performance,
            });
        }

        if !security_issues.is_empty() {
            suggestions.push(Suggestion {
                action: "Review and secure sensitive content".to_string(),
                priority: "High".to_string(),
                category: SuggestionCategory::Security,
            });
        }

        let extension = Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        if matches!(extension, "tmp" | "log" | "cache") {
            suggestions.push(Suggestion {
                action: "Consider adding to .gitignore".to_string(),
                priority: "Low".to_string(),
                category: SuggestionCategory::Organization,
            });
        }

        suggestions
    }

    pub fn display_analysis(&self, analysis: &FileAnalysis) {
        println!("📊 {}", "File Analysis".cyan().bold());
        println!("  Type: {:?}", analysis.file_type);
        println!("  Language: {:?}", analysis.language);
        println!("  Size: {} bytes", analysis.size_bytes);
        println!("  LFS Suggested: {}", if analysis.suggested_lfs { "Yes".green() } else { "No".red() });
        
        if !analysis.security_issues.is_empty() {
            println!("  Security: {} issues found", analysis.security_issues.len());
            for issue in &analysis.security_issues {
                println!("    - {}: {}", issue.severity.red(), issue.description);
            }
        }

        if !analysis.suggestions.is_empty() {
            println!("  Suggestions:");
            for suggestion in &analysis.suggestions {
                println!("    - [{}] {}", suggestion.priority.yellow(), suggestion.action);
            }
        }
    }
}
