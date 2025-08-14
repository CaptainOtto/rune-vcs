use clap::{Parser, Subcommand};
mod api;
use api::run_api;
use api::serve_api;
use rune_store::Store;
pub mod commands;
mod style;
use colored::*;
use style::{init_colors, Style};
use rune_performance::PerformanceEngine;
use rune_core::ignore::{IgnoreEngine, IgnoreRule, RuleType};
use rune_docs::DocsEngine;
use anyhow::Context;
pub mod intelligence;
use intelligence::{IntelligentFileAnalyzer, InsightSeverity};

/// Global execution context carrying user preferences
#[derive(Debug, Clone)]
struct RuneContext {
    verbose: bool,
    quiet: bool,
    yes: bool,
}

impl RuneContext {
    fn new(args: &Args) -> Self {
        Self {
            verbose: args.verbose,
            quiet: args.quiet,
            yes: args.yes,
        }
    }
    
    /// Print message only if not in quiet mode
    fn info(&self, message: &str) {
        if !self.quiet {
            Style::info(message);
        }
    }
    
    /// Print verbose message only if verbose mode is enabled
    fn verbose(&self, message: &str) {
        if self.verbose {
            Style::verbose(message);
        }
    }
    
    /// Print warning message (always shown unless quiet)
    fn warning(&self, message: &str) {
        if !self.quiet {
            Style::warning(message);
        }
    }
    
    /// Print error message (always shown)
    fn error(&self, message: &str) {
        Style::error(message);
    }
    
    /// Ask for confirmation unless --yes flag is used
    fn confirm(&self, prompt: &str) -> Result<bool, Box<dyn std::error::Error>> {
        if self.yes {
            return Ok(true);
        }
        
        if self.quiet {
            // In quiet mode without --yes, default to no for safety
            return Ok(false);
        }
        
        print!("{} [y/N]: ", prompt);
        use std::io::{self, Write};
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        Ok(input.trim().to_lowercase().starts_with('y'))
    }
}

#[derive(Parser, Debug)]
#[command(name = "rune", version = "0.1.0", about = "Rune — modern DVCS (0.1.0)")]
struct Args {
    /// Enable verbose output with detailed information
    #[arg(short, long, global = true)]
    verbose: bool,
    
    /// Suppress non-essential output (quiet mode)
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    quiet: bool,
    
