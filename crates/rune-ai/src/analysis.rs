use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysis {
    pub file_path: String,
    pub language: String,
    pub metrics: CodeMetrics,
    pub issues: Vec<CodeIssue>,
    pub suggestions: Vec<CodeSuggestion>,
    pub complexity_score: f64,
    pub maintainability_index: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetrics {
    pub lines_of_code: usize,
    pub cyclomatic_complexity: usize,
    pub cognitive_complexity: usize,
    pub function_count: usize,
    pub class_count: usize,
    pub test_coverage: f64,
    pub duplicate_lines: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeIssue {
    pub issue_type: IssueType,
    pub severity: IssueSeverity,
    pub message: String,
    pub line_number: usize,
    pub column: usize,
    pub rule_id: String,
    pub fix_suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueType {
    Bug,
    CodeSmell,
    Security,
    Performance,
    Style,
    Maintainability,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Blocker,
    Critical,
    Major,
    Minor,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSuggestion {
    pub suggestion_type: SuggestionType,
    pub title: String,
    pub description: String,
    pub confidence: f64,
    pub effort: EffortLevel,
    pub impact: ImpactLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    Refactoring,
    Optimization,
    TestImprovement,
    Documentation,
    Architecture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    Low,
    Medium,
    High,
    VeryHigh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

pub struct CodeAnalyzer {
    language_configs: HashMap<String, LanguageConfig>,
}

#[derive(Debug, Clone)]
struct LanguageConfig {
    file_extensions: Vec<String>,
    complexity_threshold: usize,
    line_length_limit: usize,
    patterns: Vec<AnalysisPattern>,
}

#[derive(Debug, Clone)]
struct AnalysisPattern {
    pattern: regex::Regex,
    issue_type: IssueType,
    severity: IssueSeverity,
    message: String,
}

impl CodeAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            language_configs: HashMap::new(),
        };
        
        analyzer.setup_language_configs();
        analyzer
    }

    pub fn analyze_file(&self, file_path: &str, content: &str) -> Result<CodeAnalysis> {
        let language = self.detect_language(file_path)?;
        let metrics = self.calculate_metrics(content, &language)?;
        let issues = self.detect_issues(content, &language)?;
        let suggestions = self.generate_suggestions(&metrics, &issues)?;
        
        let complexity_score = self.calculate_complexity_score(&metrics);
        let maintainability_index = self.calculate_maintainability_index(&metrics, &issues);

        Ok(CodeAnalysis {
            file_path: file_path.to_string(),
            language,
            metrics,
            issues,
            suggestions,
            complexity_score,
            maintainability_index,
        })
    }

    pub fn analyze_repository(&self, repo_path: &str) -> Result<Vec<CodeAnalysis>> {
        let mut analyses = Vec::new();
        
        for entry in walkdir::WalkDir::new(repo_path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                if let Some(path) = entry.path().to_str() {
                    if self.should_analyze_file(path) {
                        let content = std::fs::read_to_string(path)?;
                        match self.analyze_file(path, &content) {
                            Ok(analysis) => analyses.push(analysis),
                            Err(e) => eprintln!("Failed to analyze {}: {}", path, e),
                        }
                    }
                }
            }
        }
        
        Ok(analyses)
    }

    pub fn get_repository_summary(&self, analyses: &[CodeAnalysis]) -> RepositorySummary {
        let total_files = analyses.len();
        let total_lines = analyses.iter().map(|a| a.metrics.lines_of_code).sum();
        let avg_complexity = if total_files > 0 {
            analyses.iter().map(|a| a.complexity_score).sum::<f64>() / total_files as f64
        } else {
            0.0
        };
        
        let total_issues = analyses.iter().map(|a| a.issues.len()).sum();
        let critical_issues = analyses
            .iter()
            .flat_map(|a| &a.issues)
            .filter(|issue| matches!(issue.severity, IssueSeverity::Critical | IssueSeverity::Blocker))
            .count();

        let language_distribution = self.calculate_language_distribution(analyses);
        let top_issues = self.get_top_issues(analyses);

        RepositorySummary {
            total_files,
            total_lines,
            avg_complexity,
            total_issues,
            critical_issues,
            language_distribution,
            top_issues,
            overall_health_score: self.calculate_health_score(analyses),
        }
    }

    fn detect_language(&self, file_path: &str) -> Result<String> {
        let extension = std::path::Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let language = match extension {
            "rs" => "rust",
            "js" | "jsx" => "javascript",
            "ts" | "tsx" => "typescript",
            "py" => "python",
            "java" => "java",
            "cpp" | "cc" | "cxx" => "cpp",
            "c" => "c",
            "go" => "go",
            "rb" => "ruby",
            "php" => "php",
            _ => "unknown",
        };

        Ok(language.to_string())
    }

    fn calculate_metrics(&self, content: &str, language: &str) -> Result<CodeMetrics> {
        let lines_of_code = content.lines().filter(|line| !line.trim().is_empty()).count();
        let function_count = self.count_functions(content, language);
        let class_count = self.count_classes(content, language);
        
        Ok(CodeMetrics {
            lines_of_code,
            cyclomatic_complexity: self.calculate_cyclomatic_complexity(content, language),
            cognitive_complexity: self.calculate_cognitive_complexity(content, language),
            function_count,
            class_count,
            test_coverage: 0.0, // TODO: Integrate with test coverage tools
            duplicate_lines: self.find_duplicate_lines(content),
        })
    }

    fn detect_issues(&self, content: &str, language: &str) -> Result<Vec<CodeIssue>> {
        let mut issues = Vec::new();
        
        if let Some(config) = self.language_configs.get(language) {
            for (line_num, line) in content.lines().enumerate() {
                // Check line length
                if line.len() > config.line_length_limit {
                    issues.push(CodeIssue {
                        issue_type: IssueType::Style,
                        severity: IssueSeverity::Minor,
                        message: format!("Line too long ({} characters)", line.len()),
                        line_number: line_num + 1,
                        column: config.line_length_limit,
                        rule_id: "line-length".to_string(),
                        fix_suggestion: Some("Break long line into multiple lines".to_string()),
                    });
                }
                
                // Apply pattern-based rules
                for pattern in &config.patterns {
                    if pattern.pattern.is_match(line) {
                        issues.push(CodeIssue {
                            issue_type: pattern.issue_type.clone(),
                            severity: pattern.severity.clone(),
                            message: pattern.message.clone(),
                            line_number: line_num + 1,
                            column: 1,
                            rule_id: "pattern-match".to_string(),
                            fix_suggestion: None,
                        });
                    }
                }
            }
        }
        
        Ok(issues)
    }

    fn generate_suggestions(&self, metrics: &CodeMetrics, issues: &[CodeIssue]) -> Result<Vec<CodeSuggestion>> {
        let mut suggestions = Vec::new();
        
        // High complexity suggestion
        if metrics.cyclomatic_complexity > 10 {
            suggestions.push(CodeSuggestion {
                suggestion_type: SuggestionType::Refactoring,
                title: "Reduce Cyclomatic Complexity".to_string(),
                description: "Consider breaking down complex functions into smaller ones".to_string(),
                confidence: 0.9,
                effort: EffortLevel::Medium,
                impact: ImpactLevel::High,
            });
        }
        
        // Test coverage suggestion
        if metrics.test_coverage < 0.8 {
            suggestions.push(CodeSuggestion {
                suggestion_type: SuggestionType::TestImprovement,
                title: "Improve Test Coverage".to_string(),
                description: "Add unit tests to increase code coverage".to_string(),
                confidence: 0.85,
                effort: EffortLevel::Medium,
                impact: ImpactLevel::Medium,
            });
        }
        
        // Security issues suggestion
        let security_issues = issues.iter().filter(|i| matches!(i.issue_type, IssueType::Security)).count();
        if security_issues > 0 {
            suggestions.push(CodeSuggestion {
                suggestion_type: SuggestionType::Architecture,
                title: "Address Security Issues".to_string(),
                description: format!("Fix {} security-related issues", security_issues),
                confidence: 0.95,
                effort: EffortLevel::High,
                impact: ImpactLevel::Critical,
            });
        }
        
        Ok(suggestions)
    }

    fn calculate_complexity_score(&self, metrics: &CodeMetrics) -> f64 {
        // Simple complexity scoring algorithm
        let base_score = 100.0;
        let complexity_penalty = (metrics.cyclomatic_complexity as f64) * 2.0;
        let cognitive_penalty = (metrics.cognitive_complexity as f64) * 1.5;
        
        (base_score - complexity_penalty - cognitive_penalty).max(0.0).min(100.0)
    }

    fn calculate_maintainability_index(&self, metrics: &CodeMetrics, issues: &[CodeIssue]) -> f64 {
        // Simplified maintainability index calculation
        let base_score = 100.0;
        let lines_penalty = (metrics.lines_of_code as f64 / 1000.0) * 5.0;
        let complexity_penalty = (metrics.cyclomatic_complexity as f64) * 3.0;
        let issues_penalty = issues.len() as f64 * 2.0;
        
        (base_score - lines_penalty - complexity_penalty - issues_penalty).max(0.0).min(100.0)
    }

    fn setup_language_configs(&mut self) {
        // Rust configuration
        let rust_patterns = vec![
            AnalysisPattern {
                pattern: regex::Regex::new(r"\.unwrap\(\)").unwrap(),
                issue_type: IssueType::Bug,
                severity: IssueSeverity::Major,
                message: "Avoid using unwrap() - use proper error handling".to_string(),
            },
            AnalysisPattern {
                pattern: regex::Regex::new(r"\.clone\(\)").unwrap(),
                issue_type: IssueType::Performance,
                severity: IssueSeverity::Minor,
                message: "Consider using references instead of cloning".to_string(),
            },
        ];

        self.language_configs.insert("rust".to_string(), LanguageConfig {
            file_extensions: vec!["rs".to_string()],
            complexity_threshold: 10,
            line_length_limit: 100,
            patterns: rust_patterns,
        });

        // JavaScript configuration
        let js_patterns = vec![
            AnalysisPattern {
                pattern: regex::Regex::new(r"==\s").unwrap(),
                issue_type: IssueType::Bug,
                severity: IssueSeverity::Minor,
                message: "Use === instead of == for strict equality".to_string(),
            },
        ];

        self.language_configs.insert("javascript".to_string(), LanguageConfig {
            file_extensions: vec!["js".to_string(), "jsx".to_string()],
            complexity_threshold: 10,
            line_length_limit: 120,
            patterns: js_patterns,
        });
    }

    fn should_analyze_file(&self, path: &str) -> bool {
        let extension = std::path::Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        matches!(extension, "rs" | "js" | "jsx" | "ts" | "tsx" | "py" | "java" | "cpp" | "c" | "go")
    }

    fn count_functions(&self, content: &str, language: &str) -> usize {
        match language {
            "rust" => content.matches("fn ").count(),
            "javascript" | "typescript" => {
                content.matches("function ").count() + content.matches("=> ").count()
            }
            "python" => content.matches("def ").count(),
            "java" | "cpp" | "c" => content.matches("(").count(), // Simplified
            _ => 0,
        }
    }

    fn count_classes(&self, content: &str, language: &str) -> usize {
        match language {
            "rust" => content.matches("struct ").count() + content.matches("enum ").count(),
            "javascript" | "typescript" => content.matches("class ").count(),
            "python" => content.matches("class ").count(),
            "java" | "cpp" => content.matches("class ").count(),
            _ => 0,
        }
    }

    fn calculate_cyclomatic_complexity(&self, content: &str, _language: &str) -> usize {
        // Simplified cyclomatic complexity calculation
        let decision_points = content.matches("if ").count()
            + content.matches("else ").count()
            + content.matches("while ").count()
            + content.matches("for ").count()
            + content.matches("match ").count()
            + content.matches("case ").count();
        
        decision_points + 1 // +1 for the initial path
    }

    fn calculate_cognitive_complexity(&self, content: &str, _language: &str) -> usize {
        // Simplified cognitive complexity calculation
        content.matches("if ").count() * 1
            + content.matches("else ").count() * 1
            + content.matches("while ").count() * 2
            + content.matches("for ").count() * 2
            + content.matches("try ").count() * 2
            + content.matches("catch ").count() * 2
    }

    fn find_duplicate_lines(&self, content: &str) -> usize {
        let lines: Vec<&str> = content.lines().collect();
        let mut duplicates = 0;
        
        for i in 0..lines.len() {
            for j in (i + 1)..lines.len() {
                if lines[i].trim() == lines[j].trim() && !lines[i].trim().is_empty() {
                    duplicates += 1;
                    break;
                }
            }
        }
        
        duplicates
    }

    fn calculate_language_distribution(&self, analyses: &[CodeAnalysis]) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();
        
        for analysis in analyses {
            *distribution.entry(analysis.language.clone()).or_insert(0) += 1;
        }
        
        distribution
    }

    fn get_top_issues(&self, analyses: &[CodeAnalysis]) -> Vec<String> {
        let mut issue_counts: HashMap<String, usize> = HashMap::new();
        
        for analysis in analyses {
            for issue in &analysis.issues {
                *issue_counts.entry(issue.rule_id.clone()).or_insert(0) += 1;
            }
        }
        
        let mut sorted_issues: Vec<_> = issue_counts.into_iter().collect();
        sorted_issues.sort_by(|a, b| b.1.cmp(&a.1));
        
        sorted_issues.into_iter()
            .take(5)
            .map(|(rule, count)| format!("{}: {} occurrences", rule, count))
            .collect()
    }

    fn calculate_health_score(&self, analyses: &[CodeAnalysis]) -> f64 {
        if analyses.is_empty() {
            return 0.0;
        }
        
        let avg_maintainability = analyses.iter()
            .map(|a| a.maintainability_index)
            .sum::<f64>() / analyses.len() as f64;
        
        let avg_complexity = analyses.iter()
            .map(|a| a.complexity_score)
            .sum::<f64>() / analyses.len() as f64;
        
        (avg_maintainability + avg_complexity) / 2.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositorySummary {
    pub total_files: usize,
    pub total_lines: usize,
    pub avg_complexity: f64,
    pub total_issues: usize,
    pub critical_issues: usize,
    pub language_distribution: HashMap<String, usize>,
    pub top_issues: Vec<String>,
    pub overall_health_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_analyzer() {
        let analyzer = CodeAnalyzer::new();
        let rust_code = r#"
fn main() {
    let result = some_function().unwrap();
    println!("{}", result);
}
"#;
        
        let analysis = analyzer.analyze_file("test.rs", rust_code).unwrap();
        assert_eq!(analysis.language, "rust");
        assert!(!analysis.issues.is_empty());
    }

    #[test]
    fn test_language_detection() {
        let analyzer = CodeAnalyzer::new();
        assert_eq!(analyzer.detect_language("test.rs").unwrap(), "rust");
        assert_eq!(analyzer.detect_language("test.js").unwrap(), "javascript");
        assert_eq!(analyzer.detect_language("test.py").unwrap(), "python");
    }
}
