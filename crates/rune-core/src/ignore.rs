use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};
use regex::Regex;
use anyhow::{Result, Context};

/// Advanced ignore system that improves on Git's .gitignore
/// Features:
/// - Simple, readable syntax with auto-completion
/// - Performance-optimized with caching and indexing
/// - Clear precedence rules with explicit priorities
/// - Smart templates for project types (auto-detection)
/// - Dynamic patterns that adapt to project structure
/// - Debug mode to understand ignore decisions

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreConfig {
    /// Version of the ignore format for future compatibility
    pub version: String,
    /// Global ignore rules that apply to all projects
    pub global: Vec<IgnoreRule>,
    /// Project-specific ignore rules
    pub project: Vec<IgnoreRule>,
    /// Auto-detected project templates
    pub templates: Vec<String>,
    /// Performance settings
    pub performance: PerformanceSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreRule {
    /// Pattern to match (simplified syntax)
    pub pattern: String,
    /// Rule type (ignore, include, or conditional)
    pub rule_type: RuleType,
    /// Priority (higher numbers take precedence)
    pub priority: i32,
    /// Description for debugging
    pub description: Option<String>,
    /// Condition for conditional rules
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleType {
    /// Ignore matching files/directories
    Ignore,
    /// Include matching files (overrides ignore)
    Include,
    /// Conditional rule based on project context
    Conditional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    /// Enable caching of ignore decisions
    pub enable_cache: bool,
    /// Maximum cache size in entries
    pub cache_size: usize,
    /// Enable pattern pre-compilation
    pub precompile_patterns: bool,
}

#[derive(Debug)]
pub struct IgnoreEngine {
    config: IgnoreConfig,
    compiled_patterns: HashMap<String, Regex>,
    cache: HashMap<PathBuf, bool>,
    project_root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreTemplate {
    pub name: String,
    pub description: String,
    pub patterns: Vec<String>,
    pub auto_detect: Vec<String>, // Files that indicate this project type
}

impl Default for IgnoreConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            global: Self::default_global_rules(),
            project: Vec::new(),
            templates: Vec::new(),
            performance: PerformanceSettings {
                enable_cache: true,
                cache_size: 10000,
                precompile_patterns: true,
            },
        }
    }
}

impl IgnoreConfig {
    /// Default global ignore rules that apply to all projects
    fn default_global_rules() -> Vec<IgnoreRule> {
        vec![
            IgnoreRule {
                pattern: "**/.DS_Store".to_string(),
                rule_type: RuleType::Ignore,
                priority: 100,
                description: Some("macOS system files".to_string()),
                condition: None,
            },
            IgnoreRule {
                pattern: "**/Thumbs.db".to_string(),
                rule_type: RuleType::Ignore,
                priority: 100,
                description: Some("Windows thumbnail cache".to_string()),
                condition: None,
            },
            IgnoreRule {
                pattern: "**/*.tmp".to_string(),
                rule_type: RuleType::Ignore,
                priority: 90,
                description: Some("Temporary files".to_string()),
                condition: None,
            },
            IgnoreRule {
                pattern: "**/*.swp".to_string(),
                rule_type: RuleType::Ignore,
                priority: 90,
                description: Some("Vim swap files".to_string()),
                condition: None,
            },
            IgnoreRule {
                pattern: "**/*~".to_string(),
                rule_type: RuleType::Ignore,
                priority: 90,
                description: Some("Backup files".to_string()),
                condition: None,
            },
        ]
    }

    /// Load ignore configuration from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path).context("Failed to read ignore config file")?;
        Ok(serde_yaml::from_str(&content).context("Failed to parse ignore config")?)
    }

    /// Save ignore configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_yaml::to_string(self).context("Failed to serialize ignore config")?;
        fs::write(path, content).context("Failed to write ignore config file")?;
        Ok(())
    }
}