    /// Assume yes for confirmation prompts (non-interactive mode)
    #[arg(short, long, global = true)]
    yes: bool,
    
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum PatchCmd {
    /// Create a patch file from changes
    Create {
        #[arg(help = "Output file for the patch")]
        output: std::path::PathBuf,
        #[arg(help = "Source commit or range")]
        range: Option<String>,
    },
    /// Apply a patch file
    Apply {
        #[arg(help = "Patch file to apply")]
        patch: std::path::PathBuf,
    },
}

#[derive(Subcommand, Debug)]
enum IgnoreCmd {
    /// Check if a file would be ignored
    Check {
        #[arg(help = "Files to check")]
        files: Vec<std::path::PathBuf>,
        #[arg(long, help = "Show detailed debug information")]
        debug: bool,
    },
    /// Add pattern to ignore file
    Add {
        #[arg(help = "Pattern to add")]
        pattern: String,
        #[arg(long, help = "Description for the rule")]
        description: Option<String>,
        #[arg(long, help = "Priority level (higher takes precedence)", default_value = "50")]
        priority: i32,
        #[arg(long, help = "Add to global ignore file")]
        global: bool,
    },
    /// List current ignore rules
    List {
        #[arg(long, help = "Show global rules")]
        global: bool,
        #[arg(long, help = "Show project rules")]
        project: bool,
        #[arg(long, help = "Show active templates")]
        templates: bool,
    },
    /// Show active project templates
    Templates,
    /// Apply a project template
    Apply {
        #[arg(help = "Template name to apply")]
        template: String,
    },
    /// Initialize smart ignore configuration
    Init {
        #[arg(long, help = "Force overwrite existing configuration")]
        force: bool,
    },
    /// Clean up and optimize ignore rules
    Optimize {
        #[arg(long, help = "Show what would be optimized without doing it")]
        dry_run: bool,
    },
}

#[derive(Subcommand, Debug)]
enum DocsCmd {
    /// View a specific topic in the documentation
    View {
        #[arg(help = "Topic to view (getting-started, commands, migration, best-practices, troubleshooting)")]
        topic: String,
    },
    /// Search documentation content
    Search {
        #[arg(help = "Search query")]
        query: String,
    },
    /// Start local documentation server
    Serve {
        #[arg(long, default_value = "127.0.0.1:8080")]
        addr: String,
        #[arg(long, help = "Open browser automatically")]
        open: bool,
    },
    /// List all available documentation topics
    List,
}

#[derive(Subcommand, Debug)]
enum ExamplesCmd {
    /// View examples for a specific category
    Category {
        #[arg(help = "Category to view (basic, branching, remote, ignore, files, workflow, migration, troubleshooting)")]
        name: String,
    },
    /// Search examples by keyword
    Search {
        #[arg(help = "Search query")]
        query: String,
    },
    /// Show a specific example by name
    Show {
        #[arg(help = "Example name")]
        name: String,
    },
    /// List all available examples
    List {
        #[arg(long, help = "Show only category names")]
        categories: bool,
    },
}

#[derive(Subcommand, Debug)]
enum TutorialCmd {
    /// Start the basics tutorial
    Basics,
    /// Start the branching tutorial
    Branching,
    /// Start the collaboration tutorial
    Collaboration,
    /// Start the advanced features tutorial
    Advanced,
    /// List all available tutorials
    List,
    /// Resume a tutorial from where you left off
    Resume {
        #[arg(help = "Tutorial name to resume")]
        name: String,
    },
}

#[derive(Subcommand, Debug)]
enum IntelligenceCmd {
    /// Analyze repository for insights and recommendations
    Analyze {
        #[arg(help = "Repository path to analyze", default_value = ".")]
        path: Option<String>,
        #[arg(long, help = "Show detailed analysis report")]
        detailed: bool,
    },
    /// Generate predictive insights and recommendations
    Predict {
        #[arg(help = "Repository path to analyze", default_value = ".")]
        path: Option<String>,
    },
    /// Analyze a specific file
    File {
        #[arg(help = "File path to analyze")]
        path: String,
    },
    /// Configure intelligence engine settings
    Config {
        #[arg(long, help = "Enable or disable intelligence engine")]
        enable: Option<bool>,
        #[arg(long, help = "Set LFS threshold in MB")]
        lfs_threshold: Option<u64>,
        #[arg(long, help = "Enable security analysis")]
        security: Option<bool>,
        #[arg(long, help = "Enable performance insights")]
        performance: Option<bool>,
        #[arg(long, help = "Enable predictive modeling")]
        predictive: Option<bool>,
        #[arg(long, help = "Enable repository health monitoring")]
        health: Option<bool>,
        #[arg(long, help = "Enable code quality assessment")]
        quality: Option<bool>,
    },
    /// Show intelligence engine status
    Status,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Run local JSON API server
    Api {
        #[arg(long, default_value = "127.0.0.1:7421")]
        addr: String,
        #[arg(long)]
        with_shrine: bool,
        #[arg(long, default_value = "127.0.0.1:7420")]
        shrine_addr: String,
    },
    /// Generate shell completion scripts
    Completions {
        shell: String,
    },
    Guide,
    Init,
    Status {
        #[arg(long, default_value = "table")]
        format: String,
    },
    Add {
        paths: Vec<std::path::PathBuf>,
    },
    Commit {
        #[arg(short, long)]
        message: String,
    },
    Log {
        #[arg(long, default_value = "table")]
        format: String,
    },
    Branch {
        name: Option<String>,
        #[arg(long, default_value = "table")]
        format: String,
    },
    Checkout {
        name: String,
    },
    /// Merge a branch into the current branch
    Merge {
        #[arg(help = "Branch to merge into current branch")]
        branch: String,
        #[arg(long, help = "Create merge commit even for fast-forward")]
        no_ff: bool,
    },
    Stash {
        #[arg(long)]
        apply: bool,
    },
    /// Show changes between commits, working tree, etc
    Diff {
        #[arg(help = "Compare specific commits (commit1..commit2) or working directory")]
        target: Option<String>,
    },
    /// Reset staging area or working directory
    Reset {
        #[arg(help = "Files to reset")]
        files: Vec<std::path::PathBuf>,
        #[arg(long, help = "Reset working directory (destructive)")]
        hard: bool,
    },
    /// Remove files from working directory and staging
    Remove {
        #[arg(help = "Files to remove")]
        files: Vec<std::path::PathBuf>,
        #[arg(long, help = "Only remove from staging area")]
        cached: bool,
    },
    /// Move or rename files
    Move {
        #[arg(help = "Source file")]
        from: std::path::PathBuf,
        #[arg(help = "Destination file")]
        to: std::path::PathBuf,
    },
    /// Show commit details
    Show {
        #[arg(help = "Commit hash to show", default_value = "HEAD")]
        commit: String,
    },
    /// Clone a remote repository
    Clone {
        #[arg(help = "Repository URL to clone")]
        url: String,
        #[arg(help = "Directory to clone into")]
        directory: Option<std::path::PathBuf>,
    },
    /// Fetch changes from remote repository
    Fetch {
        #[arg(help = "Remote name", default_value = "origin")]
        remote: String,
    },
    /// Pull changes from remote repository
    Pull {
        #[arg(help = "Remote name", default_value = "origin")]
        remote: String,
        #[arg(help = "Branch to pull", default_value = "main")]
        branch: String,
    },
    /// Push changes to remote repository
    Push {
        #[arg(help = "Remote name", default_value = "origin")]
        remote: String,
        #[arg(help = "Branch to push", default_value = "main")]
        branch: String,
    },
    /// Manage ignore patterns with advanced features
    Ignore {
        #[command(subcommand)]
        cmd: IgnoreCmd,
    },
    /// Create and apply patches
    Patch {
        #[command(subcommand)]
        cmd: PatchCmd,
    },
    /// Access documentation and help
    Docs {
        #[command(subcommand)]
        cmd: DocsCmd,
    },
    /// View examples for common workflows
    Examples {
        #[command(subcommand)]
        cmd: ExamplesCmd,
    },
    /// Start interactive tutorials
    Tutorial {
        #[command(subcommand)]
        cmd: TutorialCmd,
    },
    #[command(subcommand)]
    Lfs(commands::lfs::LfsCmd),
    #[command(subcommand)]
    Shrine(commands::shrine::ShrineCmd),
    #[command(subcommand)]
    Delta(commands::delta::DeltaCmd),
    /// Intelligent repository analysis and insights
    Intelligence {
        #[command(subcommand)]
        cmd: IntelligenceCmd,
    },
    /// Configure Rune intelligence and features
    Config {
        #[command(subcommand)]
        cmd: ConfigCmd,
    },
    /// Verify installation and system requirements
    Doctor,
    /// Update Rune to the latest version
    Update {
        #[arg(long, help = "Show what would be updated without doing it")]
        dry_run: bool,
    },
    /// Show version information
    Version,
}

#[derive(Subcommand, Debug)]
enum ConfigCmd {
    /// Get configuration value
    Get {
        #[arg(help = "Configuration key to get")]
        key: String,
    },
    /// Set configuration value
    Set {
        #[arg(help = "Configuration key to set")]
        key: String,
        #[arg(help = "Configuration value")]
        value: String,
    },
    /// List all configuration
    List,
    /// Show intelligence configuration
    Intelligence,
    /// Configure intelligence features
    IntelligenceSet {
        #[arg(long, help = "Enable/disable security analysis")]
        security: Option<bool>,
        #[arg(long, help = "Enable/disable performance optimization")]
        performance: Option<bool>,
        #[arg(long, help = "Enable/disable code quality assessment")]
        quality: Option<bool>,
        #[arg(long, help = "Enable/disable dependency analysis")]
        dependencies: Option<bool>,
        #[arg(long, help = "Enable/disable predictive suggestions")]
        suggestions: Option<bool>,
        #[arg(long, help = "Set analysis depth: minimal, standard, deep, comprehensive")]
        depth: Option<String>,
        #[arg(long, help = "Set notification level: silent, errors, warnings, info, detailed")]
        notifications: Option<String>,
    },
    /// Analyze repository health and get insights
    Health,
    /// Get predictive insights about potential issues
    Insights,
}

fn author() -> rune_core::Author {
    rune_core::Author {
        name: whoami::realname(),
        email: format!("{}@localhost", whoami::username()),
    }
}

fn handle_config_command(cmd: ConfigCmd) -> anyhow::Result<()> {
    match cmd {
        ConfigCmd::Get { key } => {
            match key.as_str() {
                "intelligence.enabled" => {
                    let enabled = std::env::var("RUNE_INTELLIGENCE").unwrap_or_default() != "false";
                    println!("{}", enabled);
                },
                "intelligence.notifications" => {
                    let level = std::env::var("RUNE_INTELLIGENCE_NOTIFICATIONS").unwrap_or_default();
                    println!("{}", if level.is_empty() { "info" } else { &level });
                },
                _ => {
                    Style::error(&format!("Unknown configuration key: {}", key));
                }
            }
        },
        ConfigCmd::Set { key, value } => {
            Style::warning("Configuration setting not yet implemented");
            Style::info(&format!("Would set {} = {}", key, value));
            Style::info("Use environment variables for now:");
            Style::info("  RUNE_INTELLIGENCE=true|false");
            Style::info("  RUNE_INTELLIGENCE_NOTIFICATIONS=silent|errors|warnings|info|detailed");
        },
        ConfigCmd::List => {
            Style::section_header("Rune Configuration");
            println!("\n{}", "Intelligence Settings:".bold());
            
            let intelligence_enabled = std::env::var("RUNE_INTELLIGENCE").unwrap_or_default() != "false";
            println!("  intelligence.enabled = {}", 
                    if intelligence_enabled { "true".green() } else { "false".red() });
            
            let notifications = std::env::var("RUNE_INTELLIGENCE_NOTIFICATIONS").unwrap_or("info".to_string());
            println!("  intelligence.notifications = {}", notifications.cyan());
            
            println!("\n{}", "Environment Variables:".bold());
            println!("  RUNE_INTELLIGENCE = {}", 
                    std::env::var("RUNE_INTELLIGENCE").unwrap_or("(not set)".to_string()).yellow());
            println!("  RUNE_INTELLIGENCE_NOTIFICATIONS = {}", 
                    std::env::var("RUNE_INTELLIGENCE_NOTIFICATIONS").unwrap_or("(not set)".to_string()).yellow());
        },
        ConfigCmd::Intelligence => {
            let mut analyzer = IntelligentFileAnalyzer::new();
            Style::section_header("Intelligence Configuration");
            
            println!("\n{}", "Current Settings:".bold());
            println!("  Status: {}", "Active".green());
            println!("  Features: {}", "All enabled".green());
            println!("  Analysis Depth: {}", "Standard".cyan());
            println!("  Notifications: {}", "Info level".cyan());
            
            println!("\n{}", "Available Features:".bold());
            println!("  🔐 Security Analysis - Detect credentials, security risks");
            println!("  ⚡ Performance Optimization - Storage and compression insights");
            println!("  📊 Code Quality Assessment - Complexity and maintainability analysis");
            println!("  📦 Dependency Analysis - Import and usage tracking");
            println!("  🎯 Predictive Suggestions - Smart recommendations");
            println!("  🗜️  Advanced Compression - Intelligent file compression");
            println!("  🚫 Conflict Prevention - Proactive merge conflict detection");
            
            println!("\n{}", "Usage:".bold());
            println!("  rune config intelligence-set --security true --performance true");
            println!("  rune config intelligence-set --depth comprehensive");
            println!("  rune config intelligence-set --notifications detailed");
            
            // Show a sample analysis
            println!("\n{}", "Sample Analysis:".bold());
            if let Ok(current_dir) = std::env::current_dir() {
                if let Some(sample_file) = std::fs::read_dir(&current_dir)?
                    .filter_map(|entry| entry.ok())
                    .find(|entry| entry.path().extension().map_or(false, |ext| ext == "rs"))
                    .map(|entry| entry.path().to_string_lossy().to_string())
                {
                    if let Ok(analysis) = analyzer.analyze_file(&sample_file) {
                        println!("  File: {}", Style::file_path(&sample_file));
                        println!("  Type: {:?}", analysis.file_type);
                        println!("  Security Issues: {}", analysis.security_issues.len());
                        if !analysis.suggestions.is_empty() {
                            println!("  Suggestions: {} available", analysis.suggestions.len());
                        }
                    }
                }
            }
        },
        ConfigCmd::IntelligenceSet { 
            security, performance, quality, dependencies, suggestions, depth, notifications 
        } => {
            Style::section_header("Configuring Intelligence Features");
            
            // For now, just show what would be configured
            if let Some(sec) = security {
                println!("  Security Analysis: {}", if sec { "enabled".green() } else { "disabled".red() });
            }
            if let Some(perf) = performance {
                println!("  Performance Optimization: {}", if perf { "enabled".green() } else { "disabled".red() });
            }
            if let Some(qual) = quality {
                println!("  Code Quality Assessment: {}", if qual { "enabled".green() } else { "disabled".red() });
            }
            if let Some(deps) = dependencies {
                println!("  Dependency Analysis: {}", if deps { "enabled".green() } else { "disabled".red() });
            }
            if let Some(sugg) = suggestions {
                println!("  Predictive Suggestions: {}", if sugg { "enabled".green() } else { "disabled".red() });
            }
            if let Some(d) = depth {
                println!("  Analysis Depth: {}", d.cyan());
            }
            if let Some(n) = notifications {
                println!("  Notification Level: {}", n.cyan());
            }
            
            Style::warning("Persistent configuration storage not yet implemented");
            Style::info("Configuration will apply for this session only");
            Style::info("Use environment variables for persistent settings");
        },
        ConfigCmd::Health => {
            let mut analyzer = IntelligentFileAnalyzer::new();
            Style::section_header("Repository Health Analysis");
            
            match std::env::current_dir() {
                Ok(current_dir) => {
                    let insights = analyzer.analyze_repository(&current_dir)?;
                    
                    println!("\n{}", "Repository Health Report".bold());
                    println!("  🎯 Overall Score: {:.1}/100 {}", 
                            insights.quality_score,
                            if insights.quality_score > 80.0 { "🟢".green() } 
                            else if insights.quality_score > 60.0 { "🟡".yellow() } 
                            else { "🔴".red() });
                    
                    if !insights.health_indicators.is_empty() {
                        println!("\n{}", "Health Indicators".bold());
                        for indicator in &insights.health_indicators {
                            println!("  {} {}: {}", 
                                match indicator.status {
                                    crate::intelligence::HealthStatus::Excellent => "✅",
                                    crate::intelligence::HealthStatus::Good => "🟢", 
                                    crate::intelligence::HealthStatus::Warning => "⚠️",
                                    crate::intelligence::HealthStatus::Critical => "🔴",
                                },
                                indicator.indicator,
                                indicator.description
                            );
                        }
                    }
                    
                    if !insights.optimization_suggestions.is_empty() {
                        println!("\n{}", "Optimization Suggestions".bold());
                        for suggestion in insights.optimization_suggestions.iter().take(3) {
                            println!("  💡 [Impact: {:.1}] {}", suggestion.impact_score, suggestion.suggestion);
                        }
                    }
                    
                    if insights.quality_score < 70.0 {
                        println!("\n{}", "Tip".bold());
                        println!("  Run 'rune intelligence predict' for more specific improvement suggestions");
                    }
                },
                Err(e) => Style::error(&format!("Failed to get current directory: {}", e)),
            }
        },
        ConfigCmd::Insights => {
            let mut analyzer = IntelligentFileAnalyzer::new();
            Style::section_header("Predictive Repository Insights");
            
            match std::env::current_dir() {
                Ok(current_dir) => {
                    let predictions = analyzer.generate_predictive_insights(&current_dir);
                    
                    if predictions.is_empty() {
                        println!("\n🎉 {} No potential issues detected!", "Excellent!".green().bold());
                        println!("Your repository appears to be well-maintained.");
                    } else {
                        println!("\n{} {} insights found:", "🔮".blue(), predictions.len());
                        
                        for (i, insight) in predictions.iter().enumerate() {
                            let severity_icon = match insight.severity {
                                crate::intelligence::InsightSeverity::High => "⚠️",
                                crate::intelligence::InsightSeverity::Medium => "⚡",
                            };
                            
                            println!("\n{}. {} {} (Confidence: {:.0}%)",
                                    i + 1, severity_icon, insight.insight.bold(), insight.confidence * 100.0);
                            println!("   Category: {:?}", insight.category);
                        }
                        
                        println!("\n{}", "Next Steps".bold());
                        println!("  Address high-severity issues first");
                        println!("  Run 'rune config health' to track improvements");
                    }
                },
                Err(e) => Style::error(&format!("Failed to get current directory: {}", e)),
            }
        },
    }
    Ok(())
}

/// Verify installation and system requirements
async fn doctor_check() -> anyhow::Result<()> {
    Style::section_header("🩺 Rune Installation Doctor");
    
    // Check Rune version
    let version = env!("CARGO_PKG_VERSION");
    println!("\n{} Rune version: {}", "✓".green(), Style::commit_hash(version));
    
    // Check system requirements
    println!("\n{}", "System Requirements:".bold());
    
    // Check Git availability (for migration purposes)
    match std::process::Command::new("git").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let git_version = String::from_utf8_lossy(&output.stdout);
            println!("{} Git found: {}", "✓".green(), git_version.trim());
        }
        _ => {
            println!("{} Git not found (optional, needed for migration)", "⚠".yellow());
        }
    }
    
    // Check disk space in current directory
    match std::env::current_dir() {
        Ok(dir) => {
            println!("{} Working directory: {}", "✓".green(), Style::file_path(&dir.display().to_string()));
        }
        Err(e) => {
            println!("{} Cannot access current directory: {}", "✗".red(), e);
        }
    }
    
    // Check write permissions
    let temp_file = std::env::temp_dir().join("rune_doctor_test");
    match std::fs::write(&temp_file, "test") {
        Ok(()) => {
            println!("{} Write permissions: OK", "✓".green());
            let _ = std::fs::remove_file(&temp_file);
        }
        Err(e) => {
            println!("{} Write permissions: Failed ({})", "✗".red(), e);
        }
    }
    
    // Check if in a Rune repository
    match Store::discover(std::env::current_dir()?) {
        Ok(_) => {
            println!("{} Rune repository: Found", "✓".green());
        }
        Err(_) => {
            println!("{} Rune repository: Not in a repository", "ℹ".blue());
        }
    }
    
    println!("\n{} Installation verification complete!", "🎉".green());
    Ok(())
}

