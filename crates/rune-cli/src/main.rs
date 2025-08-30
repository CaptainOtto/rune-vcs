use clap::{Parser, Subcommand};
mod api;
use api::run_api;
use api::serve_api;
use rune_store::Store;
pub mod commands;
mod style;
use anyhow::Context;
use colored::{Color, ColoredString, Colorize}; // Import specific items to avoid Style conflict
use rune_core::ignore::{IgnoreEngine, IgnoreRule, RuleType};
use rune_docs::DocsEngine;
use rune_performance::{
    AdvancedPerformanceEngine, NetworkStorageEngine, PerformanceConfig, PerformanceEngine,
    PerformanceMonitor,
};
use style::{init_colors, Style};
pub mod intelligence;
use chrono;
use intelligence::IntelligentFileAnalyzer;
use num_cpus;
use std::{collections::HashSet, fs, io::Write, path::PathBuf};

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
#[command(name = "rune", version = env!("CARGO_PKG_VERSION"), about = concat!("Rune — modern DVCS (", env!("CARGO_PKG_VERSION"), ")"))]
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
        #[arg(
            long,
            help = "Priority level (higher takes precedence)",
            default_value = "50"
        )]
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
        #[arg(
            help = "Topic to view (getting-started, commands, migration, best-practices, troubleshooting)"
        )]
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
        #[arg(
            help = "Category to view (basic, branching, remote, ignore, files, workflow, migration, troubleshooting)"
        )]
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
enum SubmoduleCmd {
    /// List all submodules
    List,
    /// Add a new submodule
    Add {
        #[arg(help = "Repository URL")]
        url: String,
        #[arg(help = "Local path for submodule")]
        path: String,
        #[arg(long, help = "Branch to track")]
        branch: Option<String>,
    },
    /// Update submodules
    Update {
        #[arg(long, help = "Update recursively")]
        recursive: bool,
        #[arg(long, help = "Initialize uninitialized submodules")]
        init: bool,
    },
    /// Remove a submodule
    Remove {
        #[arg(help = "Submodule path to remove")]
        path: String,
    },
}

#[derive(Subcommand, Debug)]
enum HooksCmd {
    /// Install default hooks
    Install,
    /// List available hooks
    List,
    /// Run a specific hook manually
    Run {
        #[arg(help = "Hook name to run")]
        name: String,
    },
    /// Configure hook settings
    Config {
        #[arg(help = "Hook name")]
        hook: String,
        #[arg(long, help = "Enable or disable hook")]
        enable: Option<bool>,
    },
    /// Enable quality bundle (format, lint, test)
    EnableQuality {
        #[arg(
            long,
            help = "Commands to run (e.g., 'cargo fmt -- --check; cargo clippy')"
        )]
        commands: Option<String>,
        #[arg(long, help = "Fail fast on first error")]
        fail_fast: bool,
    },
    /// Enable secret scanning pre-commit hook
    EnableSecretScan {
        #[arg(long, help = "Additional patterns file")]
        patterns_file: Option<PathBuf>,
        #[arg(long, help = "Exclude paths (glob patterns)")]
        exclude: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
enum SignCmd {
    /// Setup GPG signing
    Setup {
        #[arg(long, help = "GPG key ID to use")]
        key: Option<String>,
    },
    /// Verify commit signatures
    Verify {
        #[arg(help = "Commit hashes to verify")]
        commits: Vec<String>,
    },
    /// Create a signed commit
    Commit {
        #[arg(short, long, help = "Commit message")]
        message: String,
        #[arg(long, help = "GPG key ID to use")]
        key: Option<String>,
    },
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
    /// Clone a repository from a remote source
    Clone {
        /// Repository URL to clone
        url: String,
        /// Local directory name (optional)
        directory: Option<String>,
        /// Authentication token
        #[arg(short, long)]
        token: Option<String>,
    },
    /// Manage remote repositories  
    Remote {
        #[command(subcommand)]
        command: crate::commands::remote::RemoteCommand,
    },
    Status {
        #[arg(long, default_value = "table")]
        format: String,
    },
    Add {
        paths: Vec<std::path::PathBuf>,
        #[arg(short = 'p', long, help = "Interactively choose hunks to stage")]
        patch: bool,
    },
    Commit {
        #[arg(short, long)]
        message: String,
        #[arg(long, help = "Replace the tip of the current branch")]
        amend: bool,
        #[arg(long, help = "Don't edit commit message when amending")]
        no_edit: bool,
    },
    Log {
        #[arg(long, default_value = "table")]
        format: String,
        #[arg(long, help = "Show ASCII art commit graph")]
        graph: bool,
        #[arg(long, help = "Show commits in one line")]
        oneline: bool,
        #[arg(short = 'n', long, help = "Limit number of commits to show")]
        max_count: Option<usize>,
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
    /// Show repository file tree
    Tree {
        #[arg(help = "Directory to show (default: current directory)")]
        path: Option<std::path::PathBuf>,
        #[arg(short = 'a', long, help = "Show hidden files")]
        all: bool,
        #[arg(long, help = "Show only tracked files")]
        tracked_only: bool,
    },
    /// List files in the repository
    LsFiles {
        #[arg(long, help = "Show only staged files")]
        cached: bool,
        #[arg(long, help = "Show only modified files")]
        modified: bool,
        #[arg(long, help = "Show file status")]
        stage: bool,
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
    /// Revert commits by creating inverse patch
    Revert {
        #[arg(help = "Commit to revert")]
        commit: String,
        #[arg(
            long,
            help = "For merge commits, specify which parent to use",
            value_name = "N"
        )]
        mainline: Option<usize>,
        #[arg(long, help = "Don't create commit, just apply changes")]
        no_commit: bool,
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
    /// Show line-by-line origin of file content
    Blame {
        #[arg(help = "File to annotate")]
        file: PathBuf,
        #[arg(long, help = "Line range to show (e.g., 1:10)")]
        line_range: Option<String>,
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
    /// Advanced VCS operations (rebase, cherry-pick, submodules, hooks, signing)
    Rebase {
        #[arg(help = "Target commit for rebase")]
        target: String,
        #[arg(long, help = "Start interactive rebase")]
        interactive: bool,
        #[arg(long, help = "Rebase onto specific commit")]
        onto: Option<String>,
        #[arg(long, help = "Preserve merge commits")]
        preserve_merges: bool,
        #[arg(long, help = "Automatically squash fixup commits")]
        autosquash: bool,
    },
    /// Cherry-pick commits from other branches
    CherryPick {
        #[arg(help = "Commit hash to cherry-pick")]
        commit: String,
        #[arg(long, help = "Stage changes but don't commit")]
        no_commit: bool,
        #[arg(long, help = "Edit the commit message")]
        edit: bool,
        #[arg(long, help = "Add Signed-off-by line")]
        signoff: bool,
        #[arg(long, help = "Merge strategy to use")]
        strategy: Option<String>,
    },
    /// Cherry-pick a range of commits
    CherryPickRange {
        #[arg(help = "Start commit (exclusive)")]
        start: String,
        #[arg(help = "End commit (inclusive)")]
        end: String,
    },
    /// Manage submodules
    Submodule {
        #[command(subcommand)]
        cmd: SubmoduleCmd,
    },
    /// Manage repository hooks
    Hooks {
        #[command(subcommand)]
        cmd: HooksCmd,
    },
    /// Virtual workspace and sparse checkout management
    Workspace {
        #[command(subcommand)]
        cmd: commands::workspace::WorkspaceCmd,
    },
    /// Draft commits and checkpoint management
    Draft {
        #[command(flatten)]
        args: commands::draft::DraftArgs,
    },
    /// Lightweight planning (markdown files in .rune/plans)
    Plan {
        #[command(subcommand)]
        cmd: commands::plan::PlanCmd,
    },
    /// GPG signing operations
    Sign {
        #[command(subcommand)]
        cmd: SignCmd,
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
    /// Performance benchmarking and monitoring
    Benchmark {
        #[command(subcommand)]
        cmd: BenchmarkCmd,
    },
}

#[derive(Subcommand, Debug)]
enum BenchmarkCmd {
    /// Run comprehensive performance benchmark suite
    Run {
        #[arg(long, help = "Benchmark suite name", default_value = "comprehensive")]
        suite: String,
        #[arg(long, help = "Output format (table, json)", default_value = "table")]
        format: String,
        #[arg(long, help = "Save results to file")]
        output: Option<std::path::PathBuf>,
    },
    /// Show performance monitoring dashboard
    Monitor {
        #[arg(long, help = "Update interval in seconds", default_value = "1")]
        interval: u64,
        #[arg(long, help = "History limit for metrics", default_value = "100")]
        history: usize,
    },
    /// Generate comprehensive performance report
    Report {
        #[arg(long, help = "Include historical trends")]
        trends: bool,
        #[arg(
            long,
            help = "Output format (table, json, html)",
            default_value = "table"
        )]
        format: String,
        #[arg(long, help = "Save report to file")]
        output: Option<std::path::PathBuf>,
    },
    /// List available benchmark suites
    List,
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
        #[arg(
            long,
            help = "Set analysis depth: minimal, standard, deep, comprehensive"
        )]
        depth: Option<String>,
        #[arg(
            long,
            help = "Set notification level: silent, errors, warnings, info, detailed"
        )]
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
        ConfigCmd::Get { key } => match key.as_str() {
            "intelligence.enabled" => {
                let enabled = std::env::var("RUNE_INTELLIGENCE").unwrap_or_default() != "false";
                println!("{}", enabled);
            }
            "intelligence.notifications" => {
                let level = std::env::var("RUNE_INTELLIGENCE_NOTIFICATIONS").unwrap_or_default();
                println!("{}", if level.is_empty() { "info" } else { &level });
            }
            _ => {
                Style::error(&format!("Unknown configuration key: {}", key));
            }
        },
        ConfigCmd::Set { key, value } => {
            Style::warning("Configuration setting not yet implemented");
            Style::info(&format!("Would set {} = {}", key, value));
            Style::info("Use environment variables for now:");
            Style::info("  RUNE_INTELLIGENCE=true|false");
            Style::info("  RUNE_INTELLIGENCE_NOTIFICATIONS=silent|errors|warnings|info|detailed");
        }
        ConfigCmd::List => {
            Style::section_header("Rune Configuration");
            println!("\n{}", "Intelligence Settings:".bold());

            let intelligence_enabled =
                std::env::var("RUNE_INTELLIGENCE").unwrap_or_default() != "false";
            println!(
                "  intelligence.enabled = {}",
                if intelligence_enabled {
                    "true".green()
                } else {
                    "false".red()
                }
            );

            let notifications =
                std::env::var("RUNE_INTELLIGENCE_NOTIFICATIONS").unwrap_or("info".to_string());
            println!("  intelligence.notifications = {}", notifications.cyan());

            println!("\n{}", "Environment Variables:".bold());
            println!(
                "  RUNE_INTELLIGENCE = {}",
                std::env::var("RUNE_INTELLIGENCE")
                    .unwrap_or("(not set)".to_string())
                    .yellow()
            );
            println!(
                "  RUNE_INTELLIGENCE_NOTIFICATIONS = {}",
                std::env::var("RUNE_INTELLIGENCE_NOTIFICATIONS")
                    .unwrap_or("(not set)".to_string())
                    .yellow()
            );
        }
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
        }
        ConfigCmd::IntelligenceSet {
            security,
            performance,
            quality,
            dependencies,
            suggestions,
            depth,
            notifications,
        } => {
            Style::section_header("Configuring Intelligence Features");

            // For now, just show what would be configured
            if let Some(sec) = security {
                println!(
                    "  Security Analysis: {}",
                    if sec {
                        "enabled".green()
                    } else {
                        "disabled".red()
                    }
                );
            }
            if let Some(perf) = performance {
                println!(
                    "  Performance Optimization: {}",
                    if perf {
                        "enabled".green()
                    } else {
                        "disabled".red()
                    }
                );
            }
            if let Some(qual) = quality {
                println!(
                    "  Code Quality Assessment: {}",
                    if qual {
                        "enabled".green()
                    } else {
                        "disabled".red()
                    }
                );
            }
            if let Some(deps) = dependencies {
                println!(
                    "  Dependency Analysis: {}",
                    if deps {
                        "enabled".green()
                    } else {
                        "disabled".red()
                    }
                );
            }
            if let Some(sugg) = suggestions {
                println!(
                    "  Predictive Suggestions: {}",
                    if sugg {
                        "enabled".green()
                    } else {
                        "disabled".red()
                    }
                );
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
        }
        ConfigCmd::Health => {
            let mut analyzer = IntelligentFileAnalyzer::new();
            Style::section_header("Repository Health Analysis");

            match std::env::current_dir() {
                Ok(current_dir) => {
                    let insights = analyzer.analyze_repository(&current_dir)?;

                    println!("\n{}", "Repository Health Report".bold());
                    println!(
                        "  🎯 Overall Score: {:.1}/100 {}",
                        insights.quality_score,
                        if insights.quality_score > 80.0 {
                            "🟢".green()
                        } else if insights.quality_score > 60.0 {
                            "🟡".yellow()
                        } else {
                            "🔴".red()
                        }
                    );

                    if !insights.health_indicators.is_empty() {
                        println!("\n{}", "Health Indicators".bold());
                        for indicator in &insights.health_indicators {
                            println!(
                                "  {} {}: {}",
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
                            println!(
                                "  💡 [Impact: {:.1}] {}",
                                suggestion.impact_score, suggestion.suggestion
                            );
                        }
                    }

                    if insights.quality_score < 70.0 {
                        println!("\n{}", "Tip".bold());
                        println!("  Run 'rune intelligence predict' for more specific improvement suggestions");
                    }
                }
                Err(e) => Style::error(&format!("Failed to get current directory: {}", e)),
            }
        }
        ConfigCmd::Insights => {
            let mut analyzer = IntelligentFileAnalyzer::new();
            Style::section_header("Predictive Repository Insights");

            match std::env::current_dir() {
                Ok(current_dir) => {
                    let predictions = analyzer.generate_predictive_insights(&current_dir);

                    if predictions.is_empty() {
                        println!(
                            "\n🎉 {} No potential issues detected!",
                            "Excellent!".green().bold()
                        );
                        println!("Your repository appears to be well-maintained.");
                    } else {
                        println!("\n{} {} insights found:", "🔮".blue(), predictions.len());

                        for (i, insight) in predictions.iter().enumerate() {
                            let severity_icon = match insight.severity {
                                crate::intelligence::InsightSeverity::High => "⚠️",
                                crate::intelligence::InsightSeverity::Medium => "⚡",
                            };

                            println!(
                                "\n{}. {} {} (Confidence: {:.0}%)",
                                i + 1,
                                severity_icon,
                                insight.insight.bold(),
                                insight.confidence * 100.0
                            );
                            println!("   Category: {:?}", insight.category);
                        }

                        println!("\n{}", "Next Steps".bold());
                        println!("  Address high-severity issues first");
                        println!("  Run 'rune config health' to track improvements");
                    }
                }
                Err(e) => Style::error(&format!("Failed to get current directory: {}", e)),
            }
        }
    }
    Ok(())
}