impl IgnoreEngine {
    /// Create a new ignore engine for a project
    pub fn new<P: AsRef<Path>>(project_root: P) -> Result<Self> {
        let project_root = project_root.as_ref().to_path_buf();
        let mut config = IgnoreConfig::default();
        
        // Load global ignore config
        if let Some(home) = dirs::home_dir() {
            let global_config = home.join(".config").join("rune").join("ignore.yml");
            if global_config.exists() {
                if let Ok(global) = IgnoreConfig::load_from_file(&global_config) {
                    config.global.extend(global.global);
                }
            }
        }

        // Load project ignore config
        let project_config = project_root.join(".runeignore.yml");
        if project_config.exists() {
            if let Ok(project) = IgnoreConfig::load_from_file(&project_config) {
                config.project = project.project;
                config.templates.extend(project.templates);
            }
        }

        // Auto-detect project type and apply templates
        Self::auto_detect_and_apply_templates(&mut config, &project_root)?;

        let mut engine = Self {
            config,
            compiled_patterns: HashMap::new(),
            cache: HashMap::new(),
            project_root,
        };

        // Pre-compile patterns for performance
        if engine.config.performance.precompile_patterns {
            engine.precompile_patterns()?;
        }

        Ok(engine)
    }

    /// Auto-detect project type and apply appropriate templates
    fn auto_detect_and_apply_templates(
        config: &mut IgnoreConfig,
        project_root: &Path,
    ) -> Result<()> {
        let templates = Self::get_builtin_templates();
        
        for template in &templates {
            for detector in &template.auto_detect {
                let detector_path = project_root.join(detector);
                if detector_path.exists() {
                    if !config.templates.contains(&template.name) {
                        config.templates.push(template.name.clone());
                        
                        // Add template patterns as project rules
                        for pattern in &template.patterns {
                            config.project.push(IgnoreRule {
                                pattern: pattern.clone(),
                                rule_type: RuleType::Ignore,
                                priority: 50, // Medium priority for template rules
                                description: Some(format!("Auto-detected: {}", template.description)),
                                condition: None,
                            });
                        }
                    }
                    break;
                }
            }
        }
        
        Ok(())
    }

    /// Get built-in project templates
    fn get_builtin_templates() -> Vec<IgnoreTemplate> {
        vec![
            IgnoreTemplate {
                name: "rust".to_string(),
                description: "Rust project".to_string(),
                patterns: vec![
                    "target/".to_string(),
                    "Cargo.lock".to_string(),
                    "**/*.rs.bk".to_string(),
                    "**/*.pdb".to_string(),
                ],
                auto_detect: vec!["Cargo.toml".to_string(), "src/main.rs".to_string()],
            },
            IgnoreTemplate {
                name: "node".to_string(),
                description: "Node.js project".to_string(),
                patterns: vec![
                    "node_modules/".to_string(),
                    "npm-debug.log*".to_string(),
                    "yarn-debug.log*".to_string(),
                    "yarn-error.log*".to_string(),
                    ".npm".to_string(),
                    ".yarn-integrity".to_string(),
                ],
                auto_detect: vec!["package.json".to_string(), "yarn.lock".to_string()],
            },
            IgnoreTemplate {
                name: "python".to_string(),
                description: "Python project".to_string(),
                patterns: vec![
                    "__pycache__/".to_string(),
                    "*.py[cod]".to_string(),
                    "*$py.class".to_string(),
                    "*.so".to_string(),
                    ".Python".to_string(),
                    "build/".to_string(),
                    "develop-eggs/".to_string(),
                    "dist/".to_string(),
                    "downloads/".to_string(),
                    "eggs/".to_string(),
                    ".eggs/".to_string(),
                    "lib/".to_string(),
                    "lib64/".to_string(),
                    "parts/".to_string(),
                    "sdist/".to_string(),
                    "var/".to_string(),
                    "wheels/".to_string(),
                    "*.egg-info/".to_string(),
                    ".installed.cfg".to_string(),
                    "*.egg".to_string(),
                    "MANIFEST".to_string(),
                ],
                auto_detect: vec!["setup.py".to_string(), "requirements.txt".to_string(), "pyproject.toml".to_string()],
            },
            IgnoreTemplate {
                name: "java".to_string(),
                description: "Java project".to_string(),
                patterns: vec![
                    "*.class".to_string(),
                    "*.log".to_string(),
                    "*.ctxt".to_string(),
                    ".mtj.tmp/".to_string(),
                    "*.jar".to_string(),
                    "*.war".to_string(),
                    "*.nar".to_string(),
                    "*.ear".to_string(),
                    "*.zip".to_string(),
                    "*.tar.gz".to_string(),
                    "*.rar".to_string(),
                    "hs_err_pid*".to_string(),
                    "target/".to_string(),
                    ".gradle/".to_string(),
                    "build/".to_string(),
                ],
                auto_detect: vec!["pom.xml".to_string(), "build.gradle".to_string(), "gradle.properties".to_string()],
            },
            IgnoreTemplate {
                name: "dotnet".to_string(),
                description: ".NET project".to_string(),
                patterns: vec![
                    "bin/".to_string(),
                    "obj/".to_string(),
                    "*.user".to_string(),
                    "*.suo".to_string(),
                    "*.userosscache".to_string(),
                    "*.sln.docstates".to_string(),
                    ".vs/".to_string(),
                    "packages/".to_string(),
                ],
                auto_detect: vec!["*.csproj".to_string(), "*.sln".to_string(), "*.fsproj".to_string()],
            },
        ]
    }