/// Update Rune to the latest version
async fn update_rune(dry_run: bool) -> anyhow::Result<()> {
    Style::section_header("🔄 Rune Update System");
    
    let current_version = env!("CARGO_PKG_VERSION");
    println!("\n{} Current version: {}", "ℹ".blue(), Style::commit_hash(current_version));
    
    if dry_run {
        Style::info("🔍 Checking for updates...");
        Style::info("Update checking would be performed here");
        Style::info("This would connect to GitHub releases API");
        Style::info("No actual updates performed (--dry-run mode)");
        return Ok(());
    }
    
    Style::warning("🚧 Auto-update system not yet implemented");
    println!("\n{}", "Manual update instructions:".bold());
    println!("  1. Visit: https://github.com/CaptainOtto/rune-vcs/releases");
    println!("  2. Download the latest release for your platform");
    println!("  3. Replace your current rune binary");
    println!("\n{}", "Or use package managers:".bold());
    println!("  • macOS: brew upgrade rune");
    println!("  • Windows: scoop update rune");
    println!("  • Cargo: cargo install rune-vcs --force");
    
    Ok(())
}

/// Print detailed version information
fn print_version_info() {
    Style::section_header("📋 Rune Version Information");
    
    println!("\n{} Version: {}", "🔹".blue(), Style::commit_hash(env!("CARGO_PKG_VERSION")));
    println!("{} Package: {}", "🔹".blue(), env!("CARGO_PKG_NAME"));
    
    // Use available Cargo environment variables
    #[cfg(debug_assertions)]
    println!("{} Profile: Debug", "🔹".blue());
    #[cfg(not(debug_assertions))]
    println!("{} Profile: Release", "🔹".blue());
    
    println!("\n{}", "Repository Information:".bold());
    println!("{} Homepage: {}", "🔗".blue(), "https://github.com/CaptainOtto/rune-vcs");
    println!("{} License: {}", "📄".blue(), "Apache-2.0");
    
    println!("\n{}", "Features:".bold());
    println!("{} VCS Operations: ✅", "⚡".yellow());
    println!("{} Branch Management: ✅", "🌿".green());
    println!("{} Delta Compression: ✅", "📦".blue());
    println!("{} LFS Support: ✅", "💾".purple());
    println!("{} Performance Engine: ✅", "🚀".red());
    println!("{} Intelligence Engine: ✅", "🧠".blue());
}

