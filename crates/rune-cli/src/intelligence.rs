use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir;

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
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    C,
    Cpp,
    Java,
    Kotlin,
    Other(String),
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
    Credentials,
    Certificates,
    Configuration,
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
    Performance,
    Security,
    Organization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    IOs,
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
    Security,
    Performance,
    Documentation,
    Legal,
}

#[derive(Debug, Clone)]
pub enum InsightSeverity {
    Medium,
    High,
}

pub struct IntelligentFileAnalyzer {
    pub config: IntelligenceConfig,
    cache: HashMap<String, FileAnalysis>,
    repository_insights: RepositoryInsights,
    predictive_model: PredictiveModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInsights {
    pub project_type: ProjectType,
    pub code_metrics: CodeMetrics,
    pub quality_score: f64,
    pub health_indicators: Vec<HealthIndicator>,
    pub growth_patterns: GrowthPatterns,
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetrics {
    pub total_files: usize,
    pub lines_of_code: usize,
    pub test_coverage_estimate: f64,
    pub documentation_ratio: f64,
    pub complexity_score: f64,
    pub maintainability_index: f64,
    pub language_distribution: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIndicator {
    pub indicator: String,
    pub status: HealthStatus,
    pub description: String,
    pub severity: HealthSeverity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HealthStatus {
    Excellent,
    Good,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthPatterns {
    pub files_growth_rate: f64,
    pub size_growth_rate: f64,
    pub complexity_growth_rate: f64,
    pub predicted_size_6months: u64,
    pub predicted_files_6months: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub category: OptimizationCategory,
    pub suggestion: String,
    pub impact_score: f64,
    pub effort_required: EffortLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationCategory {
    Performance,
    Storage,
    Security,
    Maintenance,
    Documentation,
    Testing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone)]
pub struct PredictiveModel {
    cache_predictions: HashMap<String, CachePrediction>,
    access_patterns: HashMap<String, AccessPattern>,
    optimization_cache: HashMap<String, OptimizationCache>,
}

#[derive(Debug, Clone)]
pub struct CachePrediction {
    pub file_path: String,
    pub likelihood_access: f64,
    pub estimated_access_time: std::time::Duration,
    pub cache_priority: CachePriority,
}

#[derive(Debug, Clone)]
pub enum CachePriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct AccessPattern {
    pub frequency: usize,
    pub last_accessed: std::time::SystemTime,
    pub access_trend: AccessTrend,
}

#[derive(Debug, Clone)]
pub enum AccessTrend {
    Increasing,
    Stable,
    Decreasing,
}

#[derive(Debug, Clone)]
pub struct OptimizationCache {
    pub recommendations: Vec<String>,
    pub performance_score: f64,
    pub last_updated: std::time::SystemTime,
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
            repository_insights: RepositoryInsights::default(),
            predictive_model: PredictiveModel::default(),
        }
    }

    pub fn with_config(config: IntelligenceConfig) -> Self {
        Self {
            config,
            cache: HashMap::new(),
            repository_insights: RepositoryInsights::default(),
            predictive_model: PredictiveModel::default(),
        }
    }

    pub fn analyze_repository(
        &mut self,
        repo_path: &Path,
    ) -> Result<RepositoryInsights, std::io::Error> {
        if !self.config.features.repository_health {
            return Ok(RepositoryInsights::default());
        }

        println!(
            "{}",
            "🧠 Analyzing repository intelligence...".cyan().bold()
        );

        let mut code_metrics = CodeMetrics::default();
        let mut files_analyzed = 0;
        let mut total_size = 0u64;

        // Walk through repository
        for entry in walkdir::WalkDir::new(repo_path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Some(file_name) = path.to_str() {
                    if self.should_analyze_file(file_name) {
                        if let Ok(analysis) = self.analyze_file(file_name) {
                            files_analyzed += 1;
                            total_size += analysis.size_bytes;
                            self.update_metrics(&mut code_metrics, &analysis, path);
                        }
                    }
                }
            }
        }

        code_metrics.total_files = files_analyzed;

        let project_type = self.detect_project_type(repo_path);
        let quality_score = self.calculate_quality_score(&code_metrics);
        let health_indicators = self.generate_health_indicators(&code_metrics, total_size);
        let growth_patterns = self.analyze_growth_patterns(&code_metrics);
        let optimization_suggestions =
            self.generate_optimization_suggestions(&code_metrics, &project_type);

        self.repository_insights = RepositoryInsights {
            project_type,
            code_metrics,
            quality_score,
            health_indicators,
            growth_patterns,
            optimization_suggestions,
        };

        Ok(self.repository_insights.clone())
    }

    pub fn generate_predictive_insights(&mut self, repo_path: &Path) -> Vec<PredictiveInsight> {
        if !self.config.features.predictive_modeling {
            return vec![];
        }

        let mut insights = vec![];

        // Analyze cache efficiency
        if let Some(cache_insight) = self.predict_cache_optimization() {
            insights.push(cache_insight);
        }

        // Security predictions
        if let Some(security_insight) = self.predict_security_issues(repo_path) {
            insights.push(security_insight);
        }

        // Performance predictions
        if let Some(performance_insight) = self.predict_performance_bottlenecks() {
            insights.push(performance_insight);
        }

        // Documentation predictions
        if let Some(doc_insight) = self.predict_documentation_needs() {
            insights.push(doc_insight);
        }

        insights
    }

    pub fn get_smart_caching_suggestions(&self) -> Vec<CachePrediction> {
        self.predictive_model
            .cache_predictions
            .values()
            .cloned()
            .collect()
    }

    pub fn update_access_pattern(&mut self, file_path: &str) {
        let pattern = self
            .predictive_model
            .access_patterns
            .entry(file_path.to_string())
            .or_insert(AccessPattern {
                frequency: 0,
                last_accessed: std::time::SystemTime::now(),
                access_trend: AccessTrend::Stable,
            });

        pattern.frequency += 1;
        pattern.last_accessed = std::time::SystemTime::now();

        // Update trend based on frequency changes
        if pattern.frequency > 10 {
            pattern.access_trend = AccessTrend::Increasing;
        } else if pattern.frequency < 3 {
            pattern.access_trend = AccessTrend::Decreasing;
        }
    }

    fn should_analyze_file(&self, file_path: &str) -> bool {
        let path = Path::new(file_path);

        // Skip hidden files and directories
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .map_or(false, |name| name.starts_with('.'))
        {
            return false;
        }

        // Skip common build/cache directories
        let skip_dirs = ["target", "node_modules", ".git", "build", "dist", "out"];
        for component in path.components() {
            if let Some(dir_name) = component.as_os_str().to_str() {
                if skip_dirs.contains(&dir_name) {
                    return false;
                }
            }
        }

        true
    }

    fn update_metrics(&self, metrics: &mut CodeMetrics, analysis: &FileAnalysis, path: &Path) {
        // Update language distribution
        let lang = match &analysis.language {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Go => "Go",
            Language::C => "C",
            Language::Cpp => "C++",
            Language::Java => "Java",
            Language::Kotlin => "Kotlin",
            Language::Other(name) => name,
        };

        *metrics
            .language_distribution
            .entry(lang.to_string())
            .or_insert(0) += 1;

        // Estimate lines of code for text files
        if let Ok(content) = std::fs::read_to_string(path) {
            metrics.lines_of_code += content.lines().count();

            // Simple heuristics for documentation ratio
            if path
                .extension()
                .and_then(|ext| ext.to_str())
                .map_or(false, |ext| {
                    matches!(ext.to_lowercase().as_str(), "md" | "txt" | "rst" | "adoc")
                })
            {
                metrics.documentation_ratio += 1.0;
            }
        }
    }

    fn detect_project_type(&self, repo_path: &Path) -> ProjectType {
        // Check for specific project indicators
        if repo_path.join("Cargo.toml").exists() {
            return ProjectType::Rust;
        }
        if repo_path.join("package.json").exists() {
            if repo_path.join("next.config.js").exists() {
                return ProjectType::NextJS;
            }
            // Read package.json to detect React, Vue, Angular
            if let Ok(content) = std::fs::read_to_string(repo_path.join("package.json")) {
                if content.contains("\"react\"") {
                    return ProjectType::React;
                }
                if content.contains("\"vue\"") {
                    return ProjectType::Vue;
                }
                if content.contains("\"@angular/") {
                    return ProjectType::Angular;
                }
            }
            return ProjectType::JavaScript;
        }
        if repo_path.join("requirements.txt").exists() || repo_path.join("setup.py").exists() {
            // Check for data science indicators
            if let Ok(content) =
                std::fs::read_to_string(repo_path.join("requirements.txt").as_path())
                    .or_else(|_| std::fs::read_to_string(repo_path.join("setup.py").as_path()))
            {
                if content.contains("pandas")
                    || content.contains("numpy")
                    || content.contains("sklearn")
                {
                    return ProjectType::DataScience;
                }
                if content.contains("tensorflow")
                    || content.contains("torch")
                    || content.contains("keras")
                {
                    return ProjectType::MachineLearning;
                }
            }
            return ProjectType::Python;
        }
        if repo_path.join("go.mod").exists() {
            return ProjectType::Go;
        }

        ProjectType::Other("Unknown".to_string())
    }

    fn calculate_quality_score(&self, metrics: &CodeMetrics) -> f64 {
        let mut score = 0.0;
        let mut factors = 0;

        // Documentation factor
        if metrics.total_files > 0 {
            let doc_ratio = metrics.documentation_ratio / metrics.total_files as f64;
            score += (doc_ratio * 100.0).min(30.0); // Max 30 points for documentation
            factors += 1;
        }

        // Language diversity (moderate diversity is good)
        let lang_count = metrics.language_distribution.len();
        if lang_count > 0 {
            let diversity_score = match lang_count {
                1..=3 => 25.0,
                4..=6 => 20.0,
                _ => 10.0, // Too many languages can indicate complexity
            };
            score += diversity_score;
            factors += 1;
        }

        // Size factor (reasonable project size)
        let size_score = match metrics.total_files {
            0..=10 => 15.0,
            11..=100 => 30.0,
            101..=1000 => 25.0,
            _ => 15.0,
        };
        score += size_score;
        factors += 1;

        // Maintainability index (simulated)
        score += metrics.maintainability_index * 40.0; // Max 40 points
        factors += 1;

        if factors > 0 {
            score / factors as f64
        } else {
            50.0 // Default score
        }
    }

    fn generate_health_indicators(
        &self,
        metrics: &CodeMetrics,
        total_size: u64,
    ) -> Vec<HealthIndicator> {
        let mut indicators = vec![];

        // File count indicator
        let file_status = match metrics.total_files {
            0..=10 => (HealthStatus::Warning, "Very few files detected"),
            11..=100 => (HealthStatus::Good, "Healthy file count"),
            101..=1000 => (HealthStatus::Excellent, "Well-structured project"),
            _ => (
                HealthStatus::Warning,
                "Very large project - consider modularization",
            ),
        };
        indicators.push(HealthIndicator {
            indicator: "Project Size".to_string(),
            status: file_status.0,
            description: file_status.1.to_string(),
            severity: if matches!(file_status.0, HealthStatus::Warning) {
                HealthSeverity::Medium
            } else {
                HealthSeverity::Low
            },
        });

        // Documentation indicator
        let doc_ratio = if metrics.total_files > 0 {
            metrics.documentation_ratio / metrics.total_files as f64
        } else {
            0.0
        };
        let doc_status = match doc_ratio {
            r if r > 0.2 => (HealthStatus::Excellent, "Excellent documentation coverage"),
            r if r > 0.1 => (HealthStatus::Good, "Good documentation coverage"),
            r if r > 0.05 => (HealthStatus::Warning, "Limited documentation"),
            _ => (HealthStatus::Critical, "Missing documentation"),
        };
        indicators.push(HealthIndicator {
            indicator: "Documentation Coverage".to_string(),
            status: doc_status.0,
            description: doc_status.1.to_string(),
            severity: match doc_status.0 {
                HealthStatus::Critical => HealthSeverity::High,
                HealthStatus::Warning => HealthSeverity::Medium,
                _ => HealthSeverity::Low,
            },
        });

        // Repository size indicator
        let size_mb = total_size as f64 / (1024.0 * 1024.0);
        let size_status = match size_mb {
            s if s < 10.0 => (HealthStatus::Good, "Compact repository size"),
            s if s < 100.0 => (HealthStatus::Good, "Reasonable repository size"),
            s if s < 500.0 => (HealthStatus::Warning, "Large repository - consider LFS"),
            _ => (
                HealthStatus::Critical,
                "Very large repository - LFS recommended",
            ),
        };
        indicators.push(HealthIndicator {
            indicator: "Repository Size".to_string(),
            status: size_status.0,
            description: format!("{} ({:.1} MB)", size_status.1, size_mb),
            severity: match size_status.0 {
                HealthStatus::Critical => HealthSeverity::High,
                HealthStatus::Warning => HealthSeverity::Medium,
                _ => HealthSeverity::Low,
            },
        });

        indicators
    }

    fn analyze_growth_patterns(&self, metrics: &CodeMetrics) -> GrowthPatterns {
        // Simplified growth pattern analysis
        // In a real implementation, this would analyze commit history
        GrowthPatterns {
            files_growth_rate: 0.1,       // 10% growth assumed
            size_growth_rate: 0.15,       // 15% size growth assumed
            complexity_growth_rate: 0.05, // 5% complexity growth
            predicted_size_6months: ((metrics.total_files as f64) * 1.5) as u64,
            predicted_files_6months: ((metrics.total_files as f64) * 1.3) as usize,
        }
    }

    fn generate_optimization_suggestions(
        &self,
        metrics: &CodeMetrics,
        project_type: &ProjectType,
    ) -> Vec<OptimizationSuggestion> {
        let mut suggestions = vec![];

        // Documentation suggestions
        let doc_ratio = if metrics.total_files > 0 {
            metrics.documentation_ratio / metrics.total_files as f64
        } else {
            0.0
        };

        if doc_ratio < 0.1 {
            suggestions.push(OptimizationSuggestion {
                category: OptimizationCategory::Documentation,
                suggestion:
                    "Add README.md and API documentation to improve project maintainability"
                        .to_string(),
                impact_score: 8.5,
                effort_required: EffortLevel::Medium,
            });
        }

        // Project-specific suggestions
        match project_type {
            ProjectType::Rust => {
                suggestions.push(OptimizationSuggestion {
                    category: OptimizationCategory::Performance,
                    suggestion: "Consider using cargo-audit for security vulnerability scanning"
                        .to_string(),
                    impact_score: 7.0,
                    effort_required: EffortLevel::Low,
                });
            }
            ProjectType::JavaScript | ProjectType::TypeScript => {
                suggestions.push(OptimizationSuggestion {
                    category: OptimizationCategory::Security,
                    suggestion: "Run npm audit to check for security vulnerabilities".to_string(),
                    impact_score: 8.0,
                    effort_required: EffortLevel::Low,
                });
            }
            ProjectType::Python => {
                suggestions.push(OptimizationSuggestion {
                    category: OptimizationCategory::Security,
                    suggestion: "Use safety or pip-audit to scan for known vulnerabilities"
                        .to_string(),
                    impact_score: 7.5,
                    effort_required: EffortLevel::Low,
                });
            }
            _ => {}
        }

        // Size-based suggestions
        if metrics.total_files > 500 {
            suggestions.push(OptimizationSuggestion {
                category: OptimizationCategory::Storage,
                suggestion:
                    "Consider implementing LFS for large binary files to improve clone performance"
                        .to_string(),
                impact_score: 9.0,
                effort_required: EffortLevel::Medium,
            });
        }

        suggestions
    }

    fn predict_cache_optimization(&self) -> Option<PredictiveInsight> {
        // Analyze access patterns to suggest cache optimizations
        let high_frequency_files = self
            .predictive_model
            .access_patterns
            .iter()
            .filter(|(_, pattern)| pattern.frequency > 5)
            .count();

        if high_frequency_files > 10 {
            Some(PredictiveInsight {
                insight: format!("Detected {} frequently accessed files. Consider implementing smart caching for improved performance.", high_frequency_files),
                confidence: 0.85,
                category: InsightCategory::Performance,
                severity: InsightSeverity::Medium,
            })
        } else {
            None
        }
    }

    fn predict_security_issues(&self, _repo_path: &Path) -> Option<PredictiveInsight> {
        // Simplified security prediction
        Some(PredictiveInsight {
            insight:
                "Regular security audits recommended. Run dependency vulnerability scans monthly."
                    .to_string(),
            confidence: 0.75,
            category: InsightCategory::Security,
            severity: InsightSeverity::Medium,
        })
    }

    fn predict_performance_bottlenecks(&self) -> Option<PredictiveInsight> {
        // Analyze code metrics for potential performance issues
        let total_files = self.repository_insights.code_metrics.total_files;

        if total_files > 1000 {
            Some(PredictiveInsight {
                insight: "Large codebase detected. Consider implementing incremental compilation and caching strategies.".to_string(),
                confidence: 0.80,
                category: InsightCategory::Performance,
                severity: InsightSeverity::High,
            })
        } else {
            None
        }
    }

    fn predict_documentation_needs(&self) -> Option<PredictiveInsight> {
        let doc_ratio = if self.repository_insights.code_metrics.total_files > 0 {
            self.repository_insights.code_metrics.documentation_ratio
                / self.repository_insights.code_metrics.total_files as f64
        } else {
            0.0
        };

        if doc_ratio < 0.05 {
            Some(PredictiveInsight {
                insight: "Low documentation coverage detected. Consider adding inline documentation and README files.".to_string(),
                confidence: 0.90,
                category: InsightCategory::Documentation,
                severity: InsightSeverity::High,
            })
        } else {
            None
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
                if file_size > 10_000_000 {
                    // Assume large files are binary
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
        if lower_path.contains(".env")
            || lower_path.contains("secret")
            || lower_path.contains("password")
        {
            issues.push(SecurityIssue {
                issue_type: "Potential credential file".to_string(),
                severity: "High".to_string(),
                description: "File may contain sensitive credentials".to_string(),
                category: SecurityCategory::Credentials,
            });
        }

        // Check for hardcoded secrets in content
        if lower_contents.contains("password")
            || lower_contents.contains("api_key")
            || lower_contents.contains("secret")
            || lower_contents.contains("token")
        {
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
        let storage_efficiency = if file_size > 100_000_000 {
            // 100MB
            0.3
        } else if file_size > 10_000_000 {
            // 10MB
            0.7
        } else {
            1.0
        };

        PerformanceImpact {
            storage_efficiency,
            compression_benefit: if file_size > 1_000_000 { 0.6 } else { 0.1 },
        }
    }

    fn generate_suggestions(
        &self,
        file_path: &str,
        file_size: u64,
        security_issues: &[SecurityIssue],
    ) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        if file_size > 50_000_000 {
            // 50MB
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
        println!(
            "  LFS Suggested: {}",
            if analysis.suggested_lfs {
                "Yes".green()
            } else {
                "No".red()
            }
        );

        if !analysis.security_issues.is_empty() {
            println!(
                "  Security: {} issues found",
                analysis.security_issues.len()
            );
            for issue in &analysis.security_issues {
                println!("    - {}: {}", issue.severity.red(), issue.description);
            }
        }

        if !analysis.suggestions.is_empty() {
            println!("  Suggestions:");
            for suggestion in &analysis.suggestions {
                println!(
                    "    - [{}] {}",
                    suggestion.priority.yellow(),
                    suggestion.action
                );
            }
        }
    }
}

impl Default for RepositoryInsights {
    fn default() -> Self {
        Self {
            project_type: ProjectType::Other("Unknown".to_string()),
            code_metrics: CodeMetrics::default(),
            quality_score: 50.0,
            health_indicators: vec![],
            growth_patterns: GrowthPatterns::default(),
            optimization_suggestions: vec![],
        }
    }
}

impl Default for CodeMetrics {
    fn default() -> Self {
        Self {
            total_files: 0,
            lines_of_code: 0,
            test_coverage_estimate: 0.0,
            documentation_ratio: 0.0,
            complexity_score: 1.0,
            maintainability_index: 0.5,
            language_distribution: HashMap::new(),
        }
    }
}

impl Default for GrowthPatterns {
    fn default() -> Self {
        Self {
            files_growth_rate: 0.0,
            size_growth_rate: 0.0,
            complexity_growth_rate: 0.0,
            predicted_size_6months: 0,
            predicted_files_6months: 0,
        }
    }
}

impl Default for PredictiveModel {
    fn default() -> Self {
        Self {
            cache_predictions: HashMap::new(),
            access_patterns: HashMap::new(),
            optimization_cache: HashMap::new(),
        }
    }
}

// Display utilities for repository insights
impl RepositoryInsights {
    pub fn display_comprehensive_report(&self) {
        println!("\n{}", "📊 REPOSITORY INTELLIGENCE REPORT".cyan().bold());
        println!("{}", "═".repeat(50).cyan());

        // Project overview
        println!("\n{}", "🎯 Project Overview".yellow().bold());
        println!("  Project Type: {:?}", self.project_type);
        println!("  Quality Score: {:.1}/100", self.quality_score);

        // Code metrics
        println!("\n{}", "📈 Code Metrics".yellow().bold());
        println!("  Total Files: {}", self.code_metrics.total_files);
        println!("  Lines of Code: {}", self.code_metrics.lines_of_code);
        println!(
            "  Documentation Ratio: {:.1}%",
            self.code_metrics.documentation_ratio * 100.0
                / self.code_metrics.total_files.max(1) as f64
        );
        println!(
            "  Maintainability Index: {:.1}/1.0",
            self.code_metrics.maintainability_index
        );

        // Language distribution
        if !self.code_metrics.language_distribution.is_empty() {
            println!("\n{}", "🌐 Language Distribution".yellow().bold());
            let mut langs: Vec<_> = self.code_metrics.language_distribution.iter().collect();
            langs.sort_by(|a, b| b.1.cmp(a.1));
            for (lang, count) in langs {
                let percentage = (*count as f64 / self.code_metrics.total_files as f64) * 100.0;
                println!("  {}: {} files ({:.1}%)", lang, count, percentage);
            }
        }

        // Health indicators
        if !self.health_indicators.is_empty() {
            println!("\n{}", "🏥 Health Indicators".yellow().bold());
            for indicator in &self.health_indicators {
                let status_color = match indicator.status {
                    HealthStatus::Excellent => "excellent".green(),
                    HealthStatus::Good => "good".blue(),
                    HealthStatus::Warning => "warning".yellow(),
                    HealthStatus::Critical => "critical".red(),
                };
                println!(
                    "  {} [{}]: {}",
                    indicator.indicator, status_color, indicator.description
                );
            }
        }

        // Growth patterns
        println!("\n{}", "📊 Growth Patterns".yellow().bold());
        println!(
            "  Files Growth Rate: {:.1}%",
            self.growth_patterns.files_growth_rate * 100.0
        );
        println!(
            "  Size Growth Rate: {:.1}%",
            self.growth_patterns.size_growth_rate * 100.0
        );
        println!(
            "  Predicted Files (6 months): {}",
            self.growth_patterns.predicted_files_6months
        );

        // Optimization suggestions
        if !self.optimization_suggestions.is_empty() {
            println!("\n{}", "💡 Optimization Suggestions".yellow().bold());
            for (i, suggestion) in self.optimization_suggestions.iter().enumerate() {
                let effort_color = match suggestion.effort_required {
                    EffortLevel::Low => "low effort".green(),
                    EffortLevel::Medium => "medium effort".yellow(),
                    EffortLevel::High => "high effort".red(),
                };
                println!(
                    "  {}. [Impact: {:.1}] [{}] {}",
                    i + 1,
                    suggestion.impact_score,
                    effort_color,
                    suggestion.suggestion
                );
            }
        }

        println!("\n{}", "═".repeat(50).cyan());
    }
}

impl PredictiveInsight {
    pub fn display(&self) {
        let severity_color = match self.severity {
            InsightSeverity::Medium => "medium".yellow(),
            InsightSeverity::High => "high".red(),
        };
        let category_icon = match self.category {
            InsightCategory::Security => "🔒",
            InsightCategory::Performance => "⚡",
            InsightCategory::Documentation => "📚",
            InsightCategory::Legal => "⚖️",
        };

        println!(
            "  {} [{}] [Confidence: {:.0}%] {}",
            category_icon,
            severity_color,
            self.confidence * 100.0,
            self.insight
        );
    }
}