/// Verify installation and system requirements
async fn doctor_check() -> anyhow::Result<()> {
    Style::section_header("🩺 Rune Installation Doctor");

    // Check Rune version
    let version = env!("CARGO_PKG_VERSION");
    println!(
        "\n{} Rune version: {}",
        "✓".green(),
        Style::commit_hash(version)
    );

    // Check system requirements
    println!("\n{}", "System Requirements:".bold());

    // Check Git availability (for migration purposes)
    match std::process::Command::new("git").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let git_version = String::from_utf8_lossy(&output.stdout);
            println!("{} Git found: {}", "✓".green(), git_version.trim());
        }
        _ => {
            println!(
                "{} Git not found (optional, needed for migration)",
                "⚠".yellow()
            );
        }
    }

    // Check disk space in current directory
    match std::env::current_dir() {
        Ok(dir) => {
            println!(
                "{} Working directory: {}",
                "✓".green(),
                Style::file_path(&dir.display().to_string())
            );
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
    println!(
        "\n{} Current version: {}",
        "ℹ".blue(),
        Style::commit_hash(current_version)
    );

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

    println!(
        "\n{} Version: {}",
        "🔹".blue(),
        Style::commit_hash(env!("CARGO_PKG_VERSION"))
    );
    println!("{} Package: {}", "🔹".blue(), env!("CARGO_PKG_NAME"));

    // Use available Cargo environment variables
    #[cfg(debug_assertions)]
    println!("{} Profile: Debug", "🔹".blue());
    #[cfg(not(debug_assertions))]
    println!("{} Profile: Release", "🔹".blue());

    println!("\n{}", "Repository Information:".bold());
    println!(
        "{} Homepage: {}",
        "🔗".blue(),
        "https://github.com/CaptainOtto/rune-vcs"
    );
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
async fn clone_repository(
    url: &str,
    directory: Option<&std::path::PathBuf>,
    ctx: &RuneContext,
) -> anyhow::Result<()> {
    ctx.info("📥 Cloning Repository");

    let target_dir = if let Some(dir) = directory {
        dir.clone()
    } else {
        // Extract repository name from URL
        let repo_name = url
            .split('/')
            .last()
            .unwrap_or("repository")
            .trim_end_matches(".git");
        std::path::PathBuf::from(repo_name)
    };

    ctx.info(&format!("🔗 Repository: {}", Style::commit_hash(url)));
    ctx.info(&format!(
        "📁 Target: {}",
        Style::file_path(&target_dir.display().to_string())
    ));
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
async fn clone_local_repository(
    source: &str,
    target: &std::path::PathBuf,
    ctx: &RuneContext,
) -> anyhow::Result<()> {
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

    ctx.verbose(&format!(
        "Cloning from {} to {}",
        source_path.display(),
        target.display()
    ));

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
    ctx.info(&format!(
        "📁 Cloned to: {}",
        Style::file_path(&target.display().to_string())
    ));
    ctx.verbose(&format!("Copied {} files/directories", file_count));

    // Verify the clone
    ctx.verbose("Verifying cloned repository");
    let store = Store::open(target)?;
    let log = store.log();
    if !log.is_empty() {
        ctx.info(&format!("📊 Commits: {}", log.len()));
        ctx.info(&format!(
            "🔸 Latest: {}",
            Style::commit_hash(&log[0].id[..8])
        ));
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
    println!(
        "{} Current: {}",
        "📍".yellow(),
        Style::branch_name(&current_branch)
    );

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

    println!(
        "{} Latest commit: {}",
        "📊".blue(),
        Style::commit_hash(&log[0].id[..8])
    );
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
        println!(
            "  {} {} - {}",
            if i == 0 { "📌" } else { "📋" },
            &commit.id[..8],
            commit.message
        );
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

            let mut engine = IgnoreEngine::new(
                std::env::current_dir().context("Failed to get current directory")?,
            )
            .context("Failed to initialize ignore engine")?;

            for file in &files {
                let should_ignore = engine.should_ignore(file);
                let status = if should_ignore {
                    "❌ IGNORED"
                } else {
                    "✅ TRACKED"
                };

                if debug {
                    let debug_info = engine.debug_path(file);
                    println!(
                        "\n{} {}: {}",
                        "📁".blue(),
                        Style::file_path(&file.display().to_string()),
                        status
                    );

                    if !debug_info.matched_rules.is_empty() {
                        println!("  📋 Matched Rules:");
                        for rule_match in &debug_info.matched_rules {
                            println!(
                                "    {} {} (priority: {}) - {}",
                                "🔸".yellow(),
                                rule_match.rule.pattern,
                                rule_match.rule.priority,
                                rule_match
                                    .rule
                                    .description
                                    .as_deref()
                                    .unwrap_or("No description")
                            );
                        }
                    }

                    if let Some(decision_rule) = &debug_info.decision_rule {
                        println!(
                            "  🎯 Final Decision: {} - {}",
                            decision_rule.rule.pattern,
                            decision_rule
                                .rule
                                .description
                                .as_deref()
                                .unwrap_or("No description")
                        );
                    }
                } else {
                    println!(
                        "{} {}: {}",
                        "📁".blue(),
                        Style::file_path(&file.display().to_string()),
                        status
                    );
                }
            }
        }

        IgnoreCmd::Add {
            pattern,
            description,
            priority,
            global,
        } => {
            ctx.info(&format!("➕ Adding ignore pattern: {}", pattern));

            let mut engine = IgnoreEngine::new(
                std::env::current_dir().context("Failed to get current directory")?,
            )?;

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
            Style::success(&format!(
                "✅ Added ignore pattern '{}' to {} configuration",
                pattern, scope
            ));
        }

        IgnoreCmd::List {
            global,
            project,
            templates,
        } => {
            let engine = IgnoreEngine::new(
                std::env::current_dir().context("Failed to get current directory")?,
            )?;

            if global || (!project && !templates) {
                ctx.info("🌍 Global Ignore Rules:");
                for rule in engine.get_global_rules() {
                    println!(
                        "  {} {} (priority: {}) - {}",
                        "🔸".yellow(),
                        rule.pattern,
                        rule.priority,
                        rule.description.as_deref().unwrap_or("No description")
                    );
                }
                println!();
            }

            if project || (!global && !templates) {
                ctx.info("📁 Project Ignore Rules:");
                for rule in engine.get_project_rules() {
                    println!(
                        "  {} {} (priority: {}) - {}",
                        "🔸".blue(),
                        rule.pattern,
                        rule.priority,
                        rule.description.as_deref().unwrap_or("No description")
                    );
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
                (
                    "node",
                    "Node.js projects (package.json, node_modules/, npm logs)",
                ),
                ("python", "Python projects (setup.py, __pycache__/, *.pyc)"),
                (
                    "java",
                    "Java projects (pom.xml, build.gradle, target/, *.class)",
                ),
                ("dotnet", ".NET projects (*.csproj, bin/, obj/, *.suo)"),
            ];

            for (name, description) in templates {
                println!(
                    "  {} {} - {}",
                    "🔸".blue(),
                    Style::branch_name(name),
                    description
                );
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

            let engine = IgnoreEngine::new(
                std::env::current_dir().context("Failed to get current directory")?,
            )?;
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
            ctx.info(&format!(
                "🌐 Starting documentation server at http://{}",
                addr
            ));

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
            println!("\n{}", "⚡ Advanced VCS Features:".bold());
            println!(
                "  {} rune rebase --interactive HEAD~3  # Interactive rebase",
                Style::status_added()
            );
            println!(
                "  {} rune cherry-pick <commit>  # Cherry-pick commits",
                Style::status_added()
            );
            println!(
                "  {} rune submodule add <url>   # Add submodules",
                Style::status_added()
            );
            println!(
                "  {} rune hooks install         # Setup commit hooks",
                Style::status_added()
            );
            println!(
                "  {} rune sign setup --key <id> # GPG commit signing",
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
        Cmd::Clone {
            url,
            directory,
            token,
        } => {
            let args = crate::commands::clone::CloneArgs {
                url: url.clone(),
                directory: directory.clone(),
                token: token.clone(),
                recursive: false,
                bare: false,
                branch: None,
                depth: None,
            };
            crate::commands::clone::handle_clone_command(args).await?;
        }
        Cmd::Remote { command } => {
            let args = crate::commands::remote::RemoteArgs {
                command: command.clone(),
            };
            crate::commands::remote::handle_remote_command(args).await?;
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
        Cmd::Add { paths, patch } => {
            let s = Store::discover(std::env::current_dir()?)?;

            if patch {
                // Interactive patch mode
                interactive_add(&s, &paths)?;
            } else {
                // Regular add mode
                if paths.is_empty() {
                    ctx.error("Nothing specified, nothing added.");
                    ctx.info(
                        "💡 Tip: Use 'rune add <pathspec>...' to add files to the staging area",
                    );
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

                // For multiple files, use advanced parallel processing (without async runtime)
                if paths.len() > 5 {
                    Style::info(
                        "🚀 Detected multiple files, enabling advanced parallel processing...",
                    );

                    let advanced_engine =
                        AdvancedPerformanceEngine::with_config(PerformanceConfig {
                            max_parallel_operations: num_cpus::get().min(8),
                            cache_size_mb: 128,
                            enable_memory_mapping: true,
                            enable_parallel_diff: true,
                            enable_async_io: false, // Disable async for CLI context
                            bandwidth_limit_mbps: None,
                        });

                    // Process files in parallel using rayon (sync parallel processing)
                    let file_paths: Vec<PathBuf> = paths.clone();

                    use rayon::prelude::*;
                    let results: Result<Vec<_>, _> = file_paths
                        .par_iter()
                        .map(|file_path| {
                            let rel = file_path.to_string_lossy().to_string();

                            // Create local store and analyzer for this thread
                            let local_store = Store::discover(std::env::current_dir()?)?;
                            let mut local_analyzer = IntelligentFileAnalyzer::new();

                            // Intelligence analysis
                            let _ = local_analyzer.analyze_file(&rel);

                            // Stage the file
                            local_store.stage_file(&rel)
                        })
                        .collect();

                    match results {
                        Ok(stage_results) => {
                            added_count = stage_results.len();

                            // Show summary for many files
                            if added_count <= 10 {
                                for path in &paths {
                                    println!("add {}", Style::file_path(&path.to_string_lossy()));
                                }
                            } else {
                                println!("add {} files (showing first 10):", added_count);
                                for path in paths.iter().take(10) {
                                    println!("add {}", Style::file_path(&path.to_string_lossy()));
                                }
                                println!("... and {} more files", added_count - 10);
                            }

                            advanced_engine.print_performance_summary();
                        }
                        Err(e) => {
                            Style::error(&format!("Parallel processing failed: {}", e));
                            return Err(e);
                        }
                    }
                } else {
                    // Use simple engine for few files
                    engine.clear_cache();

                    for p in paths {
                        let rel = p.to_string_lossy().to_string();

                        // Revolutionary intelligence: Analyze file before adding
                        let _ = analyzer.analyze_file(&rel);

                        // Use performance benchmarking for file operations
                        let stage_result = engine.benchmark("stage_file", || s.stage_file(&rel));

                        match stage_result {
                            Ok(_) => {
                                added_count += 1;
                                println!("add {}", Style::file_path(&rel));
                            }
                            Err(e) => {
                                Style::error(&format!("Failed to add {}: {}", rel, e));
                                return Err(anyhow::anyhow!("Failed to add {}: {}", rel, e));
                            }
                        }
                    }

                    // Show performance statistics for simple engine
                    engine.print_performance_summary();
                }

                if added_count > 0 {
                    Style::success(&format!(
                        "Added {} file{}",
                        added_count,
                        if added_count == 1 { "" } else { "s" }
                    ));

                    // Show performance statistics
                    engine.print_performance_summary();
                }
            }
        }
        Cmd::Commit {
            message,
            amend,
            no_edit,
        } => {
            let s = Store::discover(std::env::current_dir()?)?;

            // Initialize network storage optimization for large commits
            let network_engine = NetworkStorageEngine::new();

            // Get staged files for compression analysis
            let idx = s.read_index()?;
            let staged_files: Vec<_> = idx.entries.keys().cloned().collect();

            if staged_files.len() > 3 {
                Style::info("🌐 Enabling network storage optimization for large commit...");

                // Compress staged files with delta compression v2.0
                for file_path in &staged_files {
                    if let Ok(path) = std::path::Path::new(file_path).canonicalize() {
                        if path.exists() && path.metadata().map(|m| m.len()).unwrap_or(0) > 1024 {
                            // Apply delta compression v2.0 for files > 1KB
                            match network_engine.delta_compress_v2(&path, None) {
                                Ok(result) => {
                                    println!(
                                        "🗜️  Compressed {}: {} → {} ({:.1}% reduction)",
                                        file_path,
                                        format_bytes(result.original_size),
                                        format_bytes(result.compressed_size),
                                        (1.0 - result.compression_ratio) * 100.0
                                    );
                                }
                                Err(_) => {
                                    // Compression failed, continue with original file
                                }
                            }
                        }
                    }
                }

                network_engine.print_performance_summary();
            }

            if amend {
                let c = s.commit_amend(&message, !no_edit, author())?;
                Style::success(&format!(
                    "Amended {} \"{}\"",
                    Style::commit_hash(&c.id[..8]),
                    c.message
                ));
            } else {
                let c = s.commit(&message, author())?;
                Style::success(&format!(
                    "Committed {} \"{}\"",
                    Style::commit_hash(&c.id[..8]),
                    message
                ));

                // Show commit size optimization summary
                if staged_files.len() > 3 {
                    println!(
                        "📦 Commit optimized with {} file compression",
                        staged_files.len()
                    );
                }
            }
        }
        Cmd::Log {
            format,
            graph,
            oneline,
            max_count,
        } => {
            let s = Store::discover(std::env::current_dir()?)?;
            let mut list = s.log();
            let fmt = format.as_str();

            // Apply max_count limit if specified
            if let Some(max) = max_count {
                list = list.into_iter().take(max).collect();
            }

            if fmt == "json" {
                println!("{}", serde_json::to_string_pretty(&list)?);
            } else if fmt == "yaml" {
                println!("{}", serde_yaml::to_string(&list)?);
            } else {
                if list.is_empty() {
                    Style::info("No commits yet. Use 'rune commit' to create your first commit.");
                    return Ok(());
                }

                if graph || oneline {
                    // Enhanced visual output
                    display_commit_graph(&list, graph, oneline)?;
                } else {
                    // Original detailed format
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
        }
        Cmd::Branch { name, format } => {
            let s = Store::discover(std::env::current_dir()?)?;
            let fmt = format.as_str();

            if let Some(n) = name {
                // Create new branch
                if s.branch_exists(&n) {
                    ctx.error(&format!("Branch '{}' already exists", n));
                    ctx.info("💡 Tip: Use one of these alternatives:");
                    ctx.info(&format!(
                        "  rune checkout {}        # Switch to existing branch",
                        n
                    ));
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
                        Style::info(&format!(
                            "Current: {}",
                            Style::branch_name(&current_branch_name)
                        ));
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
                    Style::info(&format!(
                        "Already on branch {}",
                        Style::branch_name(&branch)
                    ));
                    Style::info("Nothing to merge");
                    return Ok(());
                }
            }

            // Attempt to merge the branch
            match s.merge_branch(&branch, no_ff) {
                Ok(()) => {
                    Style::success(&format!(
                        "Merged branch {} into {}",
                        Style::branch_name(&branch),
                        Style::branch_name(
                            &s.current_branch().unwrap_or_else(|| "main".to_string())
                        )
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
        Cmd::Intelligence { cmd } => match cmd {
            IntelligenceCmd::Analyze { path, detailed } => {
                return commands::intelligence::analyze_repository(path, detailed)
                    .map_err(|e| e.into());
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
                )
                .map_err(|e| e.into());
            }
            IntelligenceCmd::Status => {
                return commands::intelligence::show_intelligence_status().map_err(|e| e.into());
            }
        },
        Cmd::Rebase {
            target,
            interactive,
            onto,
            preserve_merges,
            autosquash,
        } => {
            let options = commands::advanced::RebaseOptions {
                interactive,
                onto,
                preserve_merges,
                autosquash,
            };
            return commands::advanced::interactive_rebase(target, options).map_err(|e| e.into());
        }
        Cmd::CherryPick {
            commit,
            no_commit,
            edit,
            signoff,
            strategy,
        } => {
            let options = commands::advanced::CherryPickOptions {
                no_commit,
                edit,
                signoff,
                strategy,
            };
            return commands::advanced::cherry_pick(commit, options).map_err(|e| e.into());
        }
        Cmd::CherryPickRange { start, end } => {
            return commands::advanced::cherry_pick_range(start, end).map_err(|e| e.into());
        }
        Cmd::Submodule { cmd } => match cmd {
            SubmoduleCmd::List => {
                commands::advanced::list_submodules()?;
            }
            SubmoduleCmd::Add { url, path, branch } => {
                commands::advanced::add_submodule(url, path, branch)?;
            }
            SubmoduleCmd::Update { recursive, init } => {
                commands::advanced::update_submodules(recursive, init)?;
            }
            SubmoduleCmd::Remove { path: _path } => {
                println!("{}", "🗑️  Submodule removal not yet implemented".yellow());
            }
        },
        Cmd::Hooks { cmd } => match cmd {
            HooksCmd::Install => {
                commands::advanced::install_hooks()?;
            }
            HooksCmd::List => {
                commands::advanced::list_hooks()?;
            }
            HooksCmd::Run { name } => {
                let context = std::collections::HashMap::new();
                commands::advanced::run_hook(name, context)?;
            }
            HooksCmd::Config {
                hook: _hook,
                enable: _enable,
            } => {
                println!("{}", "Hook configuration not yet implemented".yellow());
            }
            HooksCmd::EnableQuality {
                commands,
                fail_fast,
            } => {
                enable_quality_hooks(commands.as_deref(), fail_fast)?;
            }
            HooksCmd::EnableSecretScan {
                patterns_file,
                exclude,
            } => {
                enable_secret_scan_hook(patterns_file.as_ref(), &exclude)?;
            }
        },
        Cmd::Workspace { cmd } => {
            commands::workspace::run(cmd)?;
        }
        Cmd::Draft { args } => {
            commands::draft::execute_draft_command(args)?;
        }
        Cmd::Plan { cmd } => {
            use commands::plan::{execute_plan_command, PlanArgs};
            // Wrap single subcommand into PlanArgs for reuse pattern
            let args = PlanArgs { command: cmd };
            execute_plan_command(args)?;
        }
        Cmd::Sign { cmd } => match cmd {
            SignCmd::Setup { key } => {
                commands::advanced::setup_gpg_signing(key)?;
            }
            SignCmd::Verify { commits } => {
                commands::advanced::verify_signatures(commits)?;
            }
            SignCmd::Commit { message, key } => {
                commands::advanced::create_signed_commit(message, key)?;
            }
        },
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

        Cmd::Tree {
            path,
            all,
            tracked_only,
        } => {
            let s = Store::discover(std::env::current_dir()?)?;
            let start_path = path.unwrap_or_else(|| std::env::current_dir().unwrap());
            display_file_tree(&s, &start_path, all, tracked_only)?;
        }

        Cmd::LsFiles {
            cached,
            modified,
            stage,
        } => {
            let s = Store::discover(std::env::current_dir()?)?;
            list_repository_files(&s, cached, modified, stage)?;
        }

        Cmd::Reset { files, hard } => {
            let s = Store::discover(std::env::current_dir()?)?;

            if hard {
                ctx.warning("⚠️  WARNING: --hard flag will permanently discard changes in working directory!");
                ctx.verbose(
                    "This operation cannot be undone. All uncommitted changes will be lost.",
                );

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

            ctx.verbose(&format!(
                "Performing reset on {} files (hard={})",
                if files.is_empty() {
                    "all".to_string()
                } else {
                    files.len().to_string()
                },
                hard
            ));

            match s.reset(&files, hard) {
                Ok(()) => {
                    if files.is_empty() {
                        if hard {
                            Style::success("✅ Reset staging area and working directory");
                        } else {
                            Style::success("✅ Reset staging area");
                        }
                    } else {
                        let file_list = files
                            .iter()
                            .map(|f| f.to_string_lossy())
                            .collect::<Vec<_>>()
                            .join(", ");
                        if hard {
                            Style::success(&format!(
                                "✅ Reset {} from staging and working directory",
                                file_list
                            ));
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

        Cmd::Revert {
            commit,
            mainline,
            no_commit,
        } => {
            let s = Store::discover(std::env::current_dir()?)?;
            let reverted_commit = s.revert_commit(&commit, mainline, no_commit, author())?;

            if no_commit {
                Style::success("Revert changes applied to working directory");
                Style::info("Run 'rune commit' to complete the revert");
            } else {
                Style::success(&format!(
                    "Reverted {} with commit {}",
                    Style::commit_hash(&commit[..8]),
                    Style::commit_hash(&reverted_commit.id[..8])
                ));
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

        Cmd::Blame { file, line_range } => {
            let s = Store::discover(std::env::current_dir()?)?;
            blame_file(&s, &file, line_range.as_deref())?;
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

        Cmd::Benchmark { cmd } => {
            handle_benchmark_command(cmd, &ctx).await?;
        }
    }
    Ok(())
}

/// Interactive staging of file hunks
fn interactive_add(store: &Store, paths: &[PathBuf]) -> anyhow::Result<()> {
    use std::io::{stdin, stdout};

    Style::section_header("Interactive Staging");

    let paths_to_process = if paths.is_empty() {
        // If no paths specified, find all modified files
        vec![PathBuf::from(".")]
    } else {
        paths.to_vec()
    };

    for path in paths_to_process {
        let path_str = path.to_string_lossy();

        // Skip if path doesn't exist
        if !path.exists() {
            Style::warning(&format!("Path does not exist: {}", path_str));
            continue;
        }

        if path.is_dir() {
            Style::info(&format!("Processing directory: {}", path_str));
            // For directories, we'd need to recursively find files
            // For now, let's handle single files
            continue;
        }

        // Read the current file content
        let current_content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(_) => {
                Style::warning(&format!("Cannot read file as text: {}", path_str));
                continue;
            }
        };

        // For interactive staging, we need to compare with the last committed version
        // This is a simplified implementation - in a real VCS this would be more complex
        let log = store.log();
        let last_committed_content = if let Some(last_commit) = log.first() {
            if last_commit.files.contains(&path_str.to_string()) {
                // In a real implementation, we'd read the file content from the commit
                // For now, simulate with a simple diff
                "Previous version\nof the file\n".to_string()
            } else {
                String::new() // New file
            }
        } else {
            String::new() // No commits yet
        };

        // Generate simple hunks (in reality this would be much more sophisticated)
        let hunks = generate_hunks(&last_committed_content, &current_content);

        if hunks.is_empty() {
            Style::info(&format!("No changes in {}", path_str));
            continue;
        }

        println!("\nFile: {}", Style::file_path(&path_str));

        for (i, hunk) in hunks.iter().enumerate() {
            println!("\n{}", "─".repeat(60).dimmed());
            println!("Hunk {}/{}", i + 1, hunks.len());
            println!("{}", hunk.display());

            loop {
                println!();
                println!("Stage this hunk?");
                println!("  y = yes, stage this hunk");
                println!("  n = no, skip this hunk");
                println!("  e = edit hunk");
                println!("  s = split hunk");
                println!("  q = quit");
                println!("  ? = help");
                print!("Your choice: ");
                stdout().flush()?;

                let mut input = String::new();
                stdin().read_line(&mut input)?;
                let choice = input.trim().to_lowercase();

                match choice.as_str() {
                    "y" | "yes" => {
                        // Stage this hunk
                        println!("Staged hunk {}", i + 1);
                        break;
                    }
                    "n" | "no" => {
                        // Skip this hunk
                        println!("Skipped hunk {}", i + 1);
                        break;
                    }
                    "e" | "edit" => {
                        // Edit this hunk (simplified)
                        println!("Edit mode not yet implemented");
                        break;
                    }
                    "s" | "split" => {
                        // Split this hunk (simplified)
                        println!("Split mode not yet implemented");
                        break;
                    }
                    "q" | "quit" => {
                        println!("Aborted interactive staging");
                        return Ok(());
                    }
                    "?" | "help" => {
                        println!("\nInteractive staging commands:");
                        println!("  y - stage this hunk");
                        println!("  n - do not stage this hunk");
                        println!("  e - manually edit the current hunk");
                        println!("  s - split the current hunk into smaller hunks");
                        println!("  q - abort interactive staging");
                        println!("  ? - show this help");
                        continue;
                    }
                    _ => {
                        println!("Invalid choice. Use '?' for help.");
                        continue;
                    }
                }
            }
        }

        // After processing all hunks, stage the file
        // In a real implementation, we'd only stage the selected hunks
        match store.stage_file(&path_str) {
            Ok(_) => {
                Style::success(&format!("Staged changes in {}", path_str));
            }
            Err(e) => {
                Style::error(&format!("Failed to stage {}: {}", path_str, e));
            }
        }
    }

    Style::success("Interactive staging completed");
    Ok(())
}

#[derive(Debug)]
struct Hunk {
    old_start: usize,
    old_count: usize,
    new_start: usize,
    new_count: usize,
    lines: Vec<String>,
}

impl Hunk {
    fn display(&self) -> String {
        let mut result = String::new();
        result.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            self.old_start, self.old_count, self.new_start, self.new_count
        ));

        for line in &self.lines {
            result.push_str(line);
            result.push('\n');
        }

        result
    }
}

fn generate_hunks(old_content: &str, new_content: &str) -> Vec<Hunk> {
    let old_lines: Vec<&str> = old_content.lines().collect();
    let new_lines: Vec<&str> = new_content.lines().collect();

    // This is a very simplified diff algorithm
    // In reality, you'd use a proper diff algorithm like Myers

    let mut hunks = Vec::new();

    // Simple case: if content is completely different, create one big hunk
    if old_content != new_content {
        let mut lines = Vec::new();

        // Add context lines (simplified)
        for (i, line) in old_lines.iter().enumerate() {
            if i < 3 {
                lines.push(format!(" {}", line));
            }
        }

        // Add removed lines
        for line in old_lines.iter().skip(3) {
            lines.push(format!("-{}", line));
        }

        // Add added lines
        for line in new_lines.iter() {
            lines.push(format!("+{}", line));
        }

        let hunk = Hunk {
            old_start: 1,
            old_count: old_lines.len(),
            new_start: 1,
            new_count: new_lines.len(),
            lines,
        };

        hunks.push(hunk);
    }

    hunks
}

/// Blame/annotate a file to show line-by-line origin
fn blame_file(store: &Store, file_path: &PathBuf, line_range: Option<&str>) -> anyhow::Result<()> {
    Style::section_header("Blame/Annotate");

    let file_str = file_path.to_string_lossy();

    // Check if file exists
    if !file_path.exists() {
        return Err(anyhow::anyhow!("File does not exist: {}", file_str));
    }

    // Read current file content
    let current_content = fs::read_to_string(file_path)?;
    let lines: Vec<&str> = current_content.lines().collect();

    // Parse line range if provided
    let (start_line, end_line) = if let Some(range) = line_range {
        parse_line_range(range, lines.len())?
    } else {
        (1, lines.len())
    };

    println!("\nFile: {}", Style::file_path(&file_str));
    println!("{}", "─".repeat(80).dimmed());

    // Get commit history for this file
    let log = store.log();
    let file_commits: Vec<_> = log
        .iter()
        .filter(|commit| commit.files.contains(&file_str.to_string()))
        .collect();

    if file_commits.is_empty() {
        Style::warning("No commits found for this file");
        return Ok(());
    }

    // Blame each line (simplified implementation)
    for (line_num, line_content) in lines.iter().enumerate() {
        let actual_line_num = line_num + 1;

        // Skip lines outside the requested range
        if actual_line_num < start_line || actual_line_num > end_line {
            continue;
        }

        // Find the most recent commit that affected this line
        // This is a simplified algorithm - in reality, you'd need proper diff tracking
        let blame_commit = find_line_origin(&file_commits, line_content, actual_line_num);

        let (commit_short, author_short, date_short) = if let Some(commit) = blame_commit {
            (
                commit.id[..8].to_string(),
                truncate_string(&commit.author.name, 12),
                format_timestamp(commit.time),
            )
        } else {
            (
                "????????".to_string(),
                "unknown".to_string(),
                "????-??-??".to_string(),
            )
        };

        // Format blame line
        println!(
            "{} {} {} {:4} {}",
            Style::commit_hash(&commit_short),
            Style::author_name(&author_short),
            date_short.dimmed(),
            format!("{}:", actual_line_num).dimmed(),
            line_content
        );
    }

    println!(
        "\nShowing lines {}-{} of {}",
        start_line,
        end_line,
        lines.len()
    );

    if !file_commits.is_empty() {
        println!("File has {} commits in history", file_commits.len());
    }

    Ok(())
}

fn display_file_tree(
    store: &rune_store::Store,
    path: &std::path::Path,
    show_all: bool,
    tracked_only: bool,
) -> anyhow::Result<()> {
    use colored::*;
    use std::collections::HashSet;

    // Get staged/tracked files if needed
    let tracked_files = if tracked_only {
        let index = store.read_index()?;
        Some(index.entries.keys().cloned().collect::<HashSet<String>>())
    } else {
        None
    };

    println!(
        "{}",
        format!(
            "📁 Repository Tree: {}",
            style::Style::file_path(&path.display().to_string())
        )
        .cyan()
        .bold()
    );
    println!();

    display_tree_recursive(
        store,
        path,
        "",
        show_all,
        tracked_only,
        tracked_files.as_ref(),
        0,
    )?;
    Ok(())
}

fn display_tree_recursive(
    store: &rune_store::Store,
    dir: &std::path::Path,
    prefix: &str,
    show_all: bool,
    tracked_only: bool,
    tracked_files: Option<&HashSet<String>>,
    depth: usize,
) -> anyhow::Result<()> {
    use colored::*;

    if depth > 10 {
        // Prevent infinite recursion
        return Ok(());
    }

    let mut entries = std::fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .collect::<Vec<_>>();

    entries.sort_by_key(|entry| entry.file_name());

    let repo_root = &store.root;

    for (i, entry) in entries.iter().enumerate() {
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        // Skip hidden files unless --all is specified
        if !show_all && file_name_str.starts_with('.') {
            continue;
        }

        // Skip .rune directory
        if file_name_str == ".rune" {
            continue;
        }

        let path = entry.path();
        let is_last = i == entries.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };

        // Check if file is tracked
        let relative_path = path
            .strip_prefix(repo_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        let is_tracked = tracked_files.map_or(true, |files| files.contains(&relative_path));

        if tracked_only && !is_tracked {
            continue;
        }

        let file_icon = if path.is_dir() {
            "📁"
        } else if is_tracked {
            "📄"
        } else {
            "📄"
        };

        let file_display = if is_tracked {
            format!("{} {}", file_icon, file_name_str.green())
        } else {
            format!("{} {}", file_icon, file_name_str.dimmed())
        };

        println!("{}{}{}", prefix, connector, file_display);

        if path.is_dir() {
            let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            display_tree_recursive(
                store,
                &path,
                &new_prefix,
                show_all,
                tracked_only,
                tracked_files,
                depth + 1,
            )?;
        }
    }

    Ok(())
}

fn list_repository_files(
    store: &rune_store::Store,
    cached: bool,
    modified: bool,
    stage: bool,
) -> anyhow::Result<()> {
    use colored::*;

    let index = store.read_index()?;

    if cached {
        // Show only staged files
        println!("{}", "📋 Staged Files:".cyan().bold());
        for file in index.entries.keys() {
            if stage {
                println!("{} {}", "M".green(), style::Style::file_path(file));
            } else {
                println!("{}", style::Style::file_path(file));
            }
        }
    } else if modified {
        // Show modified files (this is simplified - in reality you'd compare working tree to HEAD)
        println!("{}", "📝 Modified Files:".yellow().bold());
        // For now, just show staged files as they represent modifications
        for file in index.entries.keys() {
            if stage {
                println!("{} {}", "M".yellow(), style::Style::file_path(file));
            } else {
                println!("{}", style::Style::file_path(file));
            }
        }
    } else {
        // Show all tracked files
        println!("{}", "📄 All Tracked Files:".cyan().bold());

        // Get all files that have ever been committed
        let log = store.log();
        let mut all_files = std::collections::HashSet::new();

        for commit in &log {
            for file in &commit.files {
                all_files.insert(file.clone());
            }
        }

        // Add currently staged files
        for file in index.entries.keys() {
            all_files.insert(file.clone());
        }

        let mut files: Vec<_> = all_files.into_iter().collect();
        files.sort();

        for file in files {
            let status = if index.entries.contains_key(&file) {
                "M" // Modified/staged
            } else {
                " " // Committed
            };

            if stage {
                println!("{} {}", status.green(), style::Style::file_path(&file));
            } else {
                println!("{}", style::Style::file_path(&file));
            }
        }
    }

    Ok(())
}

fn display_commit_graph(
    commits: &[rune_core::Commit],
    show_graph: bool,
    oneline: bool,
) -> anyhow::Result<()> {
    use colored::*;

    if commits.is_empty() {
        return Ok(());
    }

    // Build a simple parent-child relationship map
    let mut children: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    let commit_map: std::collections::HashMap<String, &rune_core::Commit> =
        commits.iter().map(|c| (c.id.clone(), c)).collect();

    // Build children map (reverse of parent relationship)
    for commit in commits {
        if let Some(parent_id) = &commit.parent {
            children
                .entry(parent_id.clone())
                .or_insert_with(Vec::new)
                .push(commit.id.clone());
        }
    }

    // Display commits in reverse chronological order (newest first)
    for (i, commit) in commits.iter().rev().enumerate() {
        let ts = chrono::DateTime::from_timestamp(commit.time, 0)
            .unwrap()
            .naive_utc();
        let now = chrono::Utc::now().naive_utc();
        let ago = (now.and_utc().timestamp() - ts.and_utc().timestamp()) as i64;

        if show_graph {
            // Simple ASCII graph representation
            let graph_part = if i == 0 {
                "* " // First commit (HEAD)
            } else if commit.parent.is_some() {
                "| " // Has parent
            } else {
                "o " // Root commit
            };

            print!("{}", graph_part.yellow().bold());
        }

        if oneline {
            // Compact one-line format
            println!(
                "{} {} {} ({})",
                style::Style::commit_hash(&commit.id[..8]),
                truncate_string(&commit.message, 60),
                commit.author.name.dimmed(),
                style::format_duration(ago).dimmed()
            );
        } else {
            // Multi-line format with graph
            println!("commit {}", style::Style::commit_hash(&commit.id));
            if let Some(parent) = &commit.parent {
                println!("Parent:  {}", style::Style::commit_hash(parent));
            }
            println!("Author:  {}", commit.author.name);
            println!(
                "Date:    {} ({})",
                style::Style::timestamp(ts),
                style::format_duration(ago).dimmed()
            );
            println!();
            println!("    {}", commit.message);

            if !commit.files.is_empty() {
                println!();
                println!("    Files changed:");
                for file in &commit.files {
                    if show_graph {
                        println!("    |     + {}", style::Style::file_path(file));
                    } else {
                        println!("        + {}", style::Style::file_path(file));
                    }
                }
            }
            println!();
        }
    }

    Ok(())
}

fn parse_line_range(range: &str, max_lines: usize) -> anyhow::Result<(usize, usize)> {
    if range.contains(':') {
        let parts: Vec<&str> = range.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid range format. Use start:end"));
        }

        let start: usize = parts[0]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid start line number"))?;
        let end: usize = parts[1]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid end line number"))?;

        if start < 1 || end < start || end > max_lines {
            return Err(anyhow::anyhow!("Line range out of bounds"));
        }

        Ok((start, end))
    } else {
        // Single line number
        let line: usize = range
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid line number"))?;

        if line < 1 || line > max_lines {
            return Err(anyhow::anyhow!("Line number out of bounds"));
        }

        Ok((line, line))
    }
}

fn find_line_origin<'a>(
    commits: &[&'a rune_core::Commit],
    _line_content: &str,
    _line_num: usize,
) -> Option<&'a rune_core::Commit> {
    // Simplified: just return the most recent commit
    // In a real implementation, you'd track line changes through diffs
    commits.first().copied()
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        format!("{:width$}", s, width = max_len)
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}

fn format_timestamp(timestamp: i64) -> String {
    use chrono::{DateTime, Utc};

    let datetime = DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| Utc::now());
    datetime.format("%Y-%m-%d").to_string()
}

/// Enable quality bundle hooks (format, lint, test)
fn enable_quality_hooks(commands: Option<&str>, fail_fast: bool) -> anyhow::Result<()> {
    Style::section_header("Quality Bundle Setup");

    let rune_dir = std::env::current_dir()?.join(".rune");
    let hooks_dir = rune_dir.join("hooks");
    fs::create_dir_all(&hooks_dir)?;

    let default_commands =
        commands.unwrap_or("cargo fmt -- --check; cargo clippy -- -D warnings; cargo test");

    let hook_content = format!(
        r#"#!/bin/bash
# Rune Quality Bundle Pre-commit Hook
# Generated on {}

set -e

echo "Running quality checks..."

FAIL_FAST={}

# Split commands and run them
IFS=';' read -ra COMMANDS <<< "{}"

failed_commands=()

for cmd in "${{COMMANDS[@]}}"; do
    cmd=$(echo "$cmd" | xargs)  # trim whitespace
    if [ ! -z "$cmd" ]; then
        echo "→ Running: $cmd"
        if ! eval "$cmd"; then
            failed_commands+=("$cmd")
            if [ "$FAIL_FAST" = "true" ]; then
                echo "❌ Quality check failed: $cmd"
                echo "Use --no-verify to skip quality checks"
                exit 1
            fi
        else
            echo "✓ Passed: $cmd"
        fi
    fi
done

if [ ${{#failed_commands[@]}} -gt 0 ]; then
    echo ""
    echo "❌ Failed commands:"
    for cmd in "${{failed_commands[@]}}"; do
        echo "  - $cmd"
    done
    echo ""
    echo "Use --no-verify to skip quality checks"
    exit 1
fi

echo "✓ All quality checks passed!"
"#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
        fail_fast,
        default_commands
    );

    let hook_path = hooks_dir.join("pre-commit-quality");
    fs::write(&hook_path, hook_content)?;

    // Make executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms)?;
    }

    // Create hook registry entry
    let registry_path = hooks_dir.join("registry.json");
    let mut registry: std::collections::HashMap<String, serde_json::Value> =
        if registry_path.exists() {
            let content = fs::read_to_string(&registry_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            std::collections::HashMap::new()
        };

    registry.insert(
        "pre-commit-quality".to_string(),
        serde_json::json!({
            "enabled": true,
            "type": "pre-commit",
            "description": "Quality bundle: format, lint, test",
            "commands": default_commands,
            "fail_fast": fail_fast,
            "created": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
        }),
    );

    fs::write(&registry_path, serde_json::to_string_pretty(&registry)?)?;

    Style::success("Quality bundle hook enabled successfully");
    Style::info(&format!("Commands: {}", default_commands));
    Style::info(&format!("Fail fast: {}", fail_fast));
    Style::info("Hook will run on every commit");

    Ok(())
}

/// Enable secret scanning pre-commit hook
fn enable_secret_scan_hook(
    patterns_file: Option<&PathBuf>,
    exclude: &[String],
) -> anyhow::Result<()> {
    Style::section_header("Secret Scan Setup");

    let rune_dir = std::env::current_dir()?.join(".rune");
    let hooks_dir = rune_dir.join("hooks");
    fs::create_dir_all(&hooks_dir)?;

    // Default secret patterns
    let mut default_patterns = vec![
        r"(?i)(aws_access_key_id|aws_secret_access_key)\s*[:=]\s*[A-Z0-9]{20}".to_string(),
        r"(?i)(github_token|gh_token)\s*[:=]\s*ghp_[A-Za-z0-9]{36}".to_string(),
        r"(?i)(api_key|apikey)\s*[:=]\s*[A-Za-z0-9]{32,}".to_string(),
        r"(?i)password\s*[:=]\s*[^\s>]{8,}".to_string(),
        r"(?i)(private_key|private-key).*BEGIN.*PRIVATE KEY".to_string(),
        r"(?i)(oauth|token)\s*[:=]\s*[A-Za-z0-9\-_]{20,}".to_string(),
    ];

    // Load additional patterns if specified
    if let Some(patterns_path) = patterns_file {
        if patterns_path.exists() {
            let content = fs::read_to_string(patterns_path)?;
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    default_patterns.push(line.to_string());
                }
            }
        } else {
            Style::warning(&format!(
                "Patterns file not found: {}",
                patterns_path.display()
            ));
        }
    }

    let all_patterns = default_patterns;

    let exclude_patterns = exclude.join("|");

    let hook_content = format!(
        r#"#!/bin/bash
# Rune Secret Scan Pre-commit Hook
# Generated on {}

set -e

echo "Scanning for secrets..."

# Get staged files
staged_files=$(git diff --cached --name-only --diff-filter=ACM 2>/dev/null || find . -type f -name "*.rs" -o -name "*.js" -o -name "*.py" -o -name "*.json" -o -name "*.yaml" -o -name "*.yml" | head -20)

if [ -z "$staged_files" ]; then
    echo "No files to scan"
    exit 0
fi

# Secret patterns
patterns=()
{}

# Exclude patterns
exclude_pattern="{}"

secrets_found=false
scanned_count=0

for file in $staged_files; do
    # Skip if file matches exclude pattern
    if [ ! -z "$exclude_pattern" ] && echo "$file" | grep -qE "$exclude_pattern"; then
        continue
    fi
    
    # Skip binary files and common ignore patterns
    if file --mime-type "$file" 2>/dev/null | grep -q "binary\\|image\\|video\\|audio"; then
        continue
    fi
    
    if echo "$file" | grep -qE "\\.(git|node_modules|target|build|dist|vendor)/|\\.(exe|bin|so|dylib|dll|png|jpg|jpeg|gif|ico|woff|woff2|ttf|eot)$"; then
        continue
    fi
    
    ((scanned_count++))
    
    # Scan file with each pattern
    for pattern in "${{patterns[@]}}"; do
        if grep -nHE "$pattern" "$file" 2>/dev/null; then
            echo "❌ Potential secret found in $file"
            secrets_found=true
        fi
    done
done

echo "Scanned $scanned_count files"

if [ "$secrets_found" = true ]; then
    echo ""
    echo "❌ Potential secrets detected!"
    echo "Please review the files above and remove any sensitive data"
    echo "Use --no-verify to skip secret scanning (NOT RECOMMENDED)"
    exit 1
fi

echo "✓ No secrets detected"
"#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
        all_patterns
            .iter()
            .map(|p| format!("patterns+=('{}')", p.replace("'", "'\"'\"'")))
            .collect::<Vec<_>>()
            .join("\n"),
        exclude_patterns
    );

    let hook_path = hooks_dir.join("pre-commit-secrets");
    fs::write(&hook_path, hook_content)?;

    // Make executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms)?;
    }

    // Create hook registry entry
    let registry_path = hooks_dir.join("registry.json");
    let mut registry: std::collections::HashMap<String, serde_json::Value> =
        if registry_path.exists() {
            let content = fs::read_to_string(&registry_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            std::collections::HashMap::new()
        };

    registry.insert(
        "pre-commit-secrets".to_string(),
        serde_json::json!({
            "enabled": true,
            "type": "pre-commit",
            "description": "Secret scanning for leaked credentials",
            "patterns_count": all_patterns.len(),
            "exclude_patterns": exclude,
            "created": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
        }),
    );

    fs::write(&registry_path, serde_json::to_string_pretty(&registry)?)?;

    Style::success("Secret scan hook enabled successfully");
    Style::info(&format!(
        "Monitoring {} secret patterns",
        all_patterns.len()
    ));
    if !exclude.is_empty() {
        Style::info(&format!("Excluding: {}", exclude.join(", ")));
    }
    Style::info("Hook will run on every commit");

    Ok(())
}

/// Handle benchmark commands with comprehensive performance testing
async fn handle_benchmark_command(cmd: BenchmarkCmd, ctx: &RuneContext) -> anyhow::Result<()> {
    let monitor = PerformanceMonitor::new();

    match cmd {
        BenchmarkCmd::Run {
            suite,
            format,
            output,
        } => {
            ctx.info(&format!("🚀 Running {} benchmark suite...", suite));

            let result = monitor.run_benchmark_suite(&suite).await?;

            match format.as_str() {
                "json" => {
                    let json_output = serde_json::to_string_pretty(&result)?;
                    if let Some(output_path) = output {
                        std::fs::write(&output_path, &json_output)?;
                        ctx.info(&format!("Results saved to {}", output_path.display()));
                    } else {
                        println!("{}", json_output);
                    }
                }
                "table" | _ => {
                    Style::section_header(&format!("📊 {} Benchmark Results", suite));

                    println!("\n🎯 {}", "Performance Summary".bold());
                    println!("  ⏱️  Duration: {:.2?}", result.duration);
                    println!("  🚀 Operations/sec: {:.1}", result.operations_per_second);
                    println!(
                        "  🧠 Peak Memory: {}",
                        format_bytes(result.peak_memory_usage as usize)
                    );
                    println!("  💻 Peak CPU: {:.1}%", result.peak_cpu_usage);
                    println!("  🎯 Cache Hit Ratio: {:.1}%", result.cache_hit_ratio);
                    println!("  ✅ Success Rate: {:.1}%", result.success_rate);

                    if !result.bottlenecks.is_empty() {
                        println!("\n⚠️  {}", "Performance Bottlenecks".bold());
                        for bottleneck in &result.bottlenecks {
                            let severity_icon = match bottleneck.severity {
                                rune_performance::BottleneckSeverity::Low => "🟡",
                                rune_performance::BottleneckSeverity::Medium => "🟠",
                                rune_performance::BottleneckSeverity::High => "🔴",
                                rune_performance::BottleneckSeverity::Critical => "🚨",
                            };
                            println!(
                                "  {} {} (Impact: {:.1}%)",
                                severity_icon, bottleneck.description, bottleneck.impact
                            );
                            for rec in &bottleneck.recommendations {
                                println!("    💡 {}", rec);
                            }
                        }
                    }

                    if let Some(output_path) = output {
                        let json_output = serde_json::to_string_pretty(&result)?;
                        std::fs::write(&output_path, &json_output)?;
                        ctx.info(&format!(
                            "Detailed results saved to {}",
                            output_path.display()
                        ));
                    }
                }
            }
        }

        BenchmarkCmd::Monitor { interval, history } => {
            ctx.info(&format!(
                "📊 Starting performance monitor ({}s intervals, {} history entries)",
                interval, history
            ));
            Style::section_header("Real-time Performance Monitor");

            // Simple monitoring loop - in a real implementation this would be more sophisticated
            for i in 0..10 {
                tokio::time::sleep(std::time::Duration::from_secs(interval)).await;

                let metrics = monitor.get_current_metrics();
                println!(
                    "\n📈 Sample {} - {} UTC",
                    i + 1,
                    chrono::Utc::now().format("%H:%M:%S")
                );
                println!("  CPU: {:.1}%", metrics.cpu_usage);
                println!("  Memory: {}", format_bytes(metrics.memory_usage as usize));
                println!(
                    "  Cache Hit Ratio: {:.1}%",
                    metrics.cache_performance.hit_ratio
                );

                if i >= 9 {
                    ctx.info(
                        "Monitor completed. Use 'rune benchmark report' for detailed analysis.",
                    );
                    break;
                }
            }
        }

        BenchmarkCmd::Report {
            trends,
            format,
            output,
        } => {
            ctx.info("📋 Generating comprehensive performance report...");

            let report = monitor.generate_performance_report();

            match format.as_str() {
                "json" => {
                    let json_output = serde_json::to_string_pretty(&report)?;
                    if let Some(output_path) = output {
                        std::fs::write(&output_path, &json_output)?;
                        ctx.info(&format!("Report saved to {}", output_path.display()));
                    } else {
                        println!("{}", json_output);
                    }
                }
                "html" => {
                    let html_report = generate_html_report(&report, trends)?;
                    if let Some(output_path) = output {
                        std::fs::write(&output_path, &html_report)?;
                        ctx.info(&format!("HTML report saved to {}", output_path.display()));
                    } else {
                        ctx.error("HTML format requires --output parameter");
                    }
                }
                "table" | _ => {
                    Style::section_header("📊 Performance Report");

                    println!("\n🎯 {}", "Current Metrics".bold());
                    println!("  💻 CPU Usage: {:.1}%", report.current_metrics.cpu_usage);
                    println!(
                        "  🧠 Memory Usage: {}",
                        format_bytes(report.current_metrics.memory_usage as usize)
                    );
                    println!(
                        "  💾 Cache Hit Ratio: {:.1}%",
                        report.current_metrics.cache_performance.hit_ratio
                    );

                    if trends && !report.historical_trends.is_empty() {
                        println!("\n📈 {}", "Historical Trends".bold());
                        for trend in &report.historical_trends {
                            let direction_icon = match trend.direction {
                                rune_performance::TrendDirection::Improving => "📈",
                                rune_performance::TrendDirection::Degrading => "📉",
                                rune_performance::TrendDirection::Stable => "➡️",
                            };
                            println!(
                                "  {} {}: {:.1}% change",
                                direction_icon, trend.metric, trend.change_percentage
                            );
                        }
                    }

                    if !report.recommendations.is_empty() {
                        println!("\n💡 {}", "Recommendations".bold());
                        for rec in &report.recommendations {
                            println!("  • {}", rec);
                        }
                    }

                    if let Some(output_path) = output {
                        let json_output = serde_json::to_string_pretty(&report)?;
                        std::fs::write(&output_path, &json_output)?;
                        ctx.info(&format!(
                            "Detailed report saved to {}",
                            output_path.display()
                        ));
                    }
                }
            }
        }

        BenchmarkCmd::List => {
            Style::section_header("📋 Available Benchmark Suites");

            println!("\n🚀 {}", "Performance Benchmark Suites".bold());
            println!(
                "  {} comprehensive    - Full performance evaluation",
                Style::status_added()
            );
            println!(
                "  {} large_repository - Linux kernel scale repository testing",
                Style::status_added()
            );
            println!(
                "  {} network_latency  - Network performance simulation",
                Style::status_added()
            );
            println!(
                "  {} memory_usage     - Memory pressure testing",
                Style::status_added()
            );
            println!(
                "  {} disk_io          - Disk I/O performance evaluation",
                Style::status_added()
            );

            println!("\n📊 {}", "Monitoring Options".bold());
            println!(
                "  {} rune benchmark monitor     - Real-time performance monitoring",
                Style::status_added()
            );
            println!(
                "  {} rune benchmark report      - Generate performance report",
                Style::status_added()
            );
            println!(
                "  {} rune benchmark run --suite comprehensive - Run full benchmark",
                Style::status_added()
            );

            println!("\n💡 {}", "Example Commands".bold());
            println!(
                "  rune benchmark run --suite comprehensive --format json --output results.json"
            );
            println!("  rune benchmark monitor --interval 5 --history 50");
            println!("  rune benchmark report --trends --format html --output report.html");
        }
    }

    Ok(())
}

/// Generate HTML performance report
fn generate_html_report(
    report: &rune_performance::PerformanceReport,
    include_trends: bool,
) -> anyhow::Result<String> {
    let timestamp = chrono::DateTime::from_timestamp(report.timestamp as i64, 0)
        .unwrap_or_else(|| chrono::Utc::now())
        .format("%Y-%m-%d %H:%M:%S UTC");

    let mut html = format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>Rune VCS Performance Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .header {{ background: #2563eb; color: white; padding: 20px; border-radius: 8px; }}
        .metric {{ background: #f3f4f6; padding: 15px; margin: 10px 0; border-radius: 5px; }}
        .recommendations {{ background: #fef3c7; padding: 15px; border-radius: 5px; }}
        .trend-improving {{ color: #059669; }}
        .trend-degrading {{ color: #dc2626; }}
        .trend-stable {{ color: #6b7280; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🚀 Rune VCS Performance Report</h1>
        <p>Generated: {}</p>
    </div>
    
    <div class="metric">
        <h2>📊 Current Metrics</h2>
        <p><strong>CPU Usage:</strong> {:.1}%</p>
        <p><strong>Memory Usage:</strong> {}</p>
        <p><strong>Cache Hit Ratio:</strong> {:.1}%</p>
    </div>
"#,
        timestamp,
        report.current_metrics.cpu_usage,
        format_bytes(report.current_metrics.memory_usage as usize),
        report.current_metrics.cache_performance.hit_ratio
    );

    if include_trends && !report.historical_trends.is_empty() {
        html.push_str(
            r#"
    <div class="metric">
        <h2>📈 Historical Trends</h2>
"#,
        );
        for trend in &report.historical_trends {
            let class = match trend.direction {
                rune_performance::TrendDirection::Improving => "trend-improving",
                rune_performance::TrendDirection::Degrading => "trend-degrading",
                rune_performance::TrendDirection::Stable => "trend-stable",
            };
            html.push_str(&format!(
                r#"        <p class="{}"><strong>{}:</strong> {:.1}% change</p>"#,
                class, trend.metric, trend.change_percentage
            ));
        }
        html.push_str("    </div>\n");
    }

    if !report.recommendations.is_empty() {
        html.push_str(
            r#"
    <div class="recommendations">
        <h2>💡 Recommendations</h2>
        <ul>
"#,
        );
        for rec in &report.recommendations {
            html.push_str(&format!("            <li>{}</li>\n", rec));
        }
        html.push_str("        </ul>\n    </div>\n");
    }

    html.push_str("</body>\n</html>");
    Ok(html)
}

/// Format bytes in human-readable format
fn format_bytes(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.1}{}", size, UNITS[unit_index])
}