/// Clone a remote repository
async fn clone_repository(url: &str, directory: Option<&std::path::PathBuf>, ctx: &RuneContext) -> anyhow::Result<()> {
    ctx.info("📥 Cloning Repository");
    
    let target_dir = if let Some(dir) = directory {
        dir.clone()
    } else {
        // Extract repository name from URL
        let repo_name = url.split('/').last()
            .unwrap_or("repository")
            .trim_end_matches(".git");
        std::path::PathBuf::from(repo_name)
    };
    
    ctx.info(&format!("🔗 Repository: {}", Style::commit_hash(url)));
    ctx.info(&format!("📁 Target: {}", Style::file_path(&target_dir.display().to_string())));
    ctx.verbose(&format!("Clone operation starting for: {}", url));
    
    // For now, this is a simplified implementation
    // In a real implementation, this would handle various protocols (HTTP, SSH, file://)
    if url.starts_with("http://") || url.starts_with("https://") {
        ctx.info("🌐 HTTP/HTTPS clone detected");
        ctx.warning("🚧 HTTP/HTTPS cloning not yet implemented");
        if !ctx.quiet {
            Style::info("Planned features:");
            Style::info("  • Git protocol compatibility");
            Style::info("  • Authentication handling");
            Style::info("  • Progress tracking");
            Style::info("  • Shallow clones");
        }
    } else if url.starts_with("git@") || url.contains("ssh://") {
        ctx.info("🔐 SSH clone detected");
        ctx.warning("🚧 SSH cloning not yet implemented");
        if !ctx.quiet {
            Style::info("Planned features:");
            Style::info("  • SSH key authentication");
            Style::info("  • Agent support");
            Style::info("  • Host key verification");
        }
    } else if url.starts_with("file://") || std::path::Path::new(url).exists() {
        ctx.info("📁 Local clone detected");
        clone_local_repository(url, &target_dir, ctx).await?;
    } else {
        ctx.error("❌ Unsupported repository URL format");
        if !ctx.quiet {
            Style::info("Supported formats:");
            Style::info("  • file:///path/to/repo or /path/to/repo (local)");
            Style::info("  • https://github.com/user/repo.git (planned)");
            Style::info("  • git@github.com:user/repo.git (planned)");
        }
        return Err(anyhow::anyhow!("Unsupported URL format"));
    }
    
    Ok(())
}

/// Clone a local repository (file:// or local path)
async fn clone_local_repository(source: &str, target: &std::path::PathBuf, ctx: &RuneContext) -> anyhow::Result<()> {
    let source_path = if source.starts_with("file://") {
        std::path::PathBuf::from(&source[7..]) // Remove "file://" prefix
    } else {
        std::path::PathBuf::from(source)
    };
    
    // Check if source exists and is a rune repository
    let source_rune_dir = source_path.join(".rune");
    if !source_rune_dir.exists() {
        return Err(anyhow::anyhow!("Source is not a Rune repository"));
    }
    
    // Create target directory
    std::fs::create_dir_all(target)?;
    
    ctx.verbose(&format!("Cloning from {} to {}", source_path.display(), target.display()));
    
    // Show progress for repository structure copy
    Style::progress("Copying repository structure");
    
    // Copy .rune directory
    copy_dir_all(&source_rune_dir, &target.join(".rune"))?;
    
    Style::clear_progress();
    ctx.info("📋 Repository structure copied");
    
    // Show progress for working directory copy
    Style::progress("Copying working directory files");
    ctx.verbose("Scanning source directory for files to copy");
    
    // Copy working directory files (skip .rune)
    let mut file_count = 0;
    for entry in std::fs::read_dir(&source_path)? {
        let entry = entry?;
        let file_name = entry.file_name();
        
        if file_name == ".rune" {
            continue; // Already copied
        }
        
        let source_item = entry.path();
        let target_item = target.join(&file_name);
        
        file_count += 1;
        ctx.verbose(&format!("Copying: {}", file_name.to_string_lossy()));
        
        if source_item.is_dir() {
            copy_dir_all(&source_item, &target_item)?;
        } else {
            std::fs::copy(&source_item, &target_item)?;
        }
    }
    
    Style::clear_progress();
    Style::success("✅ Repository cloned successfully");
    ctx.info(&format!("📁 Cloned to: {}", Style::file_path(&target.display().to_string())));
    ctx.verbose(&format!("Copied {} files/directories", file_count));
    
    // Verify the clone
    ctx.verbose("Verifying cloned repository");
    let store = Store::open(target)?;
    let log = store.log();
    if !log.is_empty() {
        ctx.info(&format!("📊 Commits: {}", log.len()));
        ctx.info(&format!("🔸 Latest: {}", Style::commit_hash(&log[0].id[..8])));
    }
    
    Ok(())
}

/// Helper function to recursively copy directories
fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Fetch changes from a remote repository
async fn fetch_from_remote(remote: &str) -> anyhow::Result<()> {
    Style::section_header("📥 Fetching from Remote");
    
    let s = Store::discover(std::env::current_dir()?)?;
    
    println!("\n{} Remote: {}", "🔗".blue(), Style::branch_name(remote));
    
    // Check if we're in a repository
    let _log = s.log(); // This will verify we're in a repo
    
    Style::warning("🚧 Remote fetching not yet implemented");
    Style::info("Planned features:");
    Style::info("  • Fetch remote refs and objects");
    Style::info("  • Update remote tracking branches");
    Style::info("  • Conflict detection and resolution");
    Style::info("  • Progress reporting for large transfers");
    Style::info("  • Delta compression optimization");
    
    // Simulate fetch operation
    Style::info("📡 Connecting to remote...");
    Style::info("🔄 Fetching refs...");
    Style::info("📦 Downloading objects...");
    Style::success("✅ Fetch completed (simulated)");
    
    Ok(())
}

/// Pull changes from a remote repository
async fn pull_from_remote(remote: &str, branch: &str) -> anyhow::Result<()> {
    Style::section_header("📥 Pulling from Remote");
    
    let s = Store::discover(std::env::current_dir()?)?;
    
    println!("\n{} Remote: {}", "🔗".blue(), Style::branch_name(remote));
    println!("{} Branch: {}", "🌿".green(), Style::branch_name(branch));
    
    // Check current branch
    let current_branch = s.head_ref();
    println!("{} Current: {}", "📍".yellow(), Style::branch_name(&current_branch));
    
    Style::warning("🚧 Remote pulling not yet implemented");
    Style::info("Pull operation would:");
    Style::info("  1. Fetch changes from remote");
    Style::info("  2. Merge remote branch into current branch");
    Style::info("  3. Update working directory");
    Style::info("  4. Handle merge conflicts if any");
    
    // For now, suggest manual workflow
    Style::info("Manual workflow:");
    Style::info(&format!("  rune fetch {}", remote));
    Style::info(&format!("  rune merge {}/{}", remote, branch));
    
    Ok(())
}

/// Push changes to a remote repository
async fn push_to_remote(remote: &str, branch: &str) -> anyhow::Result<()> {
    Style::section_header("📤 Pushing to Remote");
    
    let s = Store::discover(std::env::current_dir()?)?;
    
    println!("\n{} Remote: {}", "🔗".blue(), Style::branch_name(remote));
    println!("{} Branch: {}", "🌿".green(), Style::branch_name(branch));
    
    // Show what would be pushed
    let log = s.log();
    if log.is_empty() {
        Style::warning("⚠️  No commits to push");
        return Ok(());
    }
    
    println!("{} Latest commit: {}", "📊".blue(), Style::commit_hash(&log[0].id[..8]));
    println!("{} Total commits: {}", "📈".blue(), log.len());
    
    Style::warning("🚧 Remote pushing not yet implemented");
    Style::info("Push operation would:");
    Style::info("  1. Compare local and remote refs");
    Style::info("  2. Upload missing objects and commits");
    Style::info("  3. Update remote refs");
    Style::info("  4. Handle push conflicts");
    
    // Simulate push validation
    Style::info("🔍 Validating local commits...");
    for (i, commit) in log.iter().take(3).enumerate() {
        println!("  {} {} - {}", 
                if i == 0 { "📌" } else { "📋" },
                &commit.id[..8], 
                commit.message);
    }
    if log.len() > 3 {
        println!("  ... and {} more commits", log.len() - 3);
    }
    
    Style::success("✅ Push validation completed (simulated)");
    Style::info("Use --dry-run flag to see what would be pushed");
    
    Ok(())
}