    /// Pre-compile regex patterns for better performance
    fn precompile_patterns(&mut self) -> Result<()> {
        let all_rules = self.config.global.iter().chain(self.config.project.iter());
        
        for rule in all_rules {
            if !self.compiled_patterns.contains_key(&rule.pattern) {
                match Self::pattern_to_regex(&rule.pattern) {
                    Ok(regex) => {
                        self.compiled_patterns.insert(rule.pattern.clone(), regex);
                    }
                    Err(e) => {
                        eprintln!("Warning: Invalid pattern '{}': {}", rule.pattern, e);
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Convert simplified pattern syntax to regex
    fn pattern_to_regex(pattern: &str) -> Result<Regex, regex::Error> {
        let mut regex_pattern = String::new();
        
        // Start with line beginning
        regex_pattern.push('^');
        
        let chars: Vec<char> = pattern.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            match chars[i] {
                '*' => {
                    if i + 1 < chars.len() && chars[i + 1] == '*' {
                        // ** matches any number of directories
                        regex_pattern.push_str(".*");
                        i += 2;
                        if i < chars.len() && chars[i] == '/' {
                            i += 1; // Skip the following slash
                        }
                    } else {
                        // * matches anything except directory separator
                        regex_pattern.push_str("[^/]*");
                        i += 1;
                    }
                }
                '?' => {
                    regex_pattern.push_str("[^/]");
                    i += 1;
                }
                '[' => {
                    // Character class - pass through as-is
                    regex_pattern.push('[');
                    i += 1;
                    while i < chars.len() && chars[i] != ']' {
                        regex_pattern.push(chars[i]);
                        i += 1;
                    }
                    if i < chars.len() {
                        regex_pattern.push(']');
                        i += 1;
                    }
                }
                '.' | '^' | '$' | '|' | '(' | ')' | '{' | '}' | '+' | '\\' => {
                    // Escape regex special characters
                    regex_pattern.push('\\');
                    regex_pattern.push(chars[i]);
                    i += 1;
                }
                _ => {
                    regex_pattern.push(chars[i]);
                    i += 1;
                }
            }
        }
        
        // If pattern ends with /, it matches directories and their contents
        if pattern.ends_with('/') {
            regex_pattern.push_str(".*");
        } else {
            // Otherwise, match exact file or directory and its contents
            regex_pattern.push_str("(?:/.*)?");
        }
        
        regex_pattern.push('$');
        
        Regex::new(&regex_pattern)
    }

    /// Check if a path should be ignored
    pub fn should_ignore<P: AsRef<Path>>(&mut self, path: P) -> bool {
        let path = path.as_ref();
        let path_buf = path.to_path_buf();
        
        // Check cache first
        if self.config.performance.enable_cache {
            if let Some(&result) = self.cache.get(&path_buf) {
                return result;
            }
        }
        
        let result = self.should_ignore_uncached_impl(path);
        
        // Update cache
        if self.config.performance.enable_cache {
            if self.cache.len() >= self.config.performance.cache_size {
                self.cache.clear(); // Simple cache eviction
            }
            self.cache.insert(path_buf, result);
        }
        
        result
    }

    /// Check if a path should be ignored (without caching)
    /// Check if a path should be ignored without caching (for testing)
    #[cfg(test)]
    pub fn should_ignore_uncached(&self, path: &Path) -> bool {
        self.should_ignore_uncached_impl(path)
    }

    /// Internal implementation for ignore checking
    fn should_ignore_uncached_impl(&self, path: &Path) -> bool {
        // Convert absolute path to relative path from project root
        let relative_path = if path.is_absolute() {
            match path.strip_prefix(&self.project_root) {
                Ok(rel) => rel.to_string_lossy().to_string(),
                Err(_) => path.to_string_lossy().to_string(),
            }
        } else {
            path.to_string_lossy().to_string()
        };
        
        // Get all applicable rules, sorted by priority (highest first)
        let mut applicable_rules = Vec::new();
        
        for rule in self.config.global.iter().chain(self.config.project.iter()) {
            if self.rule_matches(rule, &relative_path) {
                applicable_rules.push(rule);
            }
        }
        
        // Sort by priority (highest first)
        applicable_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        // Apply rules in priority order
        for rule in applicable_rules {
            match rule.rule_type {
                RuleType::Ignore => return true,
                RuleType::Include => return false,
                RuleType::Conditional => {
                    // TODO: Implement conditional logic
                    continue;
                }
            }
        }
        
        // Default to not ignoring
        false
    }

    /// Check if a rule matches a path
    fn rule_matches(&self, rule: &IgnoreRule, path: &str) -> bool {
        if let Some(regex) = self.compiled_patterns.get(&rule.pattern) {
            regex.is_match(path)
        } else {
            // Fallback to simple pattern matching
            self.simple_pattern_match(&rule.pattern, path)
        }
    }

    /// Simple pattern matching fallback
    fn simple_pattern_match(&self, pattern: &str, path: &str) -> bool {
        // Very basic implementation - can be improved
        if pattern.contains("**") {
            let parts: Vec<&str> = pattern.split("**").collect();
            if parts.len() == 2 {
                return path.starts_with(parts[0]) && path.ends_with(parts[1]);
            }
        }
        
        if pattern.ends_with("*") {
            let prefix = &pattern[..pattern.len() - 1];
            return path.starts_with(prefix);
        }
        
        if pattern.starts_with("*") {
            let suffix = &pattern[1..];
            return path.ends_with(suffix);
        }
        
        pattern == path
    }

    /// Get debug information about why a path was ignored/included
    pub fn debug_path<P: AsRef<Path>>(&self, path: P) -> IgnoreDebugInfo {
        let path = path.as_ref();
        let path_str = path.to_string_lossy().to_string();
        
        let mut matched_rules = Vec::new();
        let mut final_decision = false;
        let mut decision_rule = None;
        
        // Check all rules
        for (source, rules) in [("global", &self.config.global), ("project", &self.config.project)] {
            for rule in rules {
                if self.rule_matches(rule, &path_str) {
                    matched_rules.push(DebugRuleMatch {
                        source: source.to_string(),
                        rule: rule.clone(),
                        matched: true,
                    });
                }
            }
        }
        
        // Sort by priority and determine final decision
        matched_rules.sort_by(|a, b| b.rule.priority.cmp(&a.rule.priority));
        
        for rule_match in &matched_rules {
            match rule_match.rule.rule_type {
                RuleType::Ignore => {
                    final_decision = true;
                    decision_rule = Some(rule_match.clone());
                    break;
                }
                RuleType::Include => {
                    final_decision = false;
                    decision_rule = Some(rule_match.clone());
                    break;
                }
                RuleType::Conditional => continue,
            }
        }
        
        IgnoreDebugInfo {
            path: path_str,
            ignored: final_decision,
            matched_rules,
            decision_rule,
        }
    }

    /// Get list of active templates
    pub fn get_active_templates(&self) -> &[String] {
        &self.config.templates
    }

    /// Get global ignore rules
    pub fn get_global_rules(&self) -> &[IgnoreRule] {
        &self.config.global
    }

    /// Get project ignore rules  
    pub fn get_project_rules(&self) -> &[IgnoreRule] {
        &self.config.project
    }

    /// Add a custom ignore rule
    pub fn add_rule(&mut self, rule: IgnoreRule) {
        self.config.project.push(rule);
        // Clear cache since rules changed
        self.cache.clear();
    }

    /// Save current configuration to project
    pub fn save_config(&self) -> Result<()> {
        let config_path = self.project_root.join(".runeignore.yml");
        self.config.save_to_file(config_path)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct IgnoreDebugInfo {
    pub path: String,
    pub ignored: bool,
    pub matched_rules: Vec<DebugRuleMatch>,
    pub decision_rule: Option<DebugRuleMatch>,
}

#[derive(Debug, Clone)]
pub struct DebugRuleMatch {
    pub source: String,
    pub rule: IgnoreRule,
    pub matched: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_pattern_to_regex() {
        let regex = IgnoreEngine::pattern_to_regex("*.txt").unwrap();
        assert!(regex.is_match("test.txt"));
        assert!(regex.is_match("file.txt"));
        assert!(!regex.is_match("test.rs"));
        assert!(!regex.is_match("dir/test.txt")); // Should not match in subdirectory
    }

    #[test]
    fn test_recursive_pattern() {
        let regex = IgnoreEngine::pattern_to_regex("**/*.log").unwrap();
        assert!(regex.is_match("test.log"));
        assert!(regex.is_match("dir/test.log"));
        assert!(regex.is_match("deep/nested/dir/test.log"));
        assert!(!regex.is_match("test.txt"));
    }

    #[test]
    fn test_directory_pattern() {
        let regex = IgnoreEngine::pattern_to_regex("target/").unwrap();
        assert!(regex.is_match("target/"));
        assert!(regex.is_match("target/debug"));
        assert!(regex.is_match("target/release/file"));
        assert!(!regex.is_match("src/target.rs"));
    }

    #[test]
    fn test_auto_detection() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        
        // Create a Cargo.toml to trigger Rust template
        fs::write(project_root.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        
        let engine = IgnoreEngine::new(project_root).unwrap();
        assert!(engine.get_active_templates().contains(&"rust".to_string()));
        
        // Test that target/ is ignored
        assert!(engine.should_ignore_uncached(&project_root.join("target/debug/test")));
    }

    #[test]
    fn test_priority_system() {
        let temp_dir = TempDir::new().unwrap();
        let mut engine = IgnoreEngine::new(temp_dir.path()).unwrap();
        
        // Add high priority include rule
        engine.add_rule(IgnoreRule {
            pattern: "important.txt".to_string(),
            rule_type: RuleType::Include,
            priority: 200,
            description: Some("Important file".to_string()),
            condition: None,
        });
        
        // Add lower priority ignore rule
        engine.add_rule(IgnoreRule {
            pattern: "*.txt".to_string(),
            rule_type: RuleType::Ignore,
            priority: 50,
            description: Some("Text files".to_string()),
            condition: None,
        });
        
        // Include rule should win due to higher priority
        assert!(!engine.should_ignore_uncached(Path::new("important.txt")));
        assert!(engine.should_ignore_uncached(Path::new("other.txt")));
    }
}
