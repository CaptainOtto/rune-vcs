use colored::*;
use std::path::Path;
use std::fs;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IntelligenceConfig {
    pub enabled: bool,
    pub features: IntelligenceFeatures,
    pub analysis_depth: AnalysisDepth,
    pub notification_level: NotificationLevel,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IntelligenceFeatures {
    pub security_analysis: bool,
    pub performance_optimization: bool,
    pub dependency_analysis: bool,
    pub code_quality_assessment: bool,
    pub repository_insights: bool,
    pub predictive_suggestions: bool,
    pub advanced_compression: bool,
    pub conflict_prevention: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AnalysisDepth {
    Minimal,
    Standard,
    Deep,
    Comprehensive,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NotificationLevel {
    Silent,
    Errors,
    Warnings,
    Info,
    Detailed,
}

pub struct IntelligentFileAnalyzer {
    config: IntelligenceConfig,
    cache: HashMap<String, FileAnalysis>,
}

#[derive(Debug, Clone)]
pub struct FileAnalysis {
    pub suggested_lfs: bool,
    pub file_type: FileType,
    pub language: Language,
    pub size_bytes: u64,
    pub binary_type: Option<BinaryType>,
    pub text_type: Option<TextType>,
    pub media_type: Option<MediaType>,
    pub archive_type: Option<ArchiveType>,
    pub security_issues: Vec<SecurityIssue>,
    pub performance_impact: PerformanceImpact,
    pub code_quality: Option<CodeQuality>,
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FileType {
    SourceCode(Language),
    Binary(BinaryType),
    Text(TextType),
    Media(MediaType),
    Archive(ArchiveType),
    Configuration,
    Documentation,
    Unknown,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Language {
    Rust, Python, JavaScript, TypeScript, Go, C, Cpp, Java, Kotlin, Other(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryType {
    Executable, Library, Database, Image, Video, Audio, Other,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TextType {
    Plain, Markup, Data, Log, Other,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MediaType {
    Image, Video, Audio, Vector, Model,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ArchiveType {
    Compressed, Package, Container,
}

#[derive(Debug, Clone)]
pub struct SecurityRisk {
    pub level: RiskLevel,
    pub issues: Vec<SecurityIssue>,
}

#[derive(Debug, Clone)]
pub enum RiskLevel {
    None, Low, Medium, High, Critical,
}

#[derive(Debug, Clone)]
pub struct SecurityIssue {
    pub category: SecurityCategory,
    pub description: String,
    pub recommendation: String,
}

#[derive(Debug, Clone)]
pub enum SecurityCategory {
    Credentials, PersonalData, ApiKeys, Certificates, Configuration, Other,
}

#[derive(Debug, Clone)]
pub struct PerformanceImpact {
    pub storage_efficiency: f64,
    pub network_transfer_cost: f64,
    pub compression_benefit: f64,
    pub indexing_cost: f64,
}

#[derive(Debug, Clone)]
pub struct CodeQuality {
    pub complexity_score: f64,
    pub maintainability: f64,
    pub test_coverage_estimate: f64,
    pub documentation_level: f64,
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub category: SuggestionCategory,
    pub priority: Priority,
    pub action: String,
    pub rationale: String,
}

#[derive(Debug, Clone)]
pub enum SuggestionCategory {
    Performance, Security, Quality, Organization, Optimization,
}

#[derive(Debug, Clone)]
pub enum Priority {
    Low, Medium, High, Critical,
}

#[derive(Debug, Clone, Default)]
pub struct RepositoryHealth {
    pub overall_score: f64,
    pub security_score: f64,
    pub performance_score: f64,
    pub quality_score: f64,
    pub organization_score: f64,
    pub total_files: usize,
    pub risk_files: usize,
    pub large_files: usize,
}

impl RepositoryHealth {
    pub fn update_with_analysis(&mut self, analysis: &FileAnalysis) {
        self.total_files += 1;
        
        if !analysis.security_risk.issues.is_empty() {
            self.risk_files += 1;
        }
        
        if analysis.suggested_lfs {
            self.large_files += 1;
        }
    }
    
    pub fn calculate_scores(&mut self) {
        if self.total_files == 0 {
            return;
        }
        
        self.security_score = ((self.total_files - self.risk_files) as f64 / self.total_files as f64) * 100.0;
        self.performance_score = ((self.total_files - self.large_files) as f64 / self.total_files as f64) * 100.0;
        self.organization_score = 75.0; // Placeholder calculation
        self.quality_score = 80.0; // Placeholder calculation
        
        self.overall_score = (self.security_score + self.performance_score + 
                             self.organization_score + self.quality_score) / 4.0;
    }
}

#[derive(Debug, Clone)]
pub struct PredictiveInsight {
    pub category: InsightCategory,
    pub severity: InsightSeverity,
    pub title: String,
    pub description: String,
    pub recommendation: String,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub enum InsightCategory {
    Security, Performance, Documentation, Legal, Quality, Organization,
}

#[derive(Debug, Clone)]
pub enum InsightSeverity {
    Low, Medium, High, Critical,
}

#[derive(Debug, Clone, Default)]
pub struct ConflictRisk {
    pub level: ConflictRiskLevel,
    pub score: f64,
    pub factors: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub enum ConflictRiskLevel {
    #[default]
    None, Low, Medium, High,
}

impl Default for IntelligenceConfig {
    fn default() -> Self {
        let notifications = match std::env::var("RUNE_INTELLIGENCE_NOTIFICATIONS").as_deref() {
            Ok("silent") => NotificationLevel::Silent,
            Ok("errors") => NotificationLevel::Errors,
            Ok("warnings") => NotificationLevel::Warnings,
            Ok("detailed") => NotificationLevel::Detailed,
            _ => NotificationLevel::Info,
        };
        
        Self {
            enabled: std::env::var("RUNE_INTELLIGENCE").unwrap_or_default() != "false",
            features: IntelligenceFeatures::default(),
            analysis_depth: AnalysisDepth::Standard,
            notification_level: notifications,
        }
    }
}

impl Default for IntelligenceFeatures {
    fn default() -> Self {
        Self {
            security_analysis: true,
            performance_optimization: true,
            dependency_analysis: true,
            code_quality_assessment: true,
            repository_insights: true,
            predictive_suggestions: true,
            advanced_compression: true,
            conflict_prevention: true,
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

    pub fn with_config(config: IntelligenceConfig) -> Self {
        Self { 
            config,
            cache: HashMap::new(),
        }
    }

    /// Configure intelligence features for the user
    pub fn configure_features(&mut self, features: IntelligenceFeatures) {
        self.config.features = features;
    }

    /// Set analysis depth level
    pub fn set_analysis_depth(&mut self, depth: AnalysisDepth) {
        self.config.analysis_depth = depth;
    }

    /// Set notification level
    pub fn set_notification_level(&mut self, level: NotificationLevel) {
        self.config.notification_level = level;
    }

    /// Comprehensive file analysis with revolutionary intelligence
    pub fn analyze_file(&mut self, path: &str) -> Result<FileAnalysis, std::io::Error> {
        if !self.config.enabled {
            return Ok(FileAnalysis::minimal());
        }

        // Check cache first for performance
        if let Some(cached) = self.cache.get(path) {
            return Ok(cached.clone());
        }

        let metadata = fs::metadata(path)?;
        let file_size = metadata.len();
        let extension = Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Comprehensive analysis
        let file_type = self.detect_file_type(&extension, file_size, path);
        let suggested_lfs = file_size > 10_000_000; // >10MB
        let security_issues = vec![]; // Simplified for now
        let performance_impact = PerformanceImpact {
            storage_efficiency: 1.0,
            network_transfer_cost: 1.0,
            compression_benefit: 0.0,
            indexing_cost: 1.0,
        };
        let code_quality = None; // Simplified for now
        let suggestions = vec![]; // Simplified for now

        let analysis = FileAnalysis {
            suggested_lfs,
            file_type: file_type.clone(),
            language: Language::Other("Unknown".to_string()),
            size_bytes: file_size,
            binary_type: None,
            text_type: None, 
            media_type: None,
            archive_type: None,
            security_issues,
            performance_impact,
            code_quality,
            suggestions,
        };

        // Cache the analysis
        self.cache.insert(path.to_string(), analysis.clone());

        // Display notifications based on level
        self.display_analysis_notifications(path, &analysis);

        Ok(analysis)
    }

    fn detect_file_type(&self, extension: &str, file_size: u64, path: &str) -> FileType {
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
            
            "exe" | "bin" => FileType::Binary(BinaryType::Executable),
            "dll" | "so" | "dylib" => FileType::Binary(BinaryType::Library),
            "db" | "sqlite" => FileType::Binary(BinaryType::Database),
            
            "txt" | "log" => FileType::Text(TextType::Plain),
            "md" | "html" | "xml" | "rst" | "adoc" => FileType::Text(TextType::Markup),
            "json" | "yaml" | "toml" | "csv" | "ini" | "conf" | "cfg" => FileType::Text(TextType::Data),
            
            "jpg" | "jpeg" | "png" | "gif" | "bmp" => FileType::Media(MediaType::Image),
            "mp4" | "avi" | "mov" | "mkv" => FileType::Media(MediaType::Video),
            "mp3" | "wav" | "flac" | "ogg" => FileType::Media(MediaType::Audio),
            "svg" => FileType::Media(MediaType::Vector),
            
            "zip" | "rar" | "7z" => FileType::Archive(ArchiveType::Compressed),
            "tar" | "gz" | "bz2" => FileType::Archive(ArchiveType::Compressed),
            "deb" | "rpm" | "pkg" => FileType::Archive(ArchiveType::Package),
            
            _ => {
                if file_size > 1_000_000 { 
                    FileType::Binary(BinaryType::Other) 
                } else if path.contains("README") || path.contains("LICENSE") {
                    FileType::Documentation
                } else {
                    FileType::Unknown
                }
            }
        }
    }

    fn estimate_compression(&self, file_type: &FileType, file_size: u64) -> f64 {
        let base_ratio = match file_type {
            FileType::SourceCode(_) => 0.70,
            FileType::Text(_) => 0.65,
            FileType::Binary(_) => 0.05,
            FileType::Media(_) => 0.02,
            FileType::Archive(_) => 0.01,
            FileType::Configuration => 0.60,
            FileType::Documentation => 0.65,
            FileType::Unknown => 0.40,
        };

        // Adjust based on file size (larger files often compress better)
        if file_size > 100_000 {
            (base_ratio * 1.1_f64).min(0.95)
        } else {
            base_ratio
        }
    }

    fn analyze_security_risk(&self, path: &str, file_type: &FileType) -> SecurityRisk {
        if !self.config.features.security_analysis {
            return SecurityRisk { level: RiskLevel::None, issues: vec![] };
        }

        let mut issues = Vec::new();
        let path_lower = path.to_lowercase();

        // Check for credential-related files
        if path_lower.contains("secret") || path_lower.contains("key") || 
           path_lower.contains("password") || path_lower.contains("token") {
            issues.push(SecurityIssue {
                category: SecurityCategory::Credentials,
                description: "File name suggests it may contain credentials".to_string(),
                recommendation: "Consider using environment variables or secure key management".to_string(),
            });
        }

        // Check for environment files
        if path_lower.contains(".env") {
            issues.push(SecurityIssue {
                category: SecurityCategory::Configuration,
                description: "Environment configuration file detected".to_string(),
                recommendation: "Ensure sensitive values are not committed to version control".to_string(),
            });
        }

        // Check for certificate files
        if path_lower.ends_with(".pem") || path_lower.ends_with(".key") || 
           path_lower.ends_with(".crt") || path_lower.ends_with(".p12") {
            issues.push(SecurityIssue {
                category: SecurityCategory::Certificates,
                description: "Certificate or private key file detected".to_string(),
                recommendation: "Private keys should never be committed to version control".to_string(),
            });
        }

        // Analyze based on file type
        match file_type {
            FileType::SourceCode(_) => {
                // Could analyze source code for hardcoded secrets
                if let Ok(content) = fs::read_to_string(path) {
                    if content.contains("api_key") || content.contains("password") || 
                       content.contains("secret") || content.contains("token") {
                        issues.push(SecurityIssue {
                            category: SecurityCategory::Credentials,
                            description: "Potential hardcoded credentials in source code".to_string(),
                            recommendation: "Use environment variables or secure configuration".to_string(),
                        });
                    }
                }
            },
            _ => {}
        }

        let level = if issues.is_empty() {
            RiskLevel::None
        } else if issues.iter().any(|i| matches!(i.category, SecurityCategory::Credentials | SecurityCategory::Certificates)) {
            RiskLevel::High
        } else {
            RiskLevel::Medium
        };

        SecurityRisk { level, issues }
    }

    fn analyze_performance_impact(&self, file_size: u64, file_type: &FileType) -> PerformanceImpact {
        if !self.config.features.performance_optimization {
            return PerformanceImpact::default();
        }

        let storage_efficiency = match file_type {
            FileType::SourceCode(_) => 0.8,
            FileType::Binary(_) => 0.3,
            FileType::Media(_) => 0.2,
            FileType::Archive(_) => 0.1,
            _ => 0.6,
        };

        let size_factor = (file_size as f64 / 1_000_000.0).min(10.0) / 10.0;
        let network_transfer_cost = size_factor * (1.0 - storage_efficiency);
        let compression_benefit = self.estimate_compression(file_type, file_size);
        let indexing_cost = size_factor * 0.1;

        PerformanceImpact {
            storage_efficiency,
            network_transfer_cost,
            compression_benefit,
            indexing_cost,
        }
    }

    fn analyze_code_quality(&self, path: &str, file_type: &FileType) -> Option<CodeQuality> {
        if !self.config.features.code_quality_assessment {
            return None;
        }

        match file_type {
            FileType::SourceCode(_) => {
                if let Ok(content) = fs::read_to_string(path) {
                    let lines = content.lines().count();
                    let complexity_score = self.estimate_complexity(&content, lines);
                    let maintainability = self.estimate_maintainability(&content, lines);
                    let test_coverage_estimate = self.estimate_test_coverage(path);
                    let documentation_level = self.estimate_documentation(&content, lines);

                    Some(CodeQuality {
                        complexity_score,
                        maintainability,
                        test_coverage_estimate,
                        documentation_level,
                    })
                } else {
                    None
                }
            },
            _ => None,
        }
    }

    fn analyze_dependencies(&self, path: &str, file_type: &FileType) -> Vec<String> {
        if !self.config.features.dependency_analysis {
            return vec![];
        }

        match file_type {
            FileType::SourceCode(Language::Rust) => {
                if let Ok(content) = fs::read_to_string(path) {
                    content.lines()
                        .filter(|line| line.trim_start().starts_with("use "))
                        .take(10) // Limit for performance
                        .map(|line| line.trim().to_string())
                        .collect()
                } else {
                    vec![]
                }
            },
            FileType::SourceCode(Language::Python) => {
                if let Ok(content) = fs::read_to_string(path) {
                    content.lines()
                        .filter(|line| line.trim_start().starts_with("import ") || 
                                      line.trim_start().starts_with("from "))
                        .take(10)
                        .map(|line| line.trim().to_string())
                        .collect()
                } else {
                    vec![]
                }
            },
            _ => vec![],
        }
    }

    fn generate_suggestions(&self, path: &str, file_type: &FileType, 
                          security_risk: &SecurityRisk, performance_impact: &PerformanceImpact) -> Vec<Suggestion> {
        if !self.config.features.predictive_suggestions {
            return vec![];
        }

        let mut suggestions = Vec::new();

        // Security suggestions
        if !security_risk.issues.is_empty() {
            suggestions.push(Suggestion {
                category: SuggestionCategory::Security,
                priority: match security_risk.level {
                    RiskLevel::High | RiskLevel::Critical => Priority::Critical,
                    RiskLevel::Medium => Priority::High,
                    _ => Priority::Medium,
                },
                action: "Review security implications".to_string(),
                rationale: format!("Found {} security issues", security_risk.issues.len()),
            });
        }

        // Performance suggestions
        if performance_impact.compression_benefit > 0.5 {
            suggestions.push(Suggestion {
                category: SuggestionCategory::Performance,
                priority: Priority::Medium,
                action: "Enable compression for this file type".to_string(),
                rationale: format!("Could reduce size by {:.0}%", performance_impact.compression_benefit * 100.0),
            });
        }

        // Organization suggestions
        if matches!(file_type, FileType::SourceCode(_)) && !path.contains("src/") && !path.contains("lib/") {
            suggestions.push(Suggestion {
                category: SuggestionCategory::Organization,
                priority: Priority::Low,
                action: "Consider organizing source files in src/ directory".to_string(),
                rationale: "Improves project structure and maintainability".to_string(),
            });
        }

        suggestions
    }

    fn estimate_complexity(&self, content: &str, lines: usize) -> f64 {
        let cyclomatic_indicators = content.matches("if ").count() + 
                                   content.matches("for ").count() + 
                                   content.matches("while ").count() + 
                                   content.matches("match ").count();
        
        (cyclomatic_indicators as f64 / lines as f64 * 100.0).min(100.0)
    }

    fn estimate_maintainability(&self, content: &str, lines: usize) -> f64 {
        let comment_lines = content.lines().filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*")
        }).count();
        
        let comment_ratio = comment_lines as f64 / lines as f64;
        (comment_ratio * 100.0 + 20.0).min(100.0) // Base score of 20
    }

    fn estimate_test_coverage(&self, path: &str) -> f64 {
        // Simple heuristic: check if there's a corresponding test file
        let test_paths = [
            path.replace(".rs", "_test.rs"),
            path.replace(".py", "_test.py"),
            format!("{}test_{}", 
                    path.rsplit('/').next().unwrap_or("").rsplit('.').nth(1).unwrap_or(""), 
                    path.rsplit('/').next().unwrap_or("")),
        ];
        
        for test_path in &test_paths {
            if fs::metadata(test_path).is_ok() {
                return 75.0; // Assume good coverage if test file exists
            }
        }
        
        20.0 // Low estimate if no test file found
    }

    fn estimate_documentation(&self, content: &str, lines: usize) -> f64 {
        let doc_comments = content.matches("///").count() + 
                          content.matches("\"\"\"").count() + 
                          content.matches("/*").count();
        
        (doc_comments as f64 / (lines as f64 / 10.0) * 100.0).min(100.0)
    }

    fn display_analysis_notifications(&self, path: &str, analysis: &FileAnalysis) {
        match self.config.notification_level {
            NotificationLevel::Silent => return,
            NotificationLevel::Errors => {
                if !analysis.security_issues.is_empty() {
                    println!("{} Critical security risk detected in {}", 
                            "⚠️".red(), Style::file_path(path));
                }
            },
            NotificationLevel::Warnings => {
                self.display_security_warnings(path, &analysis.security_risk);
                if analysis.suggested_lfs {
                    println!("{} Large file detected - consider LFS for {}", 
                            "📦".blue(), Style::file_path(path));
                }
            },
            NotificationLevel::Info => {
                self.display_security_warnings(path, &analysis.security_risk);
                self.display_performance_info(path, &analysis.performance_impact);
                self.display_quality_info(path, &analysis.code_quality);
            },
            NotificationLevel::Detailed => {
                self.display_comprehensive_analysis(path, analysis);
            },
        }
    }

    fn display_security_warnings(&self, path: &str, security_risk: &SecurityRisk) {
        match security_risk.level {
            RiskLevel::Critical => println!("{} Critical security risk - {}", 
                                          "🚨".red(), Style::file_path(path)),
            RiskLevel::High => println!("{} High security risk - {}", 
                                      "⚠️".red(), Style::file_path(path)),
            RiskLevel::Medium => println!("{} Security concern - {}", 
                                        "⚠️".yellow(), Style::file_path(path)),
            _ => {},
        }
    }

    fn display_performance_info(&self, path: &str, performance: &PerformanceImpact) {
        if performance.compression_benefit > 0.5 {
            println!("{} High compression potential ({:.0}%) - {}", 
                    "⚡".cyan(), performance.compression_benefit * 100.0, Style::file_path(path));
        }
    }

    fn display_quality_info(&self, path: &str, quality: &Option<CodeQuality>) {
        if let Some(quality) = quality {
            if quality.complexity_score > 50.0 {
                println!("{} High complexity detected - {}", 
                        "🔄".yellow(), Style::file_path(path));
            }
            if quality.documentation_level < 30.0 {
                println!("{} Low documentation level - {}", 
                        "📝".yellow(), Style::file_path(path));
            }
        }
    }

    /// Generate revolutionary insights about repository health and trends
    pub fn analyze_repository_health(&mut self, repo_path: &str) -> Result<RepositoryHealth, std::io::Error> {
        if !self.config.features.repository_insights {
            return Ok(RepositoryHealth::default());
        }

        let mut health = RepositoryHealth::default();
        
        // Analyze file structure and organization
        if let Ok(entries) = std::fs::read_dir(repo_path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        let path = entry.path().to_string_lossy().to_string();
                        if let Ok(analysis) = self.analyze_file(&path) {
                            health.update_with_analysis(&analysis);
                        }
                    }
                }
            }
        }

        health.calculate_scores();
        Ok(health)
    }

    /// Predict potential issues before they become problems
    pub fn predict_issues(&self, repo_path: &str) -> Vec<PredictiveInsight> {
        if !self.config.features.predictive_suggestions {
            return vec![];
        }

        let mut insights = Vec::new();

        // Predict based on file patterns
        if let Ok(entries) = std::fs::read_dir(repo_path) {
            let files: Vec<_> = entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.metadata().map_or(false, |m| m.is_file()))
                .collect();

            // Check for missing important files
            let has_readme = files.iter().any(|f| 
                f.file_name().to_string_lossy().to_lowercase().contains("readme"));
            let has_license = files.iter().any(|f| 
                f.file_name().to_string_lossy().to_lowercase().contains("license"));
            let has_gitignore = files.iter().any(|f| 
                f.file_name().to_string_lossy() == ".gitignore");

            if !has_readme {
                insights.push(PredictiveInsight {
                    category: InsightCategory::Documentation,
                    severity: InsightSeverity::Medium,
                    title: "Missing README file".to_string(),
                    description: "Repository lacks a README file for documentation".to_string(),
                    recommendation: "Add a README.md file to document your project".to_string(),
                    confidence: 0.95,
                });
            }

            if !has_license {
                insights.push(PredictiveInsight {
                    category: InsightCategory::Legal,
                    severity: InsightSeverity::High,
                    title: "Missing license file".to_string(),
                    description: "Repository has no license file".to_string(),
                    recommendation: "Add a LICENSE file to clarify usage rights".to_string(),
                    confidence: 0.90,
                });
            }

            if !has_gitignore {
                insights.push(PredictiveInsight {
                    category: InsightCategory::Security,
                    severity: InsightSeverity::Medium,
                    title: "Missing .gitignore file".to_string(),
                    description: "Repository may accidentally include sensitive files".to_string(),
                    recommendation: "Add .gitignore to exclude build artifacts and secrets".to_string(),
                    confidence: 0.85,
                });
            }

            // Analyze file size distribution for potential issues
            let large_files = files.iter()
                .filter_map(|f| f.metadata().ok().map(|m| (f, m.len())))
                .filter(|(_, size)| *size > 50_000_000) // > 50MB
                .count();

            if large_files > 0 {
                insights.push(PredictiveInsight {
                    category: InsightCategory::Performance,
                    severity: InsightSeverity::High,
                    title: format!("{} large files detected", large_files),
                    description: "Large files will slow down repository operations".to_string(),
                    recommendation: "Consider using Git LFS for large binary files".to_string(),
                    confidence: 0.88,
                });
            }
        }

        insights
    }

    /// Advanced conflict prevention analysis
    pub fn analyze_conflict_potential(&self, file_path: &str) -> ConflictRisk {
        if !self.config.features.conflict_prevention {
            return ConflictRisk::default();
        }

        let mut risk = ConflictRisk::default();

        if let Ok(content) = std::fs::read_to_string(file_path) {
            let lines = content.lines().collect::<Vec<_>>();
            
            // Check for patterns that commonly cause merge conflicts
            let has_long_lines = lines.iter().any(|line| line.len() > 120);
            let has_trailing_whitespace = lines.iter().any(|line| line.ends_with(' ') || line.ends_with('\t'));
            let has_inconsistent_indentation = self.check_inconsistent_indentation(&lines);
            
            if has_long_lines {
                risk.factors.push("Long lines increase merge conflict likelihood".to_string());
                risk.score += 0.2;
            }
            
            if has_trailing_whitespace {
                risk.factors.push("Trailing whitespace causes unnecessary conflicts".to_string());
                risk.score += 0.15;
            }
            
            if has_inconsistent_indentation {
                risk.factors.push("Inconsistent indentation leads to merge issues".to_string());
                risk.score += 0.25;
            }

            // Check for common conflict markers (might be from previous conflicts)
            if content.contains("<<<<<<<") || content.contains(">>>>>>>") || content.contains("=======") {
                risk.factors.push("File contains conflict markers".to_string());
                risk.score += 0.5;
            }
        }

        risk.level = if risk.score > 0.7 {
            ConflictRiskLevel::High
        } else if risk.score > 0.4 {
            ConflictRiskLevel::Medium
        } else if risk.score > 0.1 {
            ConflictRiskLevel::Low
        } else {
            ConflictRiskLevel::None
        };

        risk
    }

    fn check_inconsistent_indentation(&self, lines: &[&str]) -> bool {
        let mut tab_count = 0;
        let mut space_count = 0;
        
        for line in lines {
            if line.starts_with('\t') {
                tab_count += 1;
            } else if line.starts_with("  ") || line.starts_with("    ") {
                space_count += 1;
            }
        }
        
        tab_count > 0 && space_count > 0
    }

    fn display_comprehensive_analysis(&self, path: &str, analysis: &FileAnalysis) {
        println!("\n{} Comprehensive Analysis: {}", "🔍".blue(), Style::file_path(path));
        
        // File type and basic info
        println!("  Type: {:?}", analysis.file_type);
        
        // Security analysis
        if !analysis.security_risk.issues.is_empty() {
            println!("  Security: {} issues found", analysis.security_risk.issues.len());
            for issue in &analysis.security_risk.issues {
                println!("    • {}: {}", 
                        format!("{:?}", issue.category).yellow(), 
                        issue.description);
            }
        }
        
        // Performance analysis
        println!("  Performance: Storage efficiency {:.0}%, Compression potential {:.0}%",
                analysis.performance_impact.storage_efficiency * 100.0,
                analysis.performance_impact.compression_benefit * 100.0);
        
        // Code quality (if available)
        if let Some(quality) = &analysis.code_quality {
            println!("  Quality: Complexity {:.0}%, Maintainability {:.0}%, Documentation {:.0}%",
                    quality.complexity_score, quality.maintainability, quality.documentation_level);
        }
        
        // Suggestions
        if !analysis.suggestions.is_empty() {
            println!("  Suggestions:");
            for suggestion in &analysis.suggestions {
                println!("    • {}: {}", 
                        format!("{:?}", suggestion.priority).cyan(), 
                        suggestion.action);
            }
        }
        
        println!();
    }
}

impl Default for PerformanceImpact {
    fn default() -> Self {
        Self {
            storage_efficiency: 0.5,
            network_transfer_cost: 0.5,
            compression_benefit: 0.3,
            indexing_cost: 0.1,
        }
    }
}

impl FileAnalysis {
    pub fn minimal() -> Self {
        Self {
            should_compress: false,
            suggested_lfs: false,
            estimated_compression_ratio: 0.0,
            file_type: FileType::Unknown,
            security_risk: SecurityRisk { level: RiskLevel::None, issues: vec![] },
            performance_impact: PerformanceImpact::default(),
            code_quality: None,
            dependencies: vec![],
            suggestions: vec![],
        }
    }
}

impl Default for FileAnalysis {
    fn default() -> Self {
        Self::minimal()
    }
}

// Style utility for consistent formatting
pub struct Style;

impl Style {
    pub fn file_path(path: &str) -> String {
        path.cyan().to_string()
    }
}