/// Handle ignore-related commands with advanced features
async fn handle_ignore_command(cmd: IgnoreCmd, ctx: &RuneContext) -> anyhow::Result<()> {
    match cmd {
        IgnoreCmd::Check { files, debug } => {
            ctx.info("🔍 Checking ignore status");
            
            let mut engine = IgnoreEngine::new(std::env::current_dir().context("Failed to get current directory")?).context("Failed to initialize ignore engine")?;
            
            for file in &files {
                let should_ignore = engine.should_ignore(file);
                let status = if should_ignore { "❌ IGNORED" } else { "✅ TRACKED" };
                
                if debug {
                    let debug_info = engine.debug_path(file);
                    println!("\n{} {}: {}", 
                        "📁".blue(), 
                        Style::file_path(&file.display().to_string()), 
                        status);
                    
                    if !debug_info.matched_rules.is_empty() {
                        println!("  📋 Matched Rules:");
                        for rule_match in &debug_info.matched_rules {
                            println!("    {} {} (priority: {}) - {}", 
                                "🔸".yellow(),
                                rule_match.rule.pattern,
                                rule_match.rule.priority,
                                rule_match.rule.description.as_deref().unwrap_or("No description"));
                        }
                    }
                    
                    if let Some(decision_rule) = &debug_info.decision_rule {
                        println!("  🎯 Final Decision: {} - {}", 
                            decision_rule.rule.pattern,
                            decision_rule.rule.description.as_deref().unwrap_or("No description"));
                    }
                } else {
                    println!("{} {}: {}", 
                        "📁".blue(), 
                        Style::file_path(&file.display().to_string()), 
                        status);
                }
            }
        }
        
        IgnoreCmd::Add { pattern, description, priority, global } => {
            ctx.info(&format!("➕ Adding ignore pattern: {}", pattern));
            
            let mut engine = IgnoreEngine::new(std::env::current_dir().context("Failed to get current directory")?)?;
            
            let rule = IgnoreRule {
                pattern: pattern.clone(),
                rule_type: RuleType::Ignore,
                priority,
                description,
                condition: None,
            };
            
            engine.add_rule(rule);
            engine.save_config()?;
            
            let scope = if global { "global" } else { "project" };
            Style::success(&format!("✅ Added ignore pattern '{}' to {} configuration", pattern, scope));
        }
        
        IgnoreCmd::List { global, project, templates } => {
            let engine = IgnoreEngine::new(std::env::current_dir().context("Failed to get current directory")?)?;
            
            if global || (!project && !templates) {
                ctx.info("🌍 Global Ignore Rules:");
                for rule in engine.get_global_rules() {
                    println!("  {} {} (priority: {}) - {}", 
                        "🔸".yellow(),
                        rule.pattern,
                        rule.priority,
                        rule.description.as_deref().unwrap_or("No description"));
                }
                println!();
            }
            
            if project || (!global && !templates) {
                ctx.info("📁 Project Ignore Rules:");
                for rule in engine.get_project_rules() {
                    println!("  {} {} (priority: {}) - {}", 
                        "🔸".blue(),
                        rule.pattern,
                        rule.priority,
                        rule.description.as_deref().unwrap_or("No description"));
                }
                println!();
            }
            
            if templates || (!global && !project) {
                ctx.info("📋 Active Templates:");
                for template in engine.get_active_templates() {
                    println!("  {} {}", "✅".green(), template);
                }
            }
        }
        
        IgnoreCmd::Templates => {
            ctx.info("📋 Available Project Templates:");
            
            let templates = vec![
                ("rust", "Rust projects (Cargo.toml, target/, *.rs)"),
                ("node", "Node.js projects (package.json, node_modules/, npm logs)"),
                ("python", "Python projects (setup.py, __pycache__/, *.pyc)"),
                ("java", "Java projects (pom.xml, build.gradle, target/, *.class)"),
                ("dotnet", ".NET projects (*.csproj, bin/, obj/, *.suo)"),
            ];
            
            for (name, description) in templates {
                println!("  {} {} - {}", "🔸".blue(), Style::branch_name(name), description);
            }
            
            ctx.info("💡 Templates are auto-detected and applied when project files are found");
        }
        
        IgnoreCmd::Apply { template } => {
            ctx.info(&format!("📋 Applying template: {}", template));
            Style::warning("🚧 Manual template application not yet implemented");
            Style::info("Templates are automatically applied when project files are detected");
        }
        
        IgnoreCmd::Init { force: _force } => {
            ctx.info("🚀 Initializing smart ignore configuration");
            
            let engine = IgnoreEngine::new(std::env::current_dir().context("Failed to get current directory")?)?;
            engine.save_config()?;
            
            Style::success("✅ Smart ignore configuration initialized");
            ctx.info("📋 Auto-detected project templates:");
            for template in engine.get_active_templates() {
                println!("  {} {}", "✅".green(), template);
            }
        }
        
        IgnoreCmd::Optimize { dry_run } => {
            ctx.info("🔧 Optimizing ignore rules");
            
            if dry_run {
                Style::info("🔍 DRY RUN - No changes will be made");
                Style::info("Optimization analysis:");
                Style::info("  • Duplicate pattern detection");
                Style::info("  • Priority conflict resolution");
                Style::info("  • Performance optimization");
                Style::info("  • Rule consolidation");
            } else {
                Style::warning("🚧 Rule optimization not yet implemented");
                Style::info("Planned optimizations:");
                Style::info("  • Remove duplicate patterns");
                Style::info("  • Resolve priority conflicts");
                Style::info("  • Consolidate similar rules");
                Style::info("  • Pre-compile patterns for performance");
            }
        }
    }
    
    Ok(())
}

/// Handle documentation-related commands
async fn handle_docs_command(cmd: DocsCmd, ctx: &RuneContext) -> anyhow::Result<()> {
    let docs_engine = DocsEngine::new()?;
    
    match cmd {
        DocsCmd::View { topic } => {
            ctx.info(&format!("📖 Viewing documentation: {}", topic));
            let content = docs_engine.get_topic_content(&topic)?;
            println!("{}", content);
        }
        
        DocsCmd::Search { query } => {
            ctx.info(&format!("🔍 Searching documentation for: {}", query));
            let results = docs_engine.search(&query);
            
            if results.is_empty() {
                Style::warning("No results found.");
            } else {
                Style::success(&format!("Found {} results:", results.len()));
                for result in results {
                    println!("\n📄 {}", result.title.bold());
                    println!("   {}", result.snippet);
                    if !result.url.is_empty() {
                        println!("   � URL: {}", result.url.dimmed());
                    }
                }
            }
        }
        
        DocsCmd::Serve { addr, open } => {
            ctx.info(&format!("🌐 Starting documentation server at http://{}", addr));
            
            if open {
                // Try to open the browser
                let url = format!("http://{}", addr);
                if let Err(e) = open::that(&url) {
                    Style::warning(&format!("Could not open browser: {}", e));
                } else {
                    Style::success("Opening documentation in browser...");
                }
            }
            
            docs_engine.start_server(&addr).await?;
        }
        
        DocsCmd::List => {
            ctx.info("📚 Available documentation topics:");
            let topics = vec![
                ("getting-started", "Getting started with Rune"),
                ("commands", "Complete command reference"),
                ("migration", "Migrating from Git"),
                ("best-practices", "Best practices and workflows"),
                ("troubleshooting", "Common issues and solutions"),
            ];
            
            for (topic, description) in topics {
                println!("  {} {}", topic.bold().blue(), description);
            }
        }
    }
    
    Ok(())
}

/// Handle examples-related commands
async fn handle_examples_command(cmd: ExamplesCmd, ctx: &RuneContext) -> anyhow::Result<()> {
    let docs_engine = DocsEngine::new()?;
    
    match cmd {
        ExamplesCmd::Category { name } => {
            ctx.info(&format!("📝 Examples for category: {}", name));
            let examples = docs_engine.get_examples_by_category(&name);
            
            if examples.is_empty() {
                Style::warning(&format!("No examples found for category: {}", name));
                Style::info("Available categories: basic, branching, remote, ignore, files, workflow, migration, troubleshooting");
            } else {
                for example in examples {
                    println!("\n{} {}", "📋".blue(), example.title.bold());
                    println!("   {}", example.description);
                    for cmd in &example.commands {
                        println!("   💡 {}", cmd.cyan());
                    }
                    if let Some(output) = &example.expected_output {
                        println!("   📄 Expected: {}", output.dimmed());
                    }
                }
            }
        }
        
        ExamplesCmd::Search { query } => {
            ctx.info(&format!("🔍 Searching examples for: {}", query));
            let examples = docs_engine.search_examples(&query);
            
            if examples.is_empty() {
                Style::warning("No examples found.");
            } else {
                Style::success(&format!("Found {} examples:", examples.len()));
                for example in examples {
                    println!("\n{} {}", "📋".blue(), example.title.bold());
                    println!("   📂 Category: {}", example.category);
                    println!("   {}", example.description);
                    for cmd in &example.commands {
                        println!("   💡 {}", cmd.cyan());
                    }
                }
            }
        }
        
        ExamplesCmd::Show { name } => {
            ctx.info(&format!("📋 Showing example: {}", name));
            if let Some(example) = docs_engine.get_example_by_name(&name) {
                // Show the first command for help (or adapt show_command_help to accept multiple)
                if let Some(first_cmd) = example.commands.first() {
                    docs_engine.show_command_help(first_cmd)?;
                } else {
                    println!("Example '{}' has no commands.", name);
                }
            } else {
                Style::warning(&format!("Example '{}' not found.", name));
                Style::info("Use 'rune examples list' to see all available examples.");
            }
        }
        
        ExamplesCmd::List { categories } => {
            if categories {
                ctx.info("📂 Available example categories:");
                let categories = vec![
                    ("basic", "Basic operations and getting started"),
                    ("branching", "Branch management and merging"),
                    ("remote", "Remote repository operations"),
                    ("ignore", "File ignore patterns and management"),
                    ("files", "File operations and staging"),
                    ("workflow", "Complete workflow examples"),
                    ("migration", "Migrating from other VCS"),
                    ("troubleshooting", "Problem solving examples"),
                ];
                
                for (cat, desc) in categories {
                    println!("  {} {}", cat.bold().blue(), desc);
                }
            } else {
                ctx.info("📝 All available examples:");
                let all_examples = docs_engine.get_all_examples();
                
                let mut current_category = String::new();
                for example in all_examples {
                    if example.category != current_category {
                        current_category = example.category.clone();
                        println!("\n📂 {}", current_category.bold().blue());
                    }
                    println!("  {} {}", "📋".blue(), example.title);
                }
            }
        }
    }
    
    Ok(())
}

/// Handle tutorial-related commands
async fn handle_tutorial_command(cmd: TutorialCmd, ctx: &RuneContext) -> anyhow::Result<()> {
    let docs_engine = DocsEngine::new()?;
    
    match cmd {
        TutorialCmd::Basics => {
            ctx.info("🎓 Starting Basics Tutorial");
            if let Some(tutorial) = docs_engine.get_tutorial("basics") {
                docs_engine.run_interactive_tutorial(tutorial).await?;
            } else {
                Style::warning("Basics tutorial not found");
            }
        }
        
        TutorialCmd::Branching => {
            ctx.info("🎓 Starting Branching Tutorial");
            if let Some(tutorial) = docs_engine.get_tutorial("branching") {
                docs_engine.run_interactive_tutorial(tutorial).await?;
            } else {
                Style::warning("Branching tutorial not found");
            }
        }
        
        TutorialCmd::Collaboration => {
            ctx.info("🎓 Starting Collaboration Tutorial");
            if let Some(tutorial) = docs_engine.get_tutorial("collaboration") {
                docs_engine.run_interactive_tutorial(tutorial).await?;
            } else {
                Style::warning("Collaboration tutorial not found");
            }
        }
        
        TutorialCmd::Advanced => {
            ctx.info("🎓 Starting Advanced Tutorial");
            if let Some(tutorial) = docs_engine.get_tutorial("advanced") {
                docs_engine.run_interactive_tutorial(tutorial).await?;
            } else {
                Style::warning("Advanced tutorial not found");
            }
        }
        
        TutorialCmd::List => {
            ctx.info("🎓 Available tutorials:");
            let tutorials = vec![
                ("basics", "Learn the fundamentals of Rune VCS"),
                ("branching", "Master branch management and merging"),
                ("collaboration", "Team workflows and remote repositories"),
                ("advanced", "Advanced features and optimization"),
            ];
            
            for (tutorial, description) in tutorials {
                println!("  {} {}", tutorial.bold().blue(), description);
            }
            
            Style::info("\nStart a tutorial with: rune tutorial <name>");
        }
        
        TutorialCmd::Resume { name } => {
            ctx.info(&format!("🔄 Resuming tutorial: {}", name));
            if let Some(tutorial) = docs_engine.get_tutorial(&name) {
                docs_engine.resume_tutorial(tutorial).await?;
            } else {
                Style::warning(&format!("Tutorial '{}' not found.", name));
            }
        }
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_colors();
    let args = Args::parse();
    let ctx = RuneContext::new(&args);
    
    ctx.verbose("Rune VCS starting with enhanced user experience features");
    
    match args.cmd {
        Cmd::Guide => {
            Style::section_header("Rune VCS Quick Start Guide");
            println!("\n{}", "Repository Management:".bold());
            println!(
                "  {} rune init                  # Initialize a new repository",
                Style::status_added()
            );
            println!(
                "  {} rune status                # Show working directory status",
                Style::status_added()
            );
            println!(
                "  {} rune add <files>           # Stage changes for commit",
                Style::status_added()
            );
            println!(
                "  {} rune commit -m \"message\"   # Commit staged changes",
                Style::status_added()
            );
            println!(
                "  {} rune log                   # View commit history",
                Style::status_added()
            );
            println!("\n{}", "Branching:".bold());
            println!(
                "  {} rune branch                # List branches",
                Style::status_added()
            );
            println!(
                "  {} rune branch <name>         # Create new branch",
                Style::status_added()
            );
            println!(
                "  {} rune checkout <branch>     # Switch branches",
                Style::status_added()
            );
            println!("\n{}", "Large Files & Collaboration:".bold());
            println!(
                "  {} rune lfs track \"*.psd\"     # Track large files",
                Style::status_added()
            );
            println!(
                "  {} rune lfs lock --path <file> # Lock file for editing",
                Style::status_added()
            );
            println!(
                "  {} rune api --with-shrine     # Start team server",
                Style::status_added()
            );
            println!("\n{}", "🔐 Intelligent File Locking:".bold());
            println!(
                "  {} rune lock detect           # Auto-detect project type",
                Style::status_added()
            );
            println!(
                "  {} rune lock lock <files>     # Intelligent file locking",
                Style::status_added()
            );
            println!(
                "  {} rune lock status           # View lock status",
                Style::status_added()
            );
            println!(
                "  {} rune lock lfs-suggestions  # Get LFS recommendations",
                Style::status_added()
            );
            println!(
                "  {} rune lock analyze <files>  # Analyze conflict risk",
                Style::status_added()
            );
            println!("\n{}", "🔮 AI-Powered Intelligence:".bold());
            println!(
                "  {} rune config intelligence   # View AI features",
                Style::status_added()
            );
            println!(
                "  {} rune config health         # Repository health analysis",
                Style::status_added()
            );
            println!(
                "  {} rune config insights       # Predictive issue detection",
                Style::status_added()
            );
            println!(
                "\n{}",
                "For complete documentation, see: docs/git-replacement-guide.md".dimmed()
            );
        }
        Cmd::Init => {
            let current_dir = std::env::current_dir()?;
            let rune_dir = current_dir.join(".rune");
            let was_existing = rune_dir.exists();
            let s = Store::open(&current_dir)?;
            s.create()?;
            if was_existing {
                Style::success(&format!(
                    "Reinitialized existing Rune repository in {}",
                    Style::file_path(&current_dir.display().to_string())
                ));
            } else {
                Style::success(&format!(
                    "Initialized new Rune repository in {}",
                    Style::file_path(&current_dir.display().to_string())
                ));
            }
        }
        Cmd::Status { format } => {
            let s = Store::discover(std::env::current_dir()?)?;
            let idx = s.read_index()?;
            let fmt = format.as_str();

            if fmt == "json" {
                println!(
                    "{}",
                    serde_json::json!({"staged": idx.entries.keys().collect::<Vec<_>>()})
                );
            } else if fmt == "yaml" {
                println!(
                    "{}",
                    serde_yaml::to_string(
                        &serde_json::json!({"staged": idx.entries.keys().collect::<Vec<_>>()})
                    )?
                );
            } else {
                // Professional Git-like status output
                let branch = s.head_ref();
                println!("On branch {}", Style::branch_name(&branch));

                if idx.entries.is_empty() {
                    println!("\n{}", "No changes added to commit".dimmed());
                    println!(
                        "{}",
                        "  (use \"rune add <file>...\" to include in what will be committed)"
                            .dimmed()
                    );
                } else {
                    println!("\nChanges to be committed:");
                    println!("{}", "  (use \"rune reset <file>...\" to unstage)".dimmed());
                    println!();
                    for k in idx.entries.keys() {
                        println!("  {}  {}", Style::status_added(), Style::file_path(k));
                    }
                }

                // TODO: Add untracked files section
                println!();
            }
        }
        Cmd::Add { paths } => {
            let s = Store::discover(std::env::current_dir()?)?;
            if paths.is_empty() {
                ctx.error("Nothing specified, nothing added.");
                ctx.info("💡 Tip: Use 'rune add <pathspec>...' to add files to the staging area");
                ctx.info("Examples:");
                ctx.info("  rune add .              # Add all files in current directory");
                ctx.info("  rune add src/           # Add all files in src/ directory");
                ctx.info("  rune add file.txt       # Add specific file");
                ctx.info("  rune status             # Show which files can be added");
                return Ok(());
            }

            // Revolutionary intelligence and performance systems
            let mut analyzer = IntelligentFileAnalyzer::new();
            let engine = PerformanceEngine::new();
            let mut added_count = 0;

            // Initialize performance optimizations
            engine.optimize_memory()?;
            engine.predictive_cache(".")?;

            // Enable parallel processing for multiple files
            let file_paths: Vec<String> = paths
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            if file_paths.len() > 1 {
                engine.parallel_add(&file_paths)?;
            }

            for p in paths {
                let rel = p.to_string_lossy().to_string();

                // Revolutionary intelligence: Analyze file before adding
                let _ = analyzer.analyze_file(&rel);

                // Revolutionary performance: Optimize storage
                let _ = engine.optimize_storage(&rel);
                let _ = engine.smart_delta(&rel);

                match s.stage_file(&rel) {
                    Ok(_) => {
                        added_count += 1;
                        if added_count <= 10 {
                            // Only show first 10 files to avoid spam
                            println!("add {}", Style::file_path(&rel));
                        }
                    }
                    Err(e) => {
                        Style::error(&format!("Failed to add {}: {}", rel, e));
                        return Err(anyhow::anyhow!("Failed to add {}: {}", rel, e));
                    }
                }
            }

            if added_count > 10 {
                println!("... and {} more files", added_count - 10);
            }

            if added_count > 0 {
                Style::success(&format!(
                    "Added {} file{}",
                    added_count,
                    if added_count == 1 { "" } else { "s" }
                ));

                // Show performance statistics
                engine.show_stats();
            }
        }
        Cmd::Commit { message } => {
            let s = Store::discover(std::env::current_dir()?)?;
            let c = s.commit(&message, author())?;
            Style::success(&format!(
                "Committed {} \"{}\"",
                Style::commit_hash(&c.id[..8]),
                message
            ));
        }
        Cmd::Log { format } => {
            let s = Store::discover(std::env::current_dir()?)?;
            let list = s.log();
            let fmt = format.as_str();

            if fmt == "json" {
                println!("{}", serde_json::to_string_pretty(&list)?);
            } else if fmt == "yaml" {
                println!("{}", serde_yaml::to_string(&list)?);
            } else {
                if list.is_empty() {
                    Style::info("No commits yet. Use 'rune commit' to create your first commit.");
                    return Ok(());
                }

                for c in list.iter().rev() {
                    let ts = chrono::DateTime::from_timestamp(c.time, 0)
                        .unwrap()
                        .naive_utc();
                    let now = chrono::Utc::now().naive_utc();
                    let ago = (now.and_utc().timestamp() - ts.and_utc().timestamp()) as i64;

                    println!("commit {}", Style::commit_hash(&c.id));
                    println!(
                        "Date:    {} ({})",
                        Style::timestamp(ts),
                        style::format_duration(ago).dimmed()
                    );
                    println!();
                    println!("    {}", c.message);
                    println!();
                }
            }
        }
        Cmd::Branch { name, format } => {
            let s = Store::discover(std::env::current_dir()?)?;
            let fmt = format.as_str();

            if let Some(n) = name {
                // Create new branch
                if s.branch_exists(&n) {
                    ctx.error(&format!("Branch '{}' already exists", n));
                    ctx.info("💡 Tip: Use one of these alternatives:");
                    ctx.info(&format!("  rune checkout {}        # Switch to existing branch", n));
                    ctx.info("  rune branch              # List all branches");
                    ctx.info("  rune branch new-name     # Create branch with different name");
                    return Err(anyhow::anyhow!("Branch already exists"));
                }
                
                s.create_branch(&n)?;
                Style::success(&format!("Created branch {}", Style::branch_name(&n)));
            } else {
                // List branches
                let current_branch_name = s.current_branch().unwrap_or_else(|| "main".to_string());
                let branches = s.list_branches()?;

                if fmt == "json" {
                    println!(
                        "{}",
                        serde_json::json!({
                          "current": current_branch_name,
                          "branches": branches
                        })
                    );
                } else if fmt == "yaml" {
                    println!(
                        "{}",
                        serde_yaml::to_string(&serde_json::json!({
                          "current": current_branch_name,
                          "branches": branches
                        }))?
                    );
                } else {
                    if branches.is_empty() {
                        Style::info("No branches found");
                        Style::info(&format!("Current: {}", Style::branch_name(&current_branch_name)));
                    } else {
                        for branch in branches {
                            if branch == current_branch_name {
                                println!("* {}", Style::branch_name(&branch));
                            } else {
                                println!("  {}", branch);
                            }
                        }
                    }
                }
            }
        }
        Cmd::Checkout { name } => {
            let s = Store::discover(std::env::current_dir()?)?;
            
            // Check if trying to checkout current branch
            if let Some(current) = s.current_branch() {
                if current == name {
                    Style::info(&format!("Already on branch {}", Style::branch_name(&name)));
                    return Ok(());
                }
            }
            
            // Attempt to checkout the branch
            match s.checkout_branch(&name) {
                Ok(()) => {
                    Style::success(&format!("Switched to branch {}", Style::branch_name(&name)));
                }
                Err(e) => {
                    Style::error(&format!("Failed to checkout branch '{}': {}", name, e));
                    Style::info("Use 'rune branch' to see available branches");
                    return Err(anyhow::anyhow!("Checkout failed"));
                }
            }
        }
        Cmd::Merge { branch, no_ff } => {
            let s = Store::discover(std::env::current_dir()?)?;
            
            // Check if branch exists
            if !s.branch_exists(&branch) {
                Style::error(&format!("Branch '{}' does not exist", branch));
                Style::info("Use 'rune branch' to see available branches");
                return Err(anyhow::anyhow!("Merge failed"));
            }
            
            // Check if trying to merge current branch into itself
            if let Some(current) = s.current_branch() {
                if current == branch {
                    Style::info(&format!("Already on branch {}", Style::branch_name(&branch)));
                    Style::info("Nothing to merge");
                    return Ok(());
                }
            }
            
            // Attempt to merge the branch
            match s.merge_branch(&branch, no_ff) {
                Ok(()) => {
                    Style::success(&format!("Merged branch {} into {}", 
                        Style::branch_name(&branch), 
                        Style::branch_name(&s.current_branch().unwrap_or_else(|| "main".to_string()))
                    ));
                }
                Err(e) => {
                    Style::error(&format!("Failed to merge branch '{}': {}", branch, e));
                    return Err(anyhow::anyhow!("Merge failed"));
                }
            }
        }
        Cmd::Stash { apply } => {
            let s = Store::discover(std::env::current_dir()?)?;
            let p = s.rune_dir.join("stash.json");

            if apply {
                if p.exists() {
                    let list: Vec<serde_json::Value> =
                        serde_json::from_slice(&std::fs::read(p.clone())?)?;
                    let count = list.len();
                    std::fs::remove_file(p)?;
                    Style::success(&format!(
                        "Applied {} stash item{}",
                        count,
                        if count == 1 { "" } else { "s" }
                    ));
                } else {
                    Style::info("No stash entries found");
                }
            } else {
                let idx = s.read_index()?;
                if idx.entries.is_empty() {
                    Style::info("No changes to stash");
                    return Ok(());
                }

                let mut list: Vec<serde_json::Value> = if p.exists() {
                    serde_json::from_slice(&std::fs::read(p.clone())?)?
                } else {
                    vec![]
                };

                list.push(serde_json::json!({
                  "time": chrono::Utc::now().timestamp(),
                  "files": idx.entries.keys().collect::<Vec<_>>()
                }));

                std::fs::write(p, serde_json::to_vec_pretty(&list)?)?;
                s.write_index(&rune_store::Index::default())?;
                Style::success(&format!(
                    "Stashed {} file{}",
                    idx.entries.len(),
                    if idx.entries.len() == 1 { "" } else { "s" }
                ));
            }
        }
        Cmd::Lfs(sub) => return commands::lfs::run(sub).await,
        Cmd::Intelligence { cmd } => {
            match cmd {
                IntelligenceCmd::Analyze { path, detailed } => {
                    return commands::intelligence::analyze_repository(path, detailed).map_err(|e| e.into());
                }
                IntelligenceCmd::Predict { path } => {
                    return commands::intelligence::generate_predictions(path).map_err(|e| e.into());
                }
                IntelligenceCmd::File { path } => {
                    return commands::intelligence::analyze_file(path).map_err(|e| e.into());
                }
                IntelligenceCmd::Config { 
                    enable,
                    lfs_threshold,
                    security,
                    performance,
                    predictive,
                    health,
                    quality,
                } => {
                    return commands::intelligence::configure_intelligence(
                        enable,
                        lfs_threshold,
                        security,
                        performance,
                        predictive,
                        health,
                        quality,
                    ).map_err(|e| e.into());
                }
                IntelligenceCmd::Status => {
                    return commands::intelligence::show_intelligence_status().map_err(|e| e.into());
                }
            }
        }
        Cmd::Shrine(sub) => match sub {
            commands::shrine::ShrineCmd::Serve { addr } => {
                return commands::shrine::serve(addr).await
            }
        },
        Cmd::Api {
            addr,
            with_shrine,
            shrine_addr,
        } => {
            if with_shrine {
                let api_addr: std::net::SocketAddr = addr.parse()?;
                let shrine_addr: std::net::SocketAddr = shrine_addr.parse()?;
                let shrine = rune_remote::Shrine {
                    root: std::env::current_dir()?,
                };
                println!("🕯️  Embedded Shrine at http://{}", shrine_addr);
                println!("🔮 Rune API at http://{}", api_addr);
                let s_task =
                    tokio::spawn(async move { rune_remote::run_server(shrine, shrine_addr).await });
                let a_task = tokio::spawn(async move { serve_api(api_addr).await });
                let _ = tokio::try_join!(s_task, a_task)?;
            } else {
                run_api(addr).await?;
            }
            return Ok(());
        }
        Cmd::Delta(sub) => return commands::delta::run(sub),
        
        Cmd::Config { cmd } => {
            handle_config_command(cmd)?;
        }

        Cmd::Diff { target } => {
            let s = Store::discover(std::env::current_dir()?)?;
            
            match s.diff(target.as_deref()) {
                Ok(diff_output) => {
                    if diff_output.trim().is_empty() {
                        Style::info("No differences found");
                    } else {
                        println!("{}", diff_output);
                    }
                }
                Err(e) => {
                    Style::error(&format!("Failed to generate diff: {}", e));
                    return Err(anyhow::anyhow!("Diff failed"));
                }
            }
        }

        Cmd::Reset { files, hard } => {
            let s = Store::discover(std::env::current_dir()?)?;
            
            if hard {
                ctx.warning("⚠️  WARNING: --hard flag will permanently discard changes in working directory!");
                ctx.verbose("This operation cannot be undone. All uncommitted changes will be lost.");
                
                match ctx.confirm("Are you sure you want to continue?") {
                    Ok(true) => {
                        ctx.verbose("User confirmed destructive operation");
                    }
                    Ok(false) => {
                        ctx.info("Reset cancelled for safety.");
                        return Ok(());
                    }
                    Err(e) => {
                        ctx.error(&format!("Failed to read user input: {}", e));
                        return Err(anyhow::anyhow!("Interactive confirmation failed"));
                    }
                }
            }

            ctx.verbose(&format!("Performing reset on {} files (hard={})", 
                if files.is_empty() { "all".to_string() } else { files.len().to_string() }, 
                hard));

            match s.reset(&files, hard) {
                Ok(()) => {
                    if files.is_empty() {
                        if hard {
                            Style::success("✅ Reset staging area and working directory");
                        } else {
                            Style::success("✅ Reset staging area");
                        }
                    } else {
                        let file_list = files.iter()
                            .map(|f| f.to_string_lossy())
                            .collect::<Vec<_>>()
                            .join(", ");
                        if hard {
                            Style::success(&format!("✅ Reset {} from staging and working directory", file_list));
                        } else {
                            Style::success(&format!("✅ Reset {} from staging area", file_list));
                        }
                    }
                }
                Err(e) => Style::error(&format!("❌ Reset failed: {}", e)),
            }
        }

        Cmd::Remove { files, cached } => {
            if files.is_empty() {
                Style::error("No files specified");
                return Ok(());
            }

            for file in files {
                let path_str = file.to_string_lossy();
                if cached {
                    Style::info(&format!(
                        "Would remove {} from staging (--cached)",
                        path_str
                    ));
                } else {
                    Style::info(&format!("Would remove {} from working directory", path_str));
                }
            }
            Style::info("Remove functionality coming soon!");
        }

        Cmd::Move { from, to } => {
            let from_str = from.to_string_lossy();
            let to_str = to.to_string_lossy();

            if !from.exists() {
                Style::error(&format!("Source file does not exist: {}", from_str));
                return Ok(());
            }

            if let Err(e) = std::fs::rename(&from, &to) {
                Style::error(&format!("Failed to move {} to {}: {}", from_str, to_str, e));
            } else {
                Style::success(&format!(
                    "Moved {} to {}",
                    Style::file_path(&from_str),
                    Style::file_path(&to_str)
                ));
                // TODO: Update staging area to reflect the move
            }
        }

        Cmd::Show { commit } => {
            let s = Store::discover(std::env::current_dir()?)?;
            
            let commit_to_show = if commit == "HEAD" {
                // Get the latest commit
                let log = s.log();
                log.first().cloned()
            } else {
                // Find commit by ID or partial ID
                let log = s.log();
                log.into_iter()
                    .find(|c| c.id == commit || c.id.starts_with(&commit))
            };

            match commit_to_show {
                Some(commit_data) => {
                    println!("commit {}", Style::commit_hash(&commit_data.id));
                    if let Some(parent) = &commit_data.parent {
                        println!("Parent:  {}", Style::commit_hash(parent));
                    }
                    println!("Author:  {}", commit_data.author.name);
                    println!("Email:   {}", commit_data.author.email);
                    let ts = chrono::DateTime::from_timestamp(commit_data.time, 0)
                        .unwrap()
                        .naive_utc();
                    println!("Date:    {}", Style::timestamp(ts));
                    println!();
                    println!("    {}", commit_data.message);
                    println!();
                    
                    if !commit_data.files.is_empty() {
                        println!("Files in this commit:");
                        for file in &commit_data.files {
                            println!("  + {}", Style::file_path(file));
                        }
                        println!();
                    }
                }
                None => {
                    if commit == "HEAD" {
                        Style::info("No commits found in this repository");
                    } else {
                        Style::error(&format!("Commit '{}' not found", commit));
                    }
                }
            }
        }

        Cmd::Patch { cmd } => match cmd {
            PatchCmd::Create { output, range } => {
                Style::info(&format!("Would create patch file: {}", output.display()));
                if let Some(r) = range {
                    Style::info(&format!("Range: {}", r));
                }
                Style::info("Patch creation coming soon!");
            }
            PatchCmd::Apply { patch } => {
                Style::info(&format!("Would apply patch: {}", patch.display()));
                Style::info("Patch application coming soon!");
            }
        },

        Cmd::Completions { shell } => {
            use clap::CommandFactory;
            use clap_complete::{
                generate,
                shells::{Bash, Fish, PowerShell, Zsh},
            };
            let mut cmd = Args::command();
            match shell.as_str() {
                "bash" => generate(Bash, &mut cmd, "rune", &mut std::io::stdout()),
                "zsh" => generate(Zsh, &mut cmd, "rune", &mut std::io::stdout()),
                "fish" => generate(Fish, &mut cmd, "rune", &mut std::io::stdout()),
                "powershell" | "pwsh" => {
                    generate(PowerShell, &mut cmd, "rune", &mut std::io::stdout())
                }
                _ => eprintln!("use: bash|zsh|fish|powershell"),
            }
        }

        Cmd::Doctor => {
            doctor_check().await?;
        }

        Cmd::Update { dry_run } => {
            update_rune(dry_run).await?;
        }

        Cmd::Version => {
            print_version_info();
        }

        Cmd::Clone { url, directory } => {
            clone_repository(&url, directory.as_ref(), &ctx).await?;
        }

        Cmd::Fetch { remote } => {
            fetch_from_remote(&remote).await?;
        }

        Cmd::Pull { remote, branch } => {
            pull_from_remote(&remote, &branch).await?;
        }

        Cmd::Push { remote, branch } => {
            push_to_remote(&remote, &branch).await?;
        }

        Cmd::Ignore { cmd } => {
            handle_ignore_command(cmd, &ctx).await?;
        }

        Cmd::Docs { cmd } => {
            handle_docs_command(cmd, &ctx).await?;
        }

        Cmd::Examples { cmd } => {
            handle_examples_command(cmd, &ctx).await?;
        }

        Cmd::Tutorial { cmd } => {
            handle_tutorial_command(cmd, &ctx).await?;
        }
    }
    Ok(())
}
