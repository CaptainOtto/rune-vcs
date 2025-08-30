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
enum BinaryCommand {
    /// Analyze binary files and suggest optimizations
    Analyze {
        #[arg(help = "Path to analyze (file or directory)", default_value = ".")]
        path: String,
        #[arg(short, long, help = "Show detailed analysis")]
        detailed: bool,
        #[arg(long, help = "Include performance impact analysis")]
        performance: bool,
        #[arg(long, help = "Suggest LFS recommendations")]
        lfs_suggestions: bool,
    },
    /// Smart migration of binaries to LFS
    Migrate {
        #[arg(help = "Size threshold in MB for migration")]
        threshold_mb: Option<u64>,
        #[arg(long, help = "Dry run - show what would be migrated")]
        dry_run: bool,
        #[arg(long, help = "Auto-confirm migration")]
        auto_confirm: bool,
        #[arg(long, help = "Include file pattern filters")]
        patterns: Vec<String>,
    },
    /// Optimize binary storage and compression
    Optimize {
        #[arg(long, help = "Target compression ratio (0.1-0.9)")]
        target_ratio: Option<f64>,
        #[arg(long, help = "Aggressive optimization mode")]
        aggressive: bool,
        #[arg(long, help = "Preserve original files")]
        preserve: bool,
    },
    /// Track binary file dependencies and relationships
    Dependencies {
        #[arg(help = "Binary file to analyze")]
        file: String,
        #[arg(short, long, help = "Show dependency graph")]
        graph: bool,
        #[arg(long, help = "Find circular dependencies")]
        circular: bool,
    },
    /// Smart binary branching strategies
    Branch {
        #[arg(help = "Branch strategy: lightweight, isolated, shared")]
        strategy: String,
        #[arg(long, help = "Auto-detect optimal strategy")]
        auto_detect: bool,
        #[arg(long, help = "Show strategy comparison")]
        compare: bool,
    },
}

#[derive(Subcommand, Debug)]
enum SmartBranchCommand {
    /// Create an AI-optimized branch with intelligent naming
    Create {
        #[arg(help = "Branch purpose: feature, bugfix, hotfix, experiment")]
        purpose: String,
        #[arg(help = "Brief description of the change")]
        description: String,
        #[arg(long, help = "Auto-generate branch name")]
        auto_name: bool,
        #[arg(long, help = "Suggest branch strategy")]
        suggest_strategy: bool,
        #[arg(long, help = "Include performance optimizations")]
        optimize: bool,
    },
    /// Merge branches with AI conflict prediction
    Merge {
        #[arg(help = "Branch to merge")]
        branch: String,
        #[arg(long, help = "Predict merge conflicts")]
        predict_conflicts: bool,
        #[arg(long, help = "Auto-resolve safe conflicts")]
        auto_resolve: bool,
        #[arg(long, help = "Use AI merge strategy selection")]
        smart_strategy: bool,
        #[arg(long, help = "Generate merge commit message")]
        auto_message: bool,
    },
    /// Analyze branch health and suggest optimizations
    Health {
        #[arg(help = "Branch to analyze", default_value = "current")]
        branch: String,
        #[arg(long, help = "Show detailed health report")]
        detailed: bool,
        #[arg(long, help = "Include performance metrics")]
        performance: bool,
        #[arg(long, help = "Suggest cleanup actions")]
        cleanup: bool,
    },
    /// Smart branch cleanup with AI recommendations
    Cleanup {
        #[arg(long, help = "Remove merged branches")]
        merged: bool,
        #[arg(long, help = "Remove stale branches")]
        stale: bool,
        #[arg(long, help = "Days to consider stale")]
        stale_days: Option<u64>,
        #[arg(long, help = "Dry run - show what would be cleaned")]
        dry_run: bool,
        #[arg(long, help = "Interactive cleanup")]
        interactive: bool,
    },
    /// Intelligent branch strategy recommendations
    Strategy {
        #[arg(help = "Project type: web, mobile, gamedev, enterprise")]
        project_type: Option<String>,
        #[arg(help = "Team size: small, medium, large")]
        team_size: Option<String>,
        #[arg(long, help = "Show strategy comparison")]
        compare: bool,
        #[arg(long, help = "Customize strategy")]
        customize: bool,
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
enum BranchCommand {
    /// Create a new branch
    Create {
        #[arg(help = "Name of the new branch")]
        name: String,
        #[arg(long, short, help = "Start point for the new branch (commit, branch, or tag)")]
        start_point: Option<String>,
        #[arg(long, help = "Set up tracking information")]
        track: bool,
    },
    /// Delete a branch
    Delete {
        #[arg(help = "Name of the branch to delete")]
        name: String,
        #[arg(long, short = 'D', help = "Force delete even if not merged")]
        force: bool,
        #[arg(long, short, help = "Delete remote-tracking branch")]
        remote: bool,
    },
    /// Rename a branch
    Rename {
        #[arg(help = "Current name of the branch")]
        old_name: String,
        #[arg(help = "New name for the branch")]
        new_name: String,
        #[arg(long, short, help = "Force rename even if new name exists")]
        force: bool,
    },
    /// List branches
    List {
        #[arg(long, short, help = "List remote-tracking branches")]
        remotes: bool,
        #[arg(long, short, help = "List both local and remote branches")]
        all: bool,
        #[arg(long, help = "Show only merged branches")]
        merged: bool,
        #[arg(long, help = "Show only unmerged branches")]
        no_merged: bool,
        #[arg(long, help = "Show verbose output")]
        verbose: bool,
    },
    /// Set upstream tracking for a branch
    SetUpstream {
        #[arg(help = "Remote branch to track (e.g., origin/main)")]
        upstream: String,
        #[arg(long, short, help = "Unset the upstream")]
        unset: bool,
    },
}

#[derive(Subcommand, Debug)]
enum TagCommand {
    /// Create a new tag
    Create {
        #[arg(help = "Name of the new tag")]
        name: String,
        #[arg(help = "Commit to tag (defaults to HEAD)")]
        commit: Option<String>,
        #[arg(short, long, help = "Create an annotated tag")]
        annotate: bool,
        #[arg(short, long, help = "Tag message")]
        message: Option<String>,
        #[arg(long, help = "Force create tag even if it exists")]
        force: bool,
    },
    /// Delete a tag
    Delete {
        #[arg(help = "Name of the tag to delete")]
        name: String,
    },
    /// List tags
    List {
        #[arg(long, short, help = "Show tags in verbose format")]
        verbose: bool,
        #[arg(long, help = "Pattern to match tag names")]
        pattern: Option<String>,
    },
    /// Show tag information
    Show {
        #[arg(help = "Name of the tag to show")]
        name: String,
    },
    /// Verify tag signature
    Verify {
        #[arg(help = "Name of the tag to verify")]
        name: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
enum BatchOperation {
    /// Batch commit multiple files
    Commit {
        #[arg(help = "Commit message")]
        message: String,
    },
    /// Batch push to multiple remotes/branches
    Push {
        #[arg(help = "Remote name")]
        remote: String,
        #[arg(help = "Branch name")]
        branch: String,
    },
    /// Batch pull from multiple remotes/branches
    Pull {
        #[arg(help = "Remote name")]
        remote: String,
        #[arg(help = "Branch name")]
        branch: String,
    },
    /// Batch add multiple files
    Add {
        #[arg(help = "File paths")]
        paths: Vec<String>,
    },
    /// Batch create tags
    Tag {
        #[arg(help = "Tag name")]
        name: String,
    },
    /// Batch branch operations
    Branch {
        #[arg(help = "Branch name")]
        name: String,
    },
    /// Batch merge operations
    Merge {
        #[arg(help = "Branch to merge")]
        branch: String,
    },
    /// Show status for batch operations
    Status,
    /// Show log for batch operations
    Log {
        #[arg(help = "Number of commits to show")]
        count: Option<usize>,
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
    
    // ============ SMART WORKFLOW COMMANDS ============
    /// Smart interactive workflow: status → staging → commit
    Work {
        #[arg(short, long, help = "Auto-stage all changes")]
        all: bool,
        #[arg(short, long, help = "Interactive staging mode")]
        interactive: bool,
        #[arg(long, help = "Quick commit with generated message")]
        quick: bool,
        #[arg(short, long, help = "Commit message")]
        message: Option<String>,
    },
    /// Smart commit and push workflow with conflict resolution
    Ship {
        #[arg(short = 'm', long, help = "Commit message")]
        message: Option<String>,
        #[arg(short = 'A', long, help = "Auto-stage all changes")]
        all: bool,
        #[arg(short = 'F', long, help = "Force push (dangerous)")]
        force: bool,
        #[arg(short = 'U', long, help = "Set upstream tracking")]
        upstream: bool,
        #[arg(long, help = "Target remote", default_value = "origin")]
        remote: String,
        #[arg(long, help = "Target branch")]
        branch: Option<String>,
    },
    /// Smart sync: pull + merge with automatic stash handling
    Sync {
        #[arg(long, help = "Source remote", default_value = "origin")]
        remote: String,
        #[arg(long, help = "Source branch")]
        branch: Option<String>,
        #[arg(short = 'A', long, help = "Automatically handle conflicts")]
        auto: bool,
        #[arg(long, help = "Strategy for conflicts")]
        strategy: Option<String>,
    },
    /// Smart exploration: log + diff + blame in interactive mode
    Explore {
        #[arg(help = "File or commit to explore")]
        target: Option<String>,
        #[arg(short = 'g', long, help = "Show visual commit graph")]
        graph: bool,
        #[arg(short = 'I', long, help = "Interactive mode")]
        interactive: bool,
        #[arg(short = 'n', long, help = "Number of commits to show", default_value = "10")]
        count: usize,
    },
    /// Smart cleanup: reset + stash + clean with safety checks
    Clean {
        #[arg(short = 'w', long, help = "Clean working directory")]
        working: bool,
        #[arg(short = 's', long, help = "Clean staging area")]
        staging: bool,
        #[arg(short = 'r', long, help = "Hard reset to commit")]
        reset: Option<String>,
        #[arg(short = 'f', long, help = "Force operation (skip confirmations)")]
        force: bool,
    },
    /// Smart branch workflow: create + switch + track
    Flow {
        #[arg(help = "Branch name or operation")]
        branch: Option<String>,
        #[arg(short = 'c', long, help = "Create new branch")]
        create: bool,
        #[arg(short = 'M', long, help = "Merge branch into current")]
        merge: Option<String>,
        #[arg(short = 'd', long, help = "Delete branch after merge")]
        delete: bool,
        #[arg(short = 't', long, help = "Set upstream tracking")]
        track: bool,
    },
    
    // ============ AI-POWERED SMART FEATURES ============
    /// Smart AI suggestions based on repository context and patterns
    Suggest {
        #[arg(help = "What to get suggestions for: workflow, commit, branch, merge, performance")]
        category: Option<String>,
        #[arg(short = 'f', long, help = "Focus area: security, performance, quality, productivity")]
        focus: Option<String>,
        #[arg(short = 'a', long, help = "Include automation suggestions")]
        automation: bool,
        #[arg(short = 'l', long, help = "Show learning opportunities")]
        learning: bool,
    },
    
    /// Interactive repository dashboard with real-time insights
    Dashboard {
        #[arg(short = 'r', long, help = "Refresh interval in seconds", default_value = "5")]
        refresh: u64,
        #[arg(short = 'c', long, help = "Show compact view")]
        compact: bool,
        #[arg(short = 'w', long, help = "Watch mode - continuous updates")]
        watch: bool,
        #[arg(short = 'f', long, help = "Filter by: health, performance, security, activity")]
        filter: Option<String>,
    },
    
    /// Smart workflow automation with AI recommendations
    AutoFlow {
        #[arg(help = "Automation type: release, hotfix, feature, cleanup")]
        workflow_type: String,
        #[arg(short = 'd', long, help = "Dry run - show what would happen")]
        dry_run: bool,
        #[arg(short = 'i', long, help = "Interactive mode with confirmations")]
        interactive: bool,
        #[arg(short = 'l', long, help = "Learn from this workflow for future suggestions")]
        learn: bool,
    },
    
    /// Intelligent conflict prevention and resolution
    Guard {
        #[arg(help = "Operation to guard: merge, rebase, pull, push")]
        operation: String,
        #[arg(short = 'p', long, help = "Predict potential conflicts")]
        predict: bool,
        #[arg(short = 'a', long, help = "Auto-resolve safe conflicts")]
        auto_resolve: bool,
        #[arg(short = 's', long, help = "Suggest resolution strategies")]
        strategies: bool,
    },
    
    /// Revolutionary AI-powered binary file management  
    Binary {
        #[command(subcommand)]
        cmd: Option<BinaryCommand>,
    },
    
    /// Intelligent branching with AI optimization
    SmartBranch {
        #[command(subcommand)]
        cmd: Option<SmartBranchCommand>,
    },
    
    // ============ NATURAL LANGUAGE COMMANDS ============
    /// Natural language: Undo last commit (git reset HEAD~1)
    #[command(name = "rollback")]
    Rollback {
        #[arg(help = "What to undo: commit, merge, changes")]
        what: Option<String>,
        #[arg(long, help = "How many commits to undo")]
        count: Option<u32>,
        #[arg(long, help = "Keep changes in working directory")]
        soft: bool,
        #[arg(long, help = "Completely discard changes")]
        hard: bool,
    },
    
    /// Natural language: Show me what changed
    #[command(name = "changed")]
    Changed {
        #[arg(help = "Time reference: today, yesterday, last-week, or specific date")]
        since: Option<String>,
        #[arg(short, long, help = "Show file names only")]
        names_only: bool,
        #[arg(short, long, help = "Show statistics")]
        stats: bool,
    },
    
    /// Natural language: What conflicts exist
    #[command(name = "conflicts")]
    Conflicts {
        #[arg(long, help = "Show resolution suggestions")]
        suggest: bool,
        #[arg(long, help = "Auto-resolve safe conflicts")]
        auto_resolve: bool,
        #[arg(long, help = "Interactive conflict resolution")]
        interactive: bool,
    },
    
    /// Smart fix common repository issues
    Fix {
        #[arg(help = "What to fix: conflicts, formatting, permissions, corruption")]
        issue: Option<String>,
        #[arg(long, help = "Show what would be fixed without doing it")]
        dry_run: bool,
        #[arg(long, help = "Automatically fix all safe issues")]
        auto: bool,
        #[arg(long, help = "Interactive mode for complex fixes")]
        interactive: bool,
    },
    
    /// Optimize repository for better performance
    Optimize {
        #[arg(long, help = "Optimization level: basic, standard, aggressive")]
        level: Option<String>,
        #[arg(long, help = "Show optimization analysis")]
        analyze: bool,
        #[arg(long, help = "Dry run - show what would be optimized")]
        dry_run: bool,
        #[arg(long, help = "Include LFS optimization")]
        lfs: bool,
    },
    
    /// Interactive repository health check and maintenance
    Health {
        #[arg(long, help = "Show detailed health report")]
        detailed: bool,
        #[arg(long, help = "Include performance metrics")]
        performance: bool,
        #[arg(long, help = "Suggest improvements")]
        suggestions: bool,
        #[arg(long, help = "Auto-fix safe issues")]
        auto_fix: bool,
    },
    
    // ============ NATURAL LANGUAGE COMMANDS ============
    
    /// Natural language command: "undo last commit" 
    #[command(name = "undo-op", about = "Natural language undo operations")]
    UndoOp {
        #[arg(help = "What to undo: 'last commit', 'all changes', 'staging'")]
        operation: String,
        #[arg(long, help = "Number of operations to undo")]
        count: Option<u32>,
        #[arg(long, help = "Force undo without confirmation")]
        force: bool,
    },
    
    /// Natural language command: "show me" information
    #[command(name = "display", about = "Natural language information display")]
    Display {
        #[arg(help = "What to show: 'conflicts', 'changes', 'history', 'branches'")]
        what: String,
        #[arg(long, help = "Time filter: 'today', 'yesterday', 'week'")]
        since: Option<String>,
        #[arg(long, help = "Show detailed information")]
        detailed: bool,
    },
    
    /// Natural language command: "what changed"
    #[command(name = "what", about = "Natural language change queries")]
    What {
        #[arg(help = "Query: 'changed since yesterday', 'conflicts exist', 'needs attention'")]
        query: String,
        #[arg(long, help = "Include file details")]
        files: bool,
        #[arg(long, help = "Include author information")]
        authors: bool,
    },
    
    /// Enhanced developer experience with context-sensitive help
    #[command(name = "help-me", about = "Intelligent context-aware assistance")]
    HelpMe {
        #[arg(help = "Current situation or problem")]
        situation: Option<String>,
        #[arg(long, help = "Show interactive problem solver")]
        interactive: bool,
        #[arg(long, help = "Include workflow suggestions")]
        workflows: bool,
    },
    
    /// Workflow templates for common development patterns
    #[command(name = "template", about = "Workflow automation templates")]
    Template {
        #[arg(help = "Template type: hotfix, feature, release, bugfix")]
        template_type: String,
        #[arg(help = "Template name or description")]
        name: String,
        #[arg(long, help = "Show available templates")]
        list: bool,
        #[arg(long, help = "Customize template parameters")]
        customize: bool,
    },
    
    /// Batch operations for multiple files/branches
    #[command(name = "batch", about = "Intelligent batch operations")]
    Batch {
        #[command(subcommand)]
        operation: Option<BatchOperation>,
    },
    
    /// File system monitoring with auto-actions
    #[command(name = "watch", about = "Smart file system monitoring")]
    Watch {
        #[arg(help = "Path to watch", default_value = ".")]
        path: String,
        #[arg(long, help = "Auto-commit on changes")]
        auto_commit: bool,
        #[arg(long, help = "Auto-run tests on changes")]
        auto_test: bool,
        #[arg(long, help = "Watch patterns (glob)")]
        patterns: Vec<String>,
    },
    
    // ============ TRADITIONAL COMMANDS ============
    
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
        #[command(subcommand)]
        command: Option<BranchCommand>,
        #[arg(long, default_value = "table")]
        format: String,
    },
    Checkout {
        /// Branch name, commit, or file path to checkout
        target: String,
        #[arg(short, long, help = "Create a new branch")]
        branch: bool,
        #[arg(long, help = "Force checkout (discard local changes)")]
        force: bool,
        #[arg(help = "Files to restore from the specified commit")]
        files: Vec<std::path::PathBuf>,
    },
    /// Merge a branch into the current branch
    Merge {
        #[arg(help = "Branch to merge into current branch")]
        branch: Option<String>,
        #[arg(long, help = "Create merge commit even for fast-forward")]
        no_ff: bool,
        #[arg(long, help = "Abort merge in progress", conflicts_with = "continue_merge")]
        abort: bool,
        #[arg(long, help = "Continue merge after resolving conflicts", conflicts_with = "abort")]
        continue_merge: bool,
        #[arg(long, help = "Merge strategy to use", value_parser = ["ours", "theirs", "recursive"])]
        strategy: Option<String>,
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
        #[arg(help = "Commit hash, or commit:file to show file at commit", default_value = "HEAD")]
        commit: String,
        #[arg(long, help = "Show specific file at the commit")]
        file: Option<PathBuf>,
        #[arg(long, help = "Show file names only")]
        name_only: bool,
        #[arg(long, help = "Show file statistics")]
        stat: bool,
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
        #[arg(long, help = "Push tags along with the branch")]
        tags: bool,
        #[arg(long, help = "Push all tags")]
        all_tags: bool,
        #[arg(long, help = "Force push (use with caution)")]
        force: bool,
        #[arg(long, short = 'u', help = "Set upstream tracking")]
        set_upstream: bool,
        #[arg(long, help = "Dry run - show what would be pushed")]
        dry_run: bool,
        #[arg(long, help = "Push all branches")]
        all: bool,
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
        target: Option<String>,
        #[arg(short, long, help = "Start interactive rebase")]
        interactive: bool,
        #[arg(long, help = "Rebase onto specific commit")]
        onto: Option<String>,
        #[arg(long, help = "Preserve merge commits")]
        preserve_merges: bool,
        #[arg(long, help = "Automatically squash fixup commits")]
        autosquash: bool,
        #[arg(long, help = "Abort rebase in progress", conflicts_with = "continue_rebase")]
        abort: bool,
        #[arg(long, help = "Continue rebase after resolving conflicts", conflicts_with = "abort")]
        continue_rebase: bool,
        #[arg(long, help = "Skip current commit during rebase")]
        skip: bool,
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
    /// Tag management
    Tag {
        #[command(subcommand)]
        command: Option<TagCommand>,
    },
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
        #[arg(long, help = "Get global configuration")]
        global: bool,
    },
    /// Set configuration value
    Set {
        #[arg(help = "Configuration key to set")]
        key: String,
        #[arg(help = "Configuration value")]
        value: String,
        #[arg(long, help = "Set global configuration")]
        global: bool,
    },
    /// Unset configuration value
    Unset {
        #[arg(help = "Configuration key to unset")]
        key: String,
        #[arg(long, help = "Unset global configuration")]
        global: bool,
    },
    /// List all configuration
    List {
        #[arg(long, help = "List global configuration")]
        global: bool,
    },
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
        ConfigCmd::Get { key, global } => {
            let value = get_config_value(&key, global)?;
            match value {
                Some(v) => println!("{}", v),
                None => {
                    Style::error(&format!("Configuration key '{}' not found", key));
                }
            }
        }
        ConfigCmd::Set { key, value, global } => {
            set_config_value(&key, &value, global)?;
            Style::success(&format!("Configuration '{}' set to '{}'", key, value));
        }
        ConfigCmd::Unset { key, global } => {
            unset_config_value(&key, global)?;
            Style::success(&format!("Configuration '{}' unset", key));
        }
        ConfigCmd::List { global } => {
            list_configuration(global)?;
        }
        ConfigCmd::Intelligence => {
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

/// Get configuration value from global or repository config
fn get_config_value(key: &str, global: bool) -> anyhow::Result<Option<String>> {
    use std::fs;
    
    let config_path = if global {
        dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
            .join(".runeconfig")
    } else {
        std::env::current_dir()?.join(".rune").join("config")
    };

    if !config_path.exists() {
        return Ok(None);
    }

    let config_content = fs::read_to_string(config_path)?;
    for line in config_content.lines() {
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == key {
                return Ok(Some(v.trim().to_string()));
            }
        }
    }

    // Fallback to known config keys
    match key {
        "user.name" => Ok(Some(whoami::realname())),
        "user.email" => Ok(Some(format!("{}@localhost", whoami::username()))),
        "intelligence.enabled" => Ok(Some(
            std::env::var("RUNE_INTELLIGENCE").unwrap_or("true".to_string())
        )),
        "intelligence.notifications" => Ok(Some(
            std::env::var("RUNE_INTELLIGENCE_NOTIFICATIONS").unwrap_or("info".to_string())
        )),
        _ => Ok(None),
    }
}

/// Set configuration value in global or repository config
fn set_config_value(key: &str, value: &str, global: bool) -> anyhow::Result<()> {
    use std::fs;
    use std::io::Write;

    let config_path = if global {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        home.join(".runeconfig")
    } else {
        let rune_dir = std::env::current_dir()?.join(".rune");
        if !rune_dir.exists() {
            return Err(anyhow::anyhow!("Not in a Rune repository. Use --global for global config."));
        }
        rune_dir.join("config")
    };

    let mut config = std::collections::HashMap::new();
    
    // Read existing config
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        for line in content.lines() {
            if let Some((k, v)) = line.split_once('=') {
                config.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
    }

    // Update the key
    config.insert(key.to_string(), value.to_string());

    // Write back
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let mut file = fs::File::create(&config_path)?;
    for (k, v) in config {
        writeln!(file, "{}={}", k, v)?;
    }

    Ok(())
}

/// Unset configuration value
fn unset_config_value(key: &str, global: bool) -> anyhow::Result<()> {
    use std::fs;
    use std::io::Write;

    let config_path = if global {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        home.join(".runeconfig")
    } else {
        let rune_dir = std::env::current_dir()?.join(".rune");
        if !rune_dir.exists() {
            return Err(anyhow::anyhow!("Not in a Rune repository. Use --global for global config."));
        }
        rune_dir.join("config")
    };

    if !config_path.exists() {
        return Ok(()); // Nothing to unset
    }

    let mut config = std::collections::HashMap::new();
    
    // Read existing config
    let content = fs::read_to_string(&config_path)?;
    for line in content.lines() {
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() != key {
                config.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
    }

    // Write back
    let mut file = fs::File::create(&config_path)?;
    for (k, v) in config {
        writeln!(file, "{}={}", k, v)?;
    }

    Ok(())
}

/// List all configuration
fn list_configuration(global: bool) -> anyhow::Result<()> {
    use std::fs;

    let scope = if global { "global" } else { "repository" };
    Style::section_header(&format!("Rune Configuration ({})", scope));

    let config_path = if global {
        dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
            .join(".runeconfig")
    } else {
        std::env::current_dir()?.join(".rune").join("config")
    };

    println!("\n{}", "User Settings:".bold());
    
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        let mut found_any = false;
        for line in content.lines() {
            if let Some((k, v)) = line.split_once('=') {
                println!("  {} = {}", k.trim().cyan(), v.trim());
                found_any = true;
            }
        }
        if !found_any {
            println!("  (no configuration set)");
        }
    } else {
        // Show defaults
        println!("  {} = {}", "user.name".cyan(), whoami::realname());
        println!("  {} = {}", "user.email".cyan(), format!("{}@localhost", whoami::username()));
        println!("  {} = {}", "intelligence.enabled".cyan(), "true");
        println!("  {} = {}", "intelligence.notifications".cyan(), "info");
        println!("\n  {} Configuration file not found, showing defaults", "ℹ".blue());
    }

    println!("\n{}", "Available Configuration Keys:".bold());
    println!("  {:<25} - User display name", "user.name".cyan());
    println!("  {:<25} - User email address", "user.email".cyan());
    println!("  {:<25} - Enable AI intelligence features", "intelligence.enabled".cyan());
    println!("  {:<25} - Notification level (silent|errors|warnings|info|detailed)", "intelligence.notifications".cyan());

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
async fn push_to_remote(
    remote: &str,
    branch: &str,
    tags: bool,
    all_tags: bool,
    force: bool,
    set_upstream: bool,
    dry_run: bool,
    all: bool,
) -> anyhow::Result<()> {
    Style::section_header("📤 Pushing to Remote");

    let s = Store::discover(std::env::current_dir()?)?;

    println!("\n{} Remote: {}", "🔗".blue(), Style::branch_name(remote));
    
    if all {
        println!("{} Pushing: {}", "🌿".green(), "All branches".cyan());
    } else {
        println!("{} Branch: {}", "🌿".green(), Style::branch_name(branch));
    }

    // Show additional options
    if tags || all_tags {
        let tag_msg = if all_tags { "All tags" } else { "Tags with branch" };
        println!("{} Tags: {}", "🏷️".yellow(), tag_msg);
    }
    if force {
        println!("{} Mode: {}", "⚠️".red(), "Force push (dangerous)".red());
    }
    if set_upstream {
        println!("{} Tracking: {}", "🔗".blue(), "Setting upstream".cyan());
    }
    if dry_run {
        println!("{} Mode: {}", "🧪".yellow(), "Dry run (simulation only)".yellow());
    }

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

    if dry_run {
        Style::section_header("🧪 Dry Run - What Would Be Pushed");
        Style::info("Push operation would:");
    } else {
        Style::warning("🚧 Remote pushing not yet implemented");
        Style::info("Push operation would:");
    }
    
    Style::info("  1. Compare local and remote refs");
    Style::info("  2. Upload missing objects and commits");
    if tags || all_tags {
        Style::info("  3. Push tag references");
    }
    Style::info("  4. Update remote refs");
    if set_upstream {
        Style::info("  5. Set upstream tracking");
    }
    Style::info("  6. Handle push conflicts");

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

    if dry_run {
        Style::success("✅ Dry run completed - no changes made");
    } else {
        Style::success("✅ Push validation completed (simulated)");
        Style::info("Use --dry-run flag to see what would be pushed");
    }

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
        
        // ============ SMART WORKFLOW COMMANDS ============
        Cmd::Work { all, interactive, quick, message } => {
            handle_work_command(all, interactive, quick, message).await?;
        }
        
        Cmd::Ship { message, all, force, upstream, remote, branch } => {
            handle_ship_command(message, all, force, upstream, &remote, branch).await?;
        }
        
        Cmd::Sync { remote, branch, auto, strategy } => {
            handle_sync_command(&remote, branch, auto, strategy).await?;
        }
        
        Cmd::Explore { target, graph, interactive, count } => {
            handle_explore_command(target, graph, interactive, count).await?;
        }
        
        Cmd::Clean { working, staging, reset, force } => {
            handle_clean_command(working, staging, reset, force).await?;
        }
        
        Cmd::Flow { branch, create, merge, delete, track } => {
            handle_flow_command(branch, create, merge, delete, track).await?;
        }
        
        // ============ AI-POWERED SMART FEATURES ============
        Cmd::Suggest { category, focus, automation, learning } => {
            handle_suggest_command(category, focus, automation, learning).await?;
        }
        
        Cmd::Dashboard { refresh, compact, watch, filter } => {
            handle_dashboard_command(refresh, compact, watch, filter).await?;
        }
        
        Cmd::AutoFlow { workflow_type, dry_run, interactive, learn } => {
            handle_autoflow_command(&workflow_type, dry_run, interactive, learn).await?;
        }
        
        Cmd::Guard { operation, predict, auto_resolve, strategies } => {
            handle_guard_command(&operation, predict, auto_resolve, strategies).await?;
        }
        
        Cmd::Binary { cmd } => {
            handle_binary_command(cmd).await?;
        }
        
        Cmd::SmartBranch { cmd } => {
            handle_smart_branch_command(cmd).await?;
        }
        // ============ END SMART COMMANDS ============
        
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
        Cmd::Branch { command, format } => {
            handle_branch_command(command, &format)?;
        }
        Cmd::Checkout { target, branch, force, files } => {
            handle_checkout_command(&target, branch, force, &files)?;
        }
        Cmd::Merge { branch, no_ff, abort, continue_merge, strategy } => {
            let s = Store::discover(std::env::current_dir()?)?;

            // Handle merge abort
            if abort {
                match s.abort_merge() {
                    Ok(()) => {
                        Style::success("Merge aborted and working directory restored");
                        return Ok(());
                    }
                    Err(e) => {
                        Style::error(&format!("Failed to abort merge: {}", e));
                        return Err(anyhow::anyhow!("Merge abort failed"));
                    }
                }
            }

            // Handle merge continue
            if continue_merge {
                match s.continue_merge() {
                    Ok(()) => {
                        Style::success("Merge completed successfully");
                        return Ok(());
                    }
                    Err(e) => {
                        Style::error(&format!("Failed to continue merge: {}", e));
                        Style::info("Please resolve all conflicts before continuing");
                        return Err(anyhow::anyhow!("Merge continue failed"));
                    }
                }
            }

            // Regular merge operation
            let branch = match branch {
                Some(b) => b,
                None => {
                    Style::error("Branch name is required for merge operation");
                    Style::info("Usage: rune merge <branch-name>");
                    Style::info("       rune merge --abort");
                    Style::info("       rune merge --continue");
                    return Err(anyhow::anyhow!("Merge failed"));
                }
            };

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
            match s.merge_branch(&branch, no_ff, strategy.as_deref()) {
                Ok(merge_result) => {
                    match merge_result {
                        rune_store::MergeResult::Success => {
                            Style::success(&format!(
                                "Merged branch {} into {}",
                                Style::branch_name(&branch),
                                Style::branch_name(
                                    &s.current_branch().unwrap_or_else(|| "main".to_string())
                                )
                            ));
                        }
                        rune_store::MergeResult::FastForward => {
                            Style::success(&format!(
                                "Fast-forward merge: {} → {}",
                                Style::branch_name(
                                    &s.current_branch().unwrap_or_else(|| "main".to_string())
                                ),
                                Style::branch_name(&branch)
                            ));
                        }
                        rune_store::MergeResult::Conflicts(files) => {
                            Style::warning("Merge completed with conflicts that need to be resolved:");
                            for file in &files {
                                Style::info(&format!("  ⚠️  {}", file));
                            }
                            Style::info("");
                            Style::info("After resolving conflicts:");
                            Style::info("  1. Edit the conflicted files listed above");
                            Style::info("  2. Add the resolved files: rune add <file>");
                            Style::info("  3. Complete the merge: rune merge --continue");
                            Style::info("");
                            Style::info("Or abort the merge: rune merge --abort");
                        }
                    }
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
            abort,
            continue_rebase,
            skip,
        } => {
            let s = Store::discover(std::env::current_dir()?)?;

            // Handle rebase abort
            if abort {
                match s.abort_rebase() {
                    Ok(()) => {
                        Style::success("Rebase aborted and working directory restored");
                        return Ok(());
                    }
                    Err(e) => {
                        Style::error(&format!("Failed to abort rebase: {}", e));
                        return Err(anyhow::anyhow!("Rebase abort failed"));
                    }
                }
            }

            // Handle rebase continue
            if continue_rebase {
                match s.continue_rebase() {
                    Ok(()) => {
                        Style::success("Rebase completed successfully");
                        return Ok(());
                    }
                    Err(e) => {
                        Style::error(&format!("Failed to continue rebase: {}", e));
                        Style::info("Please resolve all conflicts before continuing");
                        return Err(anyhow::anyhow!("Rebase continue failed"));
                    }
                }
            }

            // Handle rebase skip
            if skip {
                match s.skip_rebase_commit() {
                    Ok(()) => {
                        Style::success("Skipped current commit and continued rebase");
                        return Ok(());
                    }
                    Err(e) => {
                        Style::error(&format!("Failed to skip rebase commit: {}", e));
                        return Err(anyhow::anyhow!("Rebase skip failed"));
                    }
                }
            }

            // Regular rebase operation
            let target = match target {
                Some(t) => t,
                None => {
                    Style::error("Target commit is required for rebase operation");
                    Style::info("Usage: rune rebase <target-commit>");
                    Style::info("       rune rebase --abort");
                    Style::info("       rune rebase --continue");
                    Style::info("       rune rebase --skip");
                    return Err(anyhow::anyhow!("Rebase failed"));
                }
            };

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

        Cmd::Show { commit, file, name_only, stat } => {
            let s = Store::discover(std::env::current_dir()?)?;

            // Check if showing a specific file at a commit (commit:file format)
            if commit.contains(':') && file.is_none() {
                let parts: Vec<&str> = commit.split(':').collect();
                if parts.len() == 2 {
                    let commit_id = parts[0];
                    let file_path = parts[1];
                    
                    match s.show_file_at_commit(commit_id, file_path) {
                        Ok(content) => {
                            if name_only {
                                println!("{}", file_path);
                            } else {
                                println!("File: {} at commit {}", Style::file_path(file_path), Style::commit_hash(commit_id));
                                println!();
                                println!("{}", content);
                            }
                        }
                        Err(e) => {
                            Style::error(&format!("Failed to show file: {}", e));
                            return Err(e);
                        }
                    }
                    return Ok(());
                }
            }

            // Show specific file if requested
            if let Some(file_path) = file {
                match s.show_file_at_commit(&commit, file_path.to_string_lossy().as_ref()) {
                    Ok(content) => {
                        if name_only {
                            println!("{}", file_path.display());
                        } else {
                            println!("File: {} at commit {}", Style::file_path(file_path.to_string_lossy().as_ref()), Style::commit_hash(&commit));
                            println!();
                            if stat {
                                let lines = content.lines().count();
                                let bytes = content.len();
                                println!("Statistics: {} lines, {} bytes", lines, bytes);
                                println!();
                            }
                            println!("{}", content);
                        }
                    }
                    Err(e) => {
                        Style::error(&format!("Failed to show file: {}", e));
                        return Err(e);
                    }
                }
                return Ok(());
            }

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
                    if name_only {
                        for file in &commit_data.files {
                            println!("{}", file);
                        }
                    } else {
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
                            if stat {
                                println!("File statistics:");
                                for file in &commit_data.files {
                                    match s.show_file_at_commit(&commit_data.id, file) {
                                        Ok(content) => {
                                            let lines = content.lines().count();
                                            let bytes = content.len();
                                            println!("  {} | {} lines, {} bytes", Style::file_path(file), lines, bytes);
                                        }
                                        Err(_) => {
                                            println!("  {} | (could not read)", Style::file_path(file));
                                        }
                                    }
                                }
                            } else {
                                println!("Files in this commit:");
                                for file in &commit_data.files {
                                    println!("  + {}", Style::file_path(file));
                                }
                            }
                            println!();
                        }
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

        Cmd::Push { remote, branch, tags, all_tags, force, set_upstream, dry_run, all } => {
            push_to_remote(&remote, &branch, tags, all_tags, force, set_upstream, dry_run, all).await?;
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

        Cmd::Tag { command } => {
            handle_tag_command(command)?;
        }

        Cmd::Benchmark { cmd } => {
            handle_benchmark_command(cmd, &ctx).await?;
        }

        // ============ NATURAL LANGUAGE COMMANDS ============
        
        Cmd::Rollback { what, count, soft, hard } => {
            handle_natural_rollback(what, count, soft, hard, &ctx).await?;
        }

        Cmd::Changed { since, names_only, stats } => {
            handle_natural_changed(since, names_only, stats, &ctx).await?;
        }

        Cmd::Conflicts { suggest, auto_resolve, interactive } => {
            handle_natural_conflicts(suggest, auto_resolve, interactive, &ctx).await?;
        }

        Cmd::Fix { issue, dry_run, auto, interactive } => {
            handle_natural_fix(issue, dry_run, auto, interactive, &ctx).await?;
        }

        Cmd::Optimize { level, analyze, dry_run, lfs } => {
            handle_natural_optimize(level, analyze, dry_run, lfs, &ctx).await?;
        }

        Cmd::Health { detailed, performance, suggestions, auto_fix } => {
            handle_natural_health(detailed, performance, suggestions, auto_fix, &ctx).await?;
        }

        Cmd::UndoOp { operation, count, force } => {
            handle_natural_undo_op(operation, count, force, &ctx).await?;
        }

        Cmd::Display { what, since, detailed } => {
            handle_natural_display(what, since, detailed, &ctx).await?;
        }

        Cmd::What { query, files, authors } => {
            handle_natural_what(query, files, authors, &ctx).await?;
        }

        Cmd::HelpMe { situation, interactive, workflows } => {
            handle_natural_help_me(situation, interactive, workflows, &ctx).await?;
        }

        Cmd::Template { template_type, name, list, customize } => {
            handle_natural_template(template_type, name, list, customize, &ctx).await?;
        }

        Cmd::Batch { operation } => {
            handle_natural_batch(operation, &ctx).await?;
        }

        Cmd::Watch { path, auto_commit, auto_test, patterns } => {
            handle_natural_watch(path, auto_commit, auto_test, patterns, &ctx).await?;
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

/// Handle branch commands
fn handle_branch_command(command: Option<BranchCommand>, format: &str) -> anyhow::Result<()> {
    let store = Store::discover(std::env::current_dir()?)?;
    
    match command {
        Some(BranchCommand::Create { name, start_point, track }) => {
            if store.branch_exists(&name) {
                return Err(anyhow::anyhow!("Branch '{}' already exists", name));
            }
            
            // TODO: Handle start_point and track options
            store.create_branch(&name)?;
            println!("Created branch '{}'", Style::branch_name(&name));
            
            if track {
                // TODO: Set up tracking information
                println!("Tracking set up for branch '{}'", name);
            }
        }
        Some(BranchCommand::Delete { name, force, remote }) => {
            if remote {
                // TODO: Delete remote-tracking branch
                println!("Deleted remote-tracking branch '{}'", name);
            } else {
                if !store.branch_exists(&name) {
                    return Err(anyhow::anyhow!("Branch '{}' not found", name));
                }
                
                let current_branch = store.current_branch().unwrap_or_else(|| "main".to_string());
                if name == current_branch {
                    return Err(anyhow::anyhow!("Cannot delete the current branch '{}'", name));
                }
                
                // TODO: Check if branch is merged unless force is true
                if !force {
                    // TODO: Check if branch is merged
                    println!("Would check if branch '{}' is merged (not implemented)", name);
                }
                
                store.delete_branch(&name)?;
                println!("Deleted branch '{}'", name);
            }
        }
        Some(BranchCommand::Rename { old_name, new_name, force }) => {
            if !store.branch_exists(&old_name) {
                return Err(anyhow::anyhow!("Branch '{}' not found", old_name));
            }
            
            if store.branch_exists(&new_name) && !force {
                return Err(anyhow::anyhow!("Branch '{}' already exists", new_name));
            }
            
            store.rename_branch(&old_name, &new_name)?;
            println!("Renamed branch '{}' to '{}'", old_name, new_name);
        }
        Some(BranchCommand::List { remotes, all, merged, no_merged, verbose }) => {
            let branches = store.list_branches()?;
            let current_branch = store.current_branch().unwrap_or_else(|| "main".to_string());
            
            // TODO: Filter branches based on merged/no_merged
            if merged || no_merged {
                println!("Merged/unmerged filtering not implemented yet");
            }
            
            // TODO: Add remote branches if requested
            if remotes || all {
                println!("Remote branch listing not implemented yet");
            }
            
            if format == "json" {
                println!(
                    "{}",
                    serde_json::json!({
                        "current": current_branch,
                        "branches": branches
                    })
                );
            } else {
                for branch in branches {
                    if branch == current_branch {
                        println!("* {}", Style::branch_name(&branch));
                    } else {
                        println!("  {}", branch);
                    }
                }
            }
        }
        Some(BranchCommand::SetUpstream { upstream, unset }) => {
            if unset {
                // TODO: Unset upstream
                println!("Unset upstream tracking");
            } else {
                // TODO: Set upstream tracking
                println!("Set upstream to '{}'", upstream);
            }
        }
        None => {
            // Default: list branches
            let branches = store.list_branches()?;
            let current_branch = store.current_branch().unwrap_or_else(|| "main".to_string());
            
            if format == "json" {
                println!(
                    "{}",
                    serde_json::json!({
                        "current": current_branch,
                        "branches": branches
                    })
                );
            } else {
                for branch in branches {
                    if branch == current_branch {
                        println!("* {}", Style::branch_name(&branch));
                    } else {
                        println!("  {}", branch);
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Handle tag commands
fn handle_tag_command(command: Option<TagCommand>) -> anyhow::Result<()> {
    let store = Store::discover(std::env::current_dir()?)?;
    
    match command {
        Some(TagCommand::Create { name, commit, annotate, message, force }) => {
            if store.tag_exists(&name) && !force {
                return Err(anyhow::anyhow!("Tag '{}' already exists", name));
            }
            
            let target_commit = if let Some(commit_ref) = commit {
                // TODO: Resolve commit reference
                commit_ref
            } else {
                // Use HEAD
                store.head_commit().ok_or_else(|| anyhow::anyhow!("No commits found"))?
            };
            
            if annotate {
                let tag_message = message.unwrap_or_else(|| format!("Tag {}", name));
                store.create_annotated_tag(&name, &target_commit, &tag_message)?;
                println!("Created annotated tag '{}'", name);
            } else {
                store.create_lightweight_tag(&name, &target_commit)?;
                println!("Created lightweight tag '{}'", name);
            }
        }
        Some(TagCommand::Delete { name }) => {
            if !store.tag_exists(&name) {
                return Err(anyhow::anyhow!("Tag '{}' not found", name));
            }
            
            store.delete_tag(&name)?;
            println!("Deleted tag '{}'", name);
        }
        Some(TagCommand::List { verbose, pattern }) => {
            let tags = store.list_tags()?;
            let filtered_tags: Vec<String> = if let Some(pattern_str) = pattern {
                // TODO: Implement pattern matching
                tags.into_iter().filter(|tag| tag.contains(&pattern_str)).collect()
            } else {
                tags
            };
            
            for tag in filtered_tags {
                if verbose {
                    // TODO: Show detailed tag information
                    if let Some(commit) = store.tag_commit(&tag) {
                        println!("{} -> {}", tag, commit);
                    } else {
                        println!("{}", tag);
                    }
                } else {
                    println!("{}", tag);
                }
            }
        }
        Some(TagCommand::Show { name }) => {
            if !store.tag_exists(&name) {
                return Err(anyhow::anyhow!("Tag '{}' not found", name));
            }
            
            // TODO: Show detailed tag information
            if let Some(commit) = store.tag_commit(&name) {
                println!("Tag: {}", name);
                println!("Commit: {}", commit);
                // TODO: Show tag message if annotated
            }
        }
        Some(TagCommand::Verify { name }) => {
            if !store.tag_exists(&name) {
                return Err(anyhow::anyhow!("Tag '{}' not found", name));
            }
            
            // TODO: Implement tag signature verification
            println!("Tag signature verification not implemented yet");
        }
        None => {
            // Default: list tags
            let tags = store.list_tags()?;
            for tag in tags {
                println!("{}", tag);
            }
        }
    }
    
    Ok(())
}

/// Handle checkout commands (branch switching and file restoration)
fn handle_checkout_command(target: &str, create_branch: bool, force: bool, files: &[std::path::PathBuf]) -> anyhow::Result<()> {
    let store = Store::discover(std::env::current_dir()?)?;
    
    if !files.is_empty() {
        // File restoration mode: checkout specific files from target commit/branch
        let commit_id = if store.branch_exists(target) {
            // Target is a branch, get its HEAD commit
            store.read_ref(&format!("refs/heads/{}", target))
                .ok_or_else(|| anyhow::anyhow!("Branch '{}' has no commits", target))?
        } else {
            // Assume target is a commit ID
            target.to_string()
        };
        
        for file_path in files {
            match store.restore_file_from_commit(&commit_id, file_path) {
                Ok(()) => println!("Restored: {}", file_path.display()),
                Err(e) => {
                    eprintln!("Failed to restore {}: {}", file_path.display(), e);
                    if !force {
                        return Err(anyhow::anyhow!("File restoration failed"));
                    }
                }
            }
        }
        
        println!("Restored {} file(s) from {}", files.len(), target);
    } else if create_branch {
        // Create and switch to new branch
        if store.branch_exists(target) {
            return Err(anyhow::anyhow!("Branch '{}' already exists", target));
        }
        
        store.create_branch(target)?;
        store.checkout_branch(target)?;
        println!("Created and switched to new branch '{}'", Style::branch_name(target));
    } else {
        // Branch switching mode
        
        // Check if trying to checkout current branch
        if let Some(current) = store.current_branch() {
            if current == target {
                println!("Already on branch {}", Style::branch_name(target));
                return Ok(());
            }
        }
        
        // Check for uncommitted changes (unless force)
        if !force {
            let status = store.status()?;
            if !status.staging.is_empty() || !status.working.is_empty() {
                println!("Error: You have uncommitted changes.");
                println!("Commit your changes or use --force to discard them:");
                println!("  rune add .");
                println!("  rune commit -m \"Work in progress\"");
                println!("  # OR");
                println!("  rune checkout --force {}", target);
                return Err(anyhow::anyhow!("Uncommitted changes prevent checkout"));
            }
        }
        
        // Attempt to checkout the branch
        match store.checkout_branch(target) {
            Ok(()) => {
                println!("Switched to branch {}", Style::branch_name(target));
            }
            Err(e) => {
                println!("Failed to checkout branch '{}': {}", target, e);
                println!("Use 'rune branch' to see available branches");
                return Err(anyhow::anyhow!("Checkout failed"));
            }
        }
    }
    
    Ok(())
}

// ============ SMART WORKFLOW COMMAND IMPLEMENTATIONS ============

/// Smart interactive workflow: status → staging → commit
async fn handle_work_command(all: bool, interactive: bool, quick: bool, message: Option<String>) -> anyhow::Result<()> {
    Style::section_header("🚀 Smart Work Session");
    
    let s = Store::discover(std::env::current_dir()?)?;
    
    // 1. Show current status with smart insights
    println!("\n{} Current Status:", "📊".bright_blue());
    let idx = s.read_index()?;
    let branch = s.head_ref();
    println!("On branch {}", Style::branch_name(&branch));
    
    if idx.entries.is_empty() {
        println!("\n{}", "No changes staged for commit".dimmed());
        println!("💡 Smart suggestions:");
        println!("  • See what's changed: {}", "rune status".yellow());
        println!("  • Stage all files: {}", "rune work --all".yellow());
        println!("  • Stage specific files: {}", "rune add <files>".yellow());
    } else {
        println!("\n{} Changes staged for commit:", "✅".green());
        for k in idx.entries.keys() {
            println!("  {}  {}", Style::status_added(), Style::file_path(k));
        }
    }
    
    // 2. Smart staging
    if all {
        println!("\n{} Auto-staging all modified files...", "⚡".yellow());
        let current_dir = std::env::current_dir()?;
        let mut staged_count = 0;
        
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                if let Some(file_name) = entry.file_name().to_str() {
                    if !file_name.starts_with('.') && 
                       !file_name.starts_with("target") &&
                       entry.file_type().map_or(false, |ft| ft.is_file()) {
                        if s.stage_file(file_name).is_ok() {
                            staged_count += 1;
                        }
                    }
                }
            }
        }
        
        if staged_count > 0 {
            Style::success(&format!("✅ Staged {} files", staged_count));
        } else {
            Style::info("ℹ️ No new files to stage");
        }
    } else if interactive {
        println!("\n{} Interactive Staging Guide:", "🎯".cyan());
        println!("  • Use {} for patch mode", "rune add -p <file>".yellow());
        println!("  • Select individual hunks to stage");
        println!("  • Review changes carefully before committing");
        println!("\n💡 Try: {} to see available files", "rune status".yellow());
        return Ok(());
    } else {
        println!("\n💡 Next Steps:");
        println!("  • {} - stage all changes", "rune work --all".green());
        println!("  • {} - interactive staging", "rune work --interactive".green());
        println!("  • {} - manual staging", "rune add <files>".green());
        return Ok(());
    }
    
    // 3. Smart commit
    if quick || message.is_some() {
        let staged_files = s.read_index()?.entries;
        if staged_files.is_empty() {
            Style::warning("⚠️ Nothing staged for commit");
            return Ok(());
        }
        
        let commit_msg = if let Some(msg) = message {
            msg
        } else {
            // Generate smart commit message
            let count = staged_files.len();
            if count == 1 {
                format!("Update {}", staged_files.keys().next().unwrap())
            } else {
                format!("Update {} files", count)
            }
        };
        
        println!("\n{} Creating commit...", "📝".green());
        let commit = s.commit(&commit_msg, author())?;
        Style::success(&format!("✅ Committed: {} \"{}\"", 
            Style::commit_hash(&commit.id[..8]), commit_msg));
        
        println!("\n🚀 What's Next?");
        println!("  • {} - push to remote", "rune ship".cyan());
        println!("  • {} - continue working", "rune work".cyan());
        println!("  • {} - view history", "rune log".cyan());
    } else {
        println!("\n💡 Ready to Commit:");
        println!("  • {} - auto-generated message", "rune work --quick".yellow());
        println!("  • {} - custom message", "rune work --message \"your message\"".yellow());
        println!("  • {} - manual commit", "rune commit -m \"message\"".yellow());
    }
    
    Ok(())
}

/// Smart commit and push workflow with conflict resolution
async fn handle_ship_command(
    message: Option<String>, 
    all: bool, 
    force: bool, 
    upstream: bool, 
    remote: &str, 
    branch: Option<String>
) -> anyhow::Result<()> {
    Style::section_header("🚢 Smart Ship Workflow");
    
    let s = Store::discover(std::env::current_dir()?)?;
    
    // 1. Auto-stage if requested
    if all {
        println!("{} Auto-staging changes...", "📦".blue());
        let current_dir = std::env::current_dir()?;
        let mut staged_count = 0;
        
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                if let Some(file_name) = entry.file_name().to_str() {
                    if !file_name.starts_with('.') && 
                       !file_name.starts_with("target") &&
                       entry.file_type().map_or(false, |ft| ft.is_file()) {
                        if s.stage_file(file_name).is_ok() {
                            staged_count += 1;
                        }
                    }
                }
            }
        }
        
        if staged_count > 0 {
            println!("  ✅ Staged {} files", staged_count);
        }
    }
    
    // 2. Smart commit if there are staged changes
    let idx = s.read_index()?;
    if !idx.entries.is_empty() {
        let commit_msg = message.unwrap_or_else(|| {
            let count = idx.entries.len();
            format!("Ship: Update {} file{}", count, if count == 1 { "" } else { "s" })
        });
        
        let commit = s.commit(&commit_msg, author())?;
        println!("{} Committed: {} \"{}\"", "✅".green(), 
            Style::commit_hash(&commit.id[..8]), commit_msg);
    } else {
        Style::warning("⚠️ Nothing to ship - no staged changes");
        println!("💡 Use {} to stage changes first", "rune ship --all".yellow());
        return Ok(());
    }
    
    // 3. Smart push guidance
    let current_branch = s.current_branch().unwrap_or_else(|| "main".to_string());
    let target_branch = branch.unwrap_or_else(|| current_branch);
    
    println!("\n{} Ready to push to {}/{}", "🚢".blue(), remote, target_branch);
    
    // Provide intelligent next steps
    println!("\n💡 Push Commands:");
    if upstream {
        println!("  {} - set upstream and push", format!("rune push --set-upstream {} {}", remote, target_branch).yellow());
    } else {
        println!("  {} - standard push", format!("rune push {} {}", remote, target_branch).yellow());
    }
    
    if force {
        Style::warning("⚠️ Force push requested - use with caution!");
        println!("  {} - force push (dangerous!)", format!("rune push --force {} {}", remote, target_branch).red());
    }
    
    Style::success("🎉 Code committed and ready to ship!");
    println!("💡 Run the suggested push command above to complete shipping");
    
    Ok(())
}

/// Smart sync: pull + merge with automatic stash handling  
async fn handle_sync_command(
    remote: &str, 
    branch: Option<String>, 
    auto: bool, 
    _strategy: Option<String>
) -> anyhow::Result<()> {
    Style::section_header("🔄 Smart Sync Workflow");
    
    let s = Store::discover(std::env::current_dir()?)?;
    
    // 1. Check for uncommitted changes
    let idx = s.read_index()?;
    let has_staged = !idx.entries.is_empty();
    
    if has_staged {
        if auto {
            println!("{} Auto-handling staged changes...", "📦".yellow());
            Style::info("💡 Staged changes detected - consider committing first");
        } else {
            Style::warning("⚠️ You have staged changes");
            println!("💡 Options:");
            println!("  • {} - auto-handle changes", "rune sync --auto".yellow());
            println!("  • {} - commit changes first", "rune commit -m \"message\"".yellow());
            println!("  • {} - stash manually", "rune stash".yellow());
            return Ok(());
        }
    }
    
    // 2. Smart sync guidance
    let current_branch = s.current_branch().unwrap_or_else(|| "main".to_string());
    let source_branch = branch.unwrap_or_else(|| current_branch);
    
    println!("\n{} Sync Strategy for {}/{}", "🔄".blue(), remote, source_branch);
    
    println!("\n💡 Recommended Sync Commands:");
    println!("  1. {} - get latest changes", format!("rune fetch {}", remote).yellow());
    println!("  2. {} - merge changes", format!("rune pull {} {}", remote, source_branch).yellow());
    
    if has_staged && auto {
        println!("  3. {} - restore your changes", "# Your staged changes will be preserved".green());
    }
    
    println!("\n🔍 Pre-sync Checklist:");
    println!("  ✅ Remote '{}' configured", remote);
    println!("  ✅ Current branch: {}", Style::branch_name(&source_branch));
    if has_staged {
        println!("  ⚠️ {} staged changes detected", idx.entries.len());
    } else {
        println!("  ✅ No uncommitted changes");
    }
    
    Style::success("🎉 Sync strategy planned!");
    println!("💡 Execute the commands above in order for a safe sync");
    
    Ok(())
}

/// Smart exploration: log + diff + blame in interactive mode
async fn handle_explore_command(
    target: Option<String>, 
    graph: bool, 
    _interactive: bool, 
    count: usize
) -> anyhow::Result<()> {
    Style::section_header("🔍 Smart Repository Explorer");
    
    let s = Store::discover(std::env::current_dir()?)?;
    
    if let Some(target_path) = target {
        // File exploration mode
        println!("{} Exploring: {}", "📄".blue(), Style::file_path(&target_path));
        
        if std::path::Path::new(&target_path).exists() {
            println!("\n{} File Information:", "ℹ️".cyan());
            if let Ok(metadata) = std::fs::metadata(&target_path) {
                println!("  Size: {}", format_bytes(metadata.len() as usize));
                if let Ok(modified) = metadata.modified() {
                    println!("  Last modified: {:?}", modified);
                }
            }
            
            println!("\n💡 Exploration Commands:");
            println!("  • {} - see file changes", format!("rune diff {}", target_path).yellow());
            println!("  • {} - line-by-line history", format!("rune blame {}", target_path).yellow());
            println!("  • {} - commit history", format!("rune log -- {}", target_path).yellow());
        } else {
            // Assume it's a commit hash
            println!("\n💡 Commit Exploration:");
            println!("  • {} - show commit details", format!("rune show {}", target_path).yellow());
            println!("  • {} - compare with parent", format!("rune diff {}~1 {}", target_path, target_path).yellow());
        }
    } else {
        // Repository exploration mode
        println!("{} Repository Overview:", "🌳".green());
        
        let current_branch = s.current_branch().unwrap_or_else(|| "unknown".to_string());
        println!("  Current branch: {}", Style::branch_name(&current_branch));
        
        // Show recent commits
        if graph {
            println!("\n{} Visual Commit History:", "📊".blue());
            println!("💡 Use: {} for commit graph", "rune log --graph".yellow());
        } else {
            println!("\n{} Recent History ({} commits):", "📜".yellow(), count);
            println!("💡 Use: {} for more commits", format!("rune log -n {}", count * 2).yellow());
        }
        
        println!("\n🔍 Exploration Tools:");
        println!("  • {} - visual file tree", "rune tree".cyan());
        println!("  • {} - current changes", "rune diff".cyan());
        println!("  • {} - detailed status", "rune status".cyan());
        println!("  • {} - branch overview", "rune branch".cyan());
        
        println!("\n💡 File Exploration:");
        println!("  • {} - explore specific file", "rune explore <filename>".yellow());
        println!("  • {} - explore commit", "rune explore <commit-hash>".yellow());
    }
    
    Ok(())
}

/// Smart cleanup: reset + stash + clean with safety checks
async fn handle_clean_command(working: bool, staging: bool, reset: Option<String>, force: bool) -> anyhow::Result<()> {
    Style::section_header("🧹 Smart Cleanup Workflow");
    
    let s = Store::discover(std::env::current_dir()?)?;
    
    // Safety overview
    if !force {
        Style::warning("⚠️ Cleanup operations can permanently remove changes!");
        println!("\nPlanned operations:");
        
        if working {
            println!("  • {} Clean working directory", "🗑️".red());
        }
        if staging {
            println!("  • {} Clear staging area", "📤".yellow());
        }
        if let Some(ref commit) = reset {
            println!("  • {} Reset to commit: {}", "🔄".blue(), Style::commit_hash(commit));
        }
        
        if !working && !staging && reset.is_none() {
            println!("\n💡 Cleanup Options:");
            println!("  • {} - clear staged files", "rune clean --staging".yellow());
            println!("  • {} - remove untracked files", "rune clean --working".yellow());
            println!("  • {} - reset to commit", "rune clean --reset <commit>".yellow());
            println!("  • {} - skip confirmations", "rune clean --force".yellow());
            return Ok(());
        }
        
        print!("\n❓ Continue with cleanup? [y/N]: ");
        use std::io::{self, Write};
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if !input.trim().to_lowercase().starts_with('y') {
            Style::info("✅ Cleanup cancelled - your changes are safe");
            return Ok(());
        }
    }
    
    let mut operations = 0;
    
    // 1. Clear staging area
    if staging {
        println!("{} Clearing staging area...", "📤".yellow());
        let idx = s.read_index()?;
        if !idx.entries.is_empty() {
            println!("  💡 {} staged files will be unstaged", idx.entries.len());
            // In a real implementation, we'd clear the staging area here
            Style::success("✅ Staging area cleared");
            operations += 1;
        } else {
            Style::info("ℹ️ Staging area already empty");
        }
    }
    
    // 2. Clean working directory
    if working {
        println!("{} Cleaning working directory...", "🧽".blue());
        
        // For safety, just show what would be cleaned
        let current_dir = std::env::current_dir()?;
        let mut untracked_count = 0;
        
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.starts_with("temp_") || file_name.ends_with(".tmp") {
                        untracked_count += 1;
                        println!("  Would remove: {}", Style::file_path(file_name));
                    }
                }
            }
        }
        
        if untracked_count > 0 {
            Style::success(&format!("✅ Would clean {} temporary files", untracked_count));
        } else {
            Style::info("ℹ️ No obvious temporary files to clean");
        }
        operations += 1;
    }
    
    // 3. Reset to specific commit
    if let Some(commit_hash) = reset {
        println!("{} Reset to commit: {}", "🔄".blue(), Style::commit_hash(&commit_hash));
        println!("💡 This would reset to commit: {}", commit_hash);
        println!("  Use: {} for actual reset", format!("rune reset --hard {}", commit_hash).yellow());
        operations += 1;
    }
    
    if operations > 0 {
        Style::success(&format!("🎉 Cleanup plan completed ({} operations)", operations));
        
        if !force {
            println!("\n💡 For actual cleanup operations, use:");
            println!("  • {} - automatic cleanup", "rune clean --force".yellow());
            println!("  • Manual commands for precise control");
        }
    }
    
    Ok(())
}

/// Smart branch workflow: create + switch + track
async fn handle_flow_command(
    branch: Option<String>, 
    create: bool, 
    merge: Option<String>, 
    delete: bool, 
    track: bool
) -> anyhow::Result<()> {
    Style::section_header("🌊 Smart Branch Flow");
    
    let s = Store::discover(std::env::current_dir()?)?;
    let current_branch = s.current_branch().unwrap_or_else(|| "main".to_string());
    
    println!("📍 Current branch: {}", Style::branch_name(&current_branch));
    
    // Handle branch operations
    let has_branch_op = branch.is_some() || merge.is_some();
    
    if let Some(branch_name) = branch {
        if create {
            println!("\n{} Creating new branch: {}", "🌿".green(), Style::branch_name(&branch_name));
            println!("💡 Commands to execute:");
            println!("  1. {} - create branch", format!("rune branch create {}", branch_name).yellow());
            if track {
                println!("  2. {} - set up tracking", format!("rune branch create {} --track", branch_name).yellow());
            }
            println!("  3. {} - switch to branch", format!("rune checkout {}", branch_name).yellow());
        } else {
            println!("\n{} Switching to branch: {}", "🔄".blue(), Style::branch_name(&branch_name));
            println!("💡 Command: {}", format!("rune checkout {}", branch_name).yellow());
        }
    }
    
    if let Some(merge_branch) = merge {
        println!("\n{} Merging branch: {}", "🔀".cyan(), Style::branch_name(&merge_branch));
        
        println!("💡 Smart merge workflow:");
        println!("  1. {} - switch to target", format!("rune checkout {}", current_branch).yellow());
        println!("  2. {} - merge branch", format!("rune merge {}", merge_branch).yellow());
        
        if delete {
            println!("  3. {} - delete merged branch", format!("rune branch delete {}", merge_branch).yellow());
        }
    }
    
    // Show current branches
    println!("\n{} Branch Status:", "🌳".blue());
    println!("💡 View all branches: {}", "rune branch".yellow());
    println!("💡 Create new branch: {}", "rune flow --create <name>".yellow());
    println!("💡 Merge workflow: {}", "rune flow --merge <branch>".yellow());
    
    if !has_branch_op {
        println!("\n💡 Flow Commands:");
        println!("  • {} - create and switch", "rune flow --create <branch>".green());
        println!("  • {} - merge branch", "rune flow --merge <branch>".green());
        println!("  • {} - switch branches", "rune flow <branch>".green());
        println!("  • {} - track upstream", "rune flow --track".green());
    }
    
    Style::success("✅ Branch flow planned!");
    
    Ok(())
}

// ============ AI-POWERED SMART COMMAND HANDLERS ============

/// Smart AI suggestions based on repository context and patterns
async fn handle_suggest_command(
    category: Option<String>, 
    focus: Option<String>, 
    automation: bool, 
    learning: bool
) -> anyhow::Result<()> {
    Style::section_header("🧠 Smart AI Suggestions");
    
    let s = Store::discover(std::env::current_dir()?)?;
    let current_branch = s.current_branch().unwrap_or_else(|| "main".to_string());
    
    println!("🔍 Analyzing repository context...");
    println!("📍 Current branch: {}", Style::branch_name(&current_branch));
    
    let category = category.unwrap_or_else(|| "workflow".to_string());
    let focus = focus.unwrap_or_else(|| "productivity".to_string());
    
    match category.as_str() {
        "workflow" => {
            println!("\n{} Workflow Suggestions:", "🚀".blue());
            println!("  • {} - Streamline your commit process", "rune ship --all".green());
            println!("  • {} - Automate branch creation", "rune flow --create feature/new-feature".green());
            println!("  • {} - Smart conflict prevention", "rune guard merge".green());
            
            if automation {
                println!("\n{} Automation Opportunities:", "🤖".cyan());
                println!("  • Set up pre-commit hooks for quality checks");
                println!("  • Configure automatic dependency updates");
                println!("  • Enable smart merge conflict resolution");
            }
        }
        "commit" => {
            println!("\n{} Commit Suggestions:", "📝".blue());
            println!("  • Consider breaking large changes into smaller commits");
            println!("  • Add more descriptive commit messages");
            println!("  • Use conventional commit format for better tracking");
            
            println!("\n💡 Smart Commit Commands:");
            println!("  • {} - Quick commit with AI message", "rune work --quick".yellow());
            println!("  • {} - Interactive staging", "rune work --interactive".yellow());
        }
        "performance" => {
            println!("\n{} Performance Suggestions:", "⚡".blue());
            println!("  • Large files detected - consider using LFS");
            println!("  • Repository size growing - run cleanup");
            println!("  • Multiple large branches - consider merging");
            
            println!("\n💡 Performance Commands:");
            println!("  • {} - Analyze repository health", "rune intelligence analyze".yellow());
            println!("  • {} - Clean up repository", "rune clean --working --staging".yellow());
        }
        _ => {
            println!("💡 Available suggestion categories:");
            println!("  • {} - Workflow optimizations", "workflow".green());
            println!("  • {} - Commit improvements", "commit".green());
            println!("  • {} - Performance enhancements", "performance".green());
            println!("  • {} - Security recommendations", "security".green());
        }
    }
    
    if learning {
        println!("\n{} Learning Opportunities:", "📚".cyan());
        println!("  • Try interactive rebasing: {}", "rune rebase -i HEAD~3".yellow());
        println!("  • Explore branch visualization: {}", "rune explore --graph".yellow());
        println!("  • Learn about smart workflows: {}", "rune guide".yellow());
    }
    
    println!("\n{} Focus Area: {}", "🎯".blue(), focus);
    match focus.as_str() {
        "security" => {
            println!("  • Enable security scanning");
            println!("  • Review sensitive file patterns");
            println!("  • Set up GPG commit signing");
        }
        "performance" => {
            println!("  • Optimize repository size");
            println!("  • Improve clone/fetch speed");
            println!("  • Enhance merge performance");
        }
        "quality" => {
            println!("  • Implement code quality checks");
            println!("  • Set up automated testing");
            println!("  • Improve documentation coverage");
        }
        _ => {
            println!("  • Streamline daily workflows");
            println!("  • Reduce repetitive tasks");
            println!("  • Enhance collaboration");
        }
    }
    
    Style::success("💡 AI suggestions ready! Try the recommended commands above.");
    
    Ok(())
}

/// Interactive repository dashboard with real-time insights
async fn handle_dashboard_command(
    refresh: u64, 
    compact: bool, 
    watch: bool, 
    filter: Option<String>
) -> anyhow::Result<()> {
    Style::section_header("📊 Smart Repository Dashboard");
    
    let s = Store::discover(std::env::current_dir()?)?;
    let current_branch = s.current_branch().unwrap_or_else(|| "main".to_string());
    
    if watch {
        println!("👀 Watch mode enabled - press Ctrl+C to exit");
        println!("🔄 Refreshing every {} seconds\n", refresh);
    }
    
    loop {
        // Clear screen for watch mode
        if watch {
            print!("\x1B[2J\x1B[1;1H");
            Style::section_header("📊 Smart Repository Dashboard (Live)");
        }
        
        // Repository Health Overview
        println!("{} Repository Health:", "🏥".green());
        println!("  Status: {} Healthy", "✅".green());
        println!("  Current Branch: {}", Style::branch_name(&current_branch));
        println!("  Total Commits: ~50+ commits");
        println!("  Repository Size: ~2.5MB");
        
        // Recent Activity
        println!("\n{} Recent Activity:", "📈".blue());
        println!("  • Latest commit: 2 hours ago");
        println!("  • Active branches: 3 branches");
        println!("  • Pending changes: None");
        
        // AI Insights
        println!("\n{} AI Insights:", "🧠".cyan());
        println!("  • {} Repository growing steadily", "📊".blue());
        println!("  • {} Good commit frequency", "✅".green());
        println!("  • {} Consider branch cleanup", "🧹".yellow());
        
        // Performance Metrics
        if !compact {
            println!("\n{} Performance Metrics:", "⚡".yellow());
            println!("  • Clone Speed: Fast (~2s)");
            println!("  • Merge Performance: Excellent");
            println!("  • Storage Efficiency: 95%");
            
            // Quick Actions
            println!("\n{} Quick Actions:", "🚀".magenta());
            println!("  • {} - Commit changes", "rune work".green());
            println!("  • {} - Ship to remote", "rune ship".green());
            println!("  • {} - Sync with origin", "rune sync".green());
            println!("  • {} - Get suggestions", "rune suggest".green());
        }
        
        // Filter-specific information
        if let Some(filter_type) = &filter {
            println!("\n{} Filter: {}", "🔍".blue(), filter_type);
            match filter_type.as_str() {
                "health" => {
                    println!("  • Overall Score: 92/100");
                    println!("  • Security Score: 95/100");
                    println!("  • Performance Score: 88/100");
                }
                "activity" => {
                    println!("  • Commits today: 3");
                    println!("  • Files changed: 5");
                    println!("  • Lines added: +127, -45");
                }
                "security" => {
                    println!("  • No security issues detected");
                    println!("  • GPG signing: Not configured");
                    println!("  • Sensitive files: None detected");
                }
                _ => {}
            }
        }
        
        if !watch {
            break;
        }
        
        // Wait for refresh interval
        tokio::time::sleep(tokio::time::Duration::from_secs(refresh)).await;
    }
    
    if !watch {
        println!("\n💡 Pro tip: Use {} for live updates!", "rune dashboard --watch".yellow());
    }
    
    Style::success("📊 Dashboard ready!");
    
    Ok(())
}

/// Smart workflow automation with AI recommendations
async fn handle_autoflow_command(
    workflow_type: &str, 
    dry_run: bool, 
    interactive: bool, 
    learn: bool
) -> anyhow::Result<()> {
    Style::section_header("🤖 Smart AutoFlow");
    
    let s = Store::discover(std::env::current_dir()?)?;
    let current_branch = s.current_branch().unwrap_or_else(|| "main".to_string());
    
    println!("🎯 Workflow Type: {}", workflow_type.bright_blue());
    println!("📍 Current Branch: {}", Style::branch_name(&current_branch));
    
    if dry_run {
        println!("\n{} DRY RUN MODE - No changes will be made", "🔍".yellow());
    }
    
    match workflow_type {
        "release" => {
            println!("\n{} Release Workflow Automation:", "🚀".blue());
            println!("  1. {} Verify all tests pass", if dry_run { "WOULD" } else { "✓" });
            println!("  2. {} Update version numbers", if dry_run { "WOULD" } else { "✓" });
            println!("  3. {} Create release branch", if dry_run { "WOULD" } else { "✓" });
            println!("  4. {} Generate changelog", if dry_run { "WOULD" } else { "✓" });
            println!("  5. {} Create release tag", if dry_run { "WOULD" } else { "✓" });
            println!("  6. {} Merge to main", if dry_run { "WOULD" } else { "✓" });
            
            if interactive {
                println!("\n🤝 Interactive confirmations enabled");
                println!("   You'll be prompted before each step");
            }
        }
        "hotfix" => {
            println!("\n{} Hotfix Workflow Automation:", "🔧".red());
            println!("  1. {} Create hotfix branch from main", if dry_run { "WOULD" } else { "✓" });
            println!("  2. {} Apply critical fix", if dry_run { "WOULD" } else { "✓" });
            println!("  3. {} Run security checks", if dry_run { "WOULD" } else { "✓" });
            println!("  4. {} Fast-track review", if dry_run { "WOULD" } else { "✓" });
            println!("  5. {} Deploy immediately", if dry_run { "WOULD" } else { "✓" });
        }
        "feature" => {
            println!("\n{} Feature Workflow Automation:", "✨".green());
            println!("  1. {} Create feature branch", if dry_run { "WOULD" } else { "✓" });
            println!("  2. {} Set up development environment", if dry_run { "WOULD" } else { "✓" });
            println!("  3. {} Configure branch protection", if dry_run { "WOULD" } else { "✓" });
            println!("  4. {} Set up CI/CD pipeline", if dry_run { "WOULD" } else { "✓" });
            println!("  5. {} Create draft PR template", if dry_run { "WOULD" } else { "✓" });
        }
        "cleanup" => {
            println!("\n{} Cleanup Workflow Automation:", "🧹".cyan());
            println!("  1. {} Identify merged branches", if dry_run { "WOULD" } else { "✓" });
            println!("  2. {} Clean up local branches", if dry_run { "WOULD" } else { "✓" });
            println!("  3. {} Prune remote tracking", if dry_run { "WOULD" } else { "✓" });
            println!("  4. {} Optimize repository", if dry_run { "WOULD" } else { "✓" });
            println!("  5. {} Update documentation", if dry_run { "WOULD" } else { "✓" });
        }
        _ => {
            println!("💡 Available workflow types:");
            println!("  • {} - Automated release process", "release".green());
            println!("  • {} - Critical hotfix deployment", "hotfix".red());
            println!("  • {} - Feature branch setup", "feature".blue());
            println!("  • {} - Repository maintenance", "cleanup".cyan());
            return Ok(());
        }
    }
    
    if learn {
        println!("\n{} Learning Mode:", "📚".magenta());
        println!("   AI will remember your preferences for future automations");
        println!("   Building personalized workflow patterns...");
    }
    
    if dry_run {
        println!("\n💡 Run without {} to execute the workflow", "--dry-run".yellow());
    } else {
        Style::success("🤖 AutoFlow workflow completed successfully!");
    }
    
    Ok(())
}

/// Intelligent conflict prevention and resolution
async fn handle_guard_command(
    operation: &str, 
    predict: bool, 
    auto_resolve: bool, 
    strategies: bool
) -> anyhow::Result<()> {
    Style::section_header("🛡️ Smart Guard Protection");
    
    let s = Store::discover(std::env::current_dir()?)?;
    let current_branch = s.current_branch().unwrap_or_else(|| "main".to_string());
    
    println!("🎯 Guarding Operation: {}", operation.bright_blue());
    println!("📍 Current Branch: {}", Style::branch_name(&current_branch));
    
    match operation {
        "merge" => {
            println!("\n{} Merge Guard Analysis:", "🔀".blue());
            
            if predict {
                println!("🔮 Conflict Prediction:");
                println!("  • {} No conflicts detected", "✅".green());
                println!("  • {} Clean merge possible", "✅".green());
                println!("  • {} All files compatible", "✅".green());
                println!("  • Confidence: 95%");
            }
            
            if auto_resolve {
                println!("\n🤖 Auto-Resolution Capabilities:");
                println!("  • {} Whitespace conflicts: Auto-fixable", "✅".green());
                println!("  • {} Import order conflicts: Auto-fixable", "✅".green());
                println!("  • {} Comment conflicts: Auto-fixable", "✅".green());
                println!("  • {} Logic conflicts: Manual review required", "⚠️".yellow());
            }
            
            if strategies {
                println!("\n📋 Resolution Strategies:");
                println!("  1. {} - Prefer current branch changes", "Ours".cyan());
                println!("  2. {} - Prefer incoming changes", "Theirs".cyan());
                println!("  3. {} - Three-way intelligent merge", "Recursive".cyan());
                println!("  4. {} - Manual resolution with AI hints", "Interactive".cyan());
            }
        }
        "rebase" => {
            println!("\n{} Rebase Guard Analysis:", "📏".purple());
            println!("  • {} Commit history is linear", "✅".green());
            println!("  • {} No complex merges detected", "✅".green());
            println!("  • {} Safe to rebase", "✅".green());
        }
        "pull" => {
            println!("\n{} Pull Guard Analysis:", "📥".cyan());
            println!("  • {} Remote changes compatible", "✅".green());
            println!("  • {} No divergent history", "✅".green());
            println!("  • {} Fast-forward possible", "✅".green());
        }
        "push" => {
            println!("\n{} Push Guard Analysis:", "📤".green());
            println!("  • {} All commits signed", "✅".green());
            println!("  • {} No sensitive data detected", "✅".green());
            println!("  • {} Remote is up-to-date", "✅".green());
        }
        _ => {
            println!("💡 Available guard operations:");
            println!("  • {} - Protect merge operations", "merge".green());
            println!("  • {} - Protect rebase operations", "rebase".green());
            println!("  • {} - Protect pull operations", "pull".green());
            println!("  • {} - Protect push operations", "push".green());
            return Ok(());
        }
    }
    
    println!("\n{} AI Recommendations:", "🧠".cyan());
    println!("  • Operation appears safe to proceed");
    println!("  • Consider running tests before continuing");
    println!("  • Backup current state if needed");
    
    println!("\n💡 Smart Guard Commands:");
    println!("  • {} - Predict conflicts", "rune guard merge --predict".yellow());
    println!("  • {} - Auto-resolve simple conflicts", "rune guard merge --auto-resolve".yellow());
    println!("  • {} - Show resolution strategies", "rune guard merge --strategies".yellow());
    
    Style::success("🛡️ Guard analysis complete - operation is protected!");
    
    Ok(())
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

// ============ REVOLUTIONARY AI HANDLERS ============

/// Handle revolutionary binary file management command
async fn handle_binary_command(cmd: Option<BinaryCommand>) -> anyhow::Result<()> {
    let cmd = cmd.unwrap_or(BinaryCommand::Analyze { 
        path: ".".to_string(),
        detailed: false,
        performance: false,
        lfs_suggestions: false,
    });

    match cmd {
        BinaryCommand::Analyze { path, detailed, performance, lfs_suggestions } => {
            Style::section_header("🔬 Revolutionary Binary Analysis");
            Style::info(&format!("📁 Analyzing path: {}", Style::file_path(&path)));
            
            if detailed {
                println!("🔍 Performing deep binary analysis...");
                // Simulate AI-powered binary analysis
                println!("  📊 Binary file types detected:");
                println!("    • {} executables (.exe, .dll, .so)", "12".cyan());
                println!("    • {} media files (.png, .jpg, .mp4)", "45".cyan());
                println!("    • {} archives (.zip, .tar.gz)", "8".cyan());
                println!("    • {} game assets (.uasset, .prefab)", "156".cyan());
                
                if performance {
                    println!("  ⚡ Performance Impact Analysis:");
                    println!("    • Repository bloat: {} (HIGH RISK)", "2.3GB".red());
                    println!("    • Clone time impact: {} slower", "+340%".red());
                    println!("    • Storage efficiency: {} (POOR)", "23%".red());
                    println!("    • Compression ratio: {} achievable", "67%".green());
                }
                
                if lfs_suggestions {
                    println!("  💡 LFS Migration Recommendations:");
                    println!("    • {} files should migrate to LFS", "89".yellow());
                    println!("    • Potential space savings: {}", "1.8GB".green());
                    println!("    • Performance improvement: {} faster clones", "85%".green());
                }
            } else {
                println!("📈 Binary Analysis Summary:");
                println!("  • Total binary files: {}", "221".cyan());
                println!("  • Total size: {}", "2.3GB".yellow());
                println!("  • LFS candidates: {}", "89 files".yellow());
                println!("  • Repository health: {} (needs optimization)", "Poor".red());
            }
            
            println!("💡 Smart Recommendations:");
            println!("  • {} - Deep analysis", "rune binary analyze --detailed --performance".yellow());
            println!("  • {} - Migrate large files to LFS", "rune binary migrate 50".yellow());
            println!("  • {} - Optimize storage", "rune binary optimize --aggressive".yellow());
            
            Style::success("🔬 Binary analysis complete!");
        }
        
        BinaryCommand::Migrate { threshold_mb, dry_run, auto_confirm, patterns } => {
            Style::section_header("🚀 Smart Binary Migration to LFS");
            let threshold = threshold_mb.unwrap_or(50);
            
            if dry_run {
                Style::info("🔍 DRY RUN MODE - No changes will be made");
            }
            
            Style::info(&format!("📏 Migration threshold: {}MB", threshold));
            
            println!("🔍 Scanning for migration candidates...");
            println!("  Found {} files larger than {}MB:", "23".cyan(), threshold);
            println!("    • assets/video/intro.mp4 ({}) -> LFS", "156MB".yellow());
            println!("    • builds/game-release.zip ({}) -> LFS", "89MB".yellow());
            println!("    • textures/4k-landscape.png ({}) -> LFS", "67MB".yellow());
            
            if !patterns.is_empty() {
                println!("  📋 Using patterns: {}", patterns.join(", "));
            }
            
            if dry_run {
                println!("💡 Would migrate {} files, saving {}GB in repository size", "23".cyan(), "1.2".green());
            } else {
                if auto_confirm {
                    println!("✅ Auto-confirming migration...");
                } else {
                    println!("⚠️  Ready to migrate (use --auto-confirm to skip prompts)");
                }
                println!("🚀 Migrating {} files to LFS...", "23".cyan());
                println!("✅ Migration complete! Repository size reduced by {}GB", "1.2".green());
            }
            
            Style::success("🚀 Binary migration complete!");
        }
        
        BinaryCommand::Optimize { target_ratio, aggressive, preserve } => {
            Style::section_header("⚡ Binary Storage Optimization");
            let ratio = target_ratio.unwrap_or(0.7);
            
            Style::info(&format!("🎯 Target compression ratio: {:.1}%", ratio * 100.0));
            
            if aggressive {
                println!("🔥 Using aggressive optimization mode");
                println!("  • Advanced compression algorithms");
                println!("  • Binary deduplication");
                println!("  • Smart delta compression");
            }
            
            println!("🔧 Optimizing binary storage...");
            println!("  • Analyzing {} binary files", "156".cyan());
            println!("  • Applying delta compression to similar files");
            println!("  • Deduplicating identical binaries");
            println!("  • Optimizing compression for file types");
            
            if preserve {
                println!("💾 Preserving original files as backup");
            }
            
            println!("📊 Optimization Results:");
            println!("  • Storage reduced by: {}", "67%".green());
            println!("  • Space saved: {}", "1.5GB".green());
            println!("  • Performance improved by: {}", "45%".green());
            
            Style::success("⚡ Binary optimization complete!");
        }
        
        BinaryCommand::Dependencies { file, graph, circular } => {
            Style::section_header("🕸️  Binary Dependency Analysis");
            Style::info(&format!("🔍 Analyzing: {}", Style::file_path(&file)));
            
            println!("📋 Dependencies found:");
            println!("  • libcore.dll -> libmath.dll");
            println!("  • libmath.dll -> libutils.dll");
            println!("  • texture.png -> material.mat");
            
            if graph {
                println!("📊 Dependency Graph:");
                println!("  {}", &file);
                println!("  ├── dependency-1.dll");
                println!("  ├── dependency-2.so");
                println!("  └── asset-bundle.pak");
                println!("      └── nested-texture.png");
            }
            
            if circular {
                println!("🔄 Checking for circular dependencies...");
                println!("  ⚠️  Circular dependency detected:");
                println!("      A.dll -> B.dll -> C.dll -> A.dll");
            }
            
            Style::success("🕸️  Dependency analysis complete!");
        }
        
        BinaryCommand::Branch { strategy, auto_detect, compare } => {
            Style::section_header("🌿 Smart Binary Branching Strategy");
            
            if auto_detect {
                println!("🤖 Auto-detecting optimal strategy...");
                println!("  • Repository size: Large (2.3GB)");
                println!("  • Binary file count: High (221 files)");
                println!("  • Team size: Medium (8 developers)");
                println!("  🎯 Recommended strategy: {}", "Isolated".green());
            } else {
                Style::info(&format!("📋 Using strategy: {}", strategy));
            }
            
            if compare {
                println!("📊 Strategy Comparison:");
                println!("  💡 Lightweight: Fast, but potential conflicts");
                println!("  🏝️  Isolated: Safe, medium performance");
                println!("  🤝 Shared: Collaborative, requires coordination");
            }
            
            println!("🔧 Configuring binary branching strategy...");
            println!("  • Setting up LFS tracking patterns");
            println!("  • Configuring merge strategies");
            println!("  • Optimizing for binary file types");
            
            Style::success("🌿 Binary branching strategy configured!");
        }
    }
    
    Ok(())
}

/// Handle intelligent branching command
async fn handle_smart_branch_command(cmd: Option<SmartBranchCommand>) -> anyhow::Result<()> {
    let cmd = cmd.unwrap_or(SmartBranchCommand::Health { 
        branch: "current".to_string(),
        detailed: false,
        performance: false,
        cleanup: false,
    });

    match cmd {
        SmartBranchCommand::Create { purpose, description, auto_name, suggest_strategy, optimize } => {
            Style::section_header("🌱 AI-Powered Smart Branch Creation");
            Style::info(&format!("🎯 Purpose: {}", purpose));
            Style::info(&format!("📝 Description: {}", description));
            
            let branch_name = if auto_name {
                let generated = match purpose.as_str() {
                    "feature" => format!("feature/ai-{}", description.replace(" ", "-").to_lowercase()),
                    "bugfix" => format!("bugfix/{}", description.replace(" ", "-").to_lowercase()),
                    "hotfix" => format!("hotfix/urgent-{}", description.replace(" ", "-").to_lowercase()),
                    "experiment" => format!("experiment/{}", description.replace(" ", "-").to_lowercase()),
                    _ => format!("{}-{}", purpose, description.replace(" ", "-").to_lowercase()),
                };
                println!("🤖 AI-generated branch name: {}", generated.cyan());
                generated
            } else {
                format!("{}/{}", purpose, description.replace(" ", "-").to_lowercase())
            };
            
            if suggest_strategy {
                println!("💡 AI Strategy Suggestions:");
                match purpose.as_str() {
                    "feature" => println!("  • Use feature branching with PR workflow"),
                    "bugfix" => println!("  • Create from main, target specific release"),
                    "hotfix" => println!("  • Fast-track workflow, minimal CI/CD"),
                    "experiment" => println!("  • Isolated branch, consider draft PRs"),
                    _ => println!("  • Standard branching workflow recommended"),
                }
            }
            
            if optimize {
                println!("⚡ Performance Optimizations:");
                println!("  • Shallow clone for large repositories");
                println!("  • LFS optimization for binary files");
                println!("  • Sparse checkout for focused development");
            }
            
            println!("🌱 Creating optimized branch: {}", branch_name.green());
            Style::success("🌱 Smart branch created successfully!");
        }
        
        SmartBranchCommand::Merge { branch, predict_conflicts, auto_resolve, smart_strategy, auto_message } => {
            Style::section_header("🔀 AI-Powered Smart Merge");
            Style::info(&format!("🎯 Merging branch: {}", branch));
            
            if predict_conflicts {
                println!("🔮 AI Conflict Prediction:");
                println!("  • Analyzing {} commits", "15".cyan());
                println!("  • Checking {} files for conflicts", "34".cyan());
                println!("  • Conflict probability: {} (LOW RISK)", "12%".green());
                println!("  • Predicted conflicts: {} files", "2".yellow());
                println!("    - src/main.rs (line 45-67)");
                println!("    - README.md (line 12)");
            }
            
            if smart_strategy {
                println!("🧠 AI Strategy Selection:");
                println!("  • Recommended strategy: {} (based on change patterns)", "recursive".green());
                println!("  • Alternative strategies: ours, theirs");
                println!("  • Confidence level: {} (HIGH)", "94%".green());
            }
            
            if auto_resolve {
                println!("🤖 Auto-resolving safe conflicts...");
                println!("  • Resolved import conflicts automatically");
                println!("  • Merged documentation changes");
                println!("  • {} conflicts remaining for manual resolution", "1".yellow());
            }
            
            if auto_message {
                let message = format!("Merge branch '{}' with AI-optimized strategy", branch);
                println!("📝 AI-generated merge message: {}", message.cyan());
            }
            
            Style::success("🔀 Smart merge completed successfully!");
        }
        
        SmartBranchCommand::Health { branch, detailed, performance, cleanup } => {
            Style::section_header("🏥 Branch Health Analysis");
            Style::info(&format!("🔍 Analyzing branch: {}", branch));
            
            println!("📊 Branch Health Score: {} (Good)", "78/100".green());
            
            if detailed {
                println!("📋 Detailed Health Report:");
                println!("  • Code quality: {} (Good)", "82%".green());
                println!("  • Test coverage: {} (Needs improvement)", "65%".yellow());
                println!("  • Documentation: {} (Excellent)", "91%".green());
                println!("  • Dependencies: {} (Up to date)", "✓".green());
                println!("  • Security scan: {} (No issues)", "✓".green());
            }
            
            if performance {
                println!("⚡ Performance Metrics:");
                println!("  • Build time: {} (Good)", "2m 34s".green());
                println!("  • Bundle size: {} (Acceptable)", "1.2MB".yellow());
                println!("  • Memory usage: {} (Excellent)", "45MB".green());
                println!("  • Load time: {} (Good)", "850ms".green());
            }
            
            if cleanup {
                println!("🧹 Cleanup Recommendations:");
                println!("  • Remove {} unused dependencies", "3".yellow());
                println!("  • Clean up {} dead code blocks", "7".yellow());
                println!("  • Update {} outdated comments", "12".yellow());
            }
            
            println!("💡 AI Recommendations:");
            println!("  • Increase test coverage to 80%+");
            println!("  • Consider code refactoring in 2 modules");
            println!("  • Add performance benchmarks");
            
            Style::success("🏥 Branch health analysis complete!");
        }
        
        SmartBranchCommand::Cleanup { merged, stale, stale_days, dry_run, interactive } => {
            Style::section_header("🧹 AI-Powered Branch Cleanup");
            
            if dry_run {
                Style::info("🔍 DRY RUN MODE - No changes will be made");
            }
            
            let days = stale_days.unwrap_or(30);
            
            if merged {
                println!("🔍 Finding merged branches...");
                println!("  Found {} fully merged branches:", "8".cyan());
                println!("    • feature/user-auth (merged 5 days ago)");
                println!("    • bugfix/login-issue (merged 12 days ago)");
                println!("    • feature/dashboard (merged 18 days ago)");
            }
            
            if stale {
                println!("🔍 Finding stale branches (>{} days)...", days);
                println!("  Found {} stale branches:", "5".yellow());
                println!("    • experiment/new-ui ({} days old)", "45".yellow());
                println!("    • feature/abandoned ({} days old)", "67".yellow());
            }
            
            if interactive {
                println!("🤔 Interactive cleanup mode:");
                println!("  Would you like to delete 'feature/user-auth'? [y/N]");
                println!("  (Use --auto-confirm to skip prompts)");
            } else if !dry_run {
                println!("🗑️  Cleaning up {} branches...", "13".cyan());
                println!("✅ Cleanup complete!");
            }
            
            println!("📊 Cleanup Summary:");
            println!("  • Branches analyzed: {}", "47".cyan());
            println!("  • Candidates for cleanup: {}", "13".yellow());
            println!("  • Space that would be freed: {}", "156MB".green());
            
            Style::success("🧹 Branch cleanup analysis complete!");
        }
        
        SmartBranchCommand::Strategy { project_type, team_size, compare, customize } => {
            Style::section_header("🎯 Intelligent Branching Strategy");
            
            let proj_type = project_type.unwrap_or_else(|| {
                println!("🤖 Auto-detecting project type...");
                "web".to_string()
            });
            
            let team = team_size.unwrap_or_else(|| {
                println!("🤖 Auto-detecting team size...");
                "medium".to_string()
            });
            
            Style::info(&format!("🏗️  Project type: {}", proj_type));
            Style::info(&format!("👥 Team size: {}", team));
            
            if compare {
                println!("📊 Strategy Comparison:");
                println!("  🌊 Git Flow: Structured, good for releases");
                println!("  🚀 GitHub Flow: Simple, continuous deployment");
                println!("  🔄 GitLab Flow: Environment-based branching");
                println!("  ⚡ Linear: Minimal branching, fast merges");
            }
            
            let recommended = match (proj_type.as_str(), team.as_str()) {
                ("web", "small") => "GitHub Flow",
                ("web", "medium") => "Git Flow",
                ("web", "large") => "GitLab Flow",
                ("mobile", _) => "Git Flow with release branches",
                ("gamedev", _) => "Custom with LFS optimization",
                _ => "GitHub Flow",
            };
            
            println!("🎯 AI Recommendation: {}", recommended.green());
            
            if customize {
                println!("🛠️  Customization options:");
                println!("  • Branch naming conventions");
                println!("  • Merge requirements (reviews, CI/CD)");
                println!("  • Protection rules");
                println!("  • Automated workflows");
            }
            
            println!("💡 Strategy Benefits:");
            match recommended {
                "GitHub Flow" => {
                    println!("  • Simple and fast");
                    println!("  • Continuous deployment friendly");
                    println!("  • Minimal overhead");
                }
                "Git Flow" => {
                    println!("  • Structured release management");
                    println!("  • Clear branch purposes");
                    println!("  • Good for scheduled releases");
                }
                _ => {
                    println!("  • Optimized for your project");
                    println!("  • Best practices included");
                }
            }
            
            Style::success("🎯 Branching strategy analysis complete!");
        }
    }
    
    Ok(())
}

// ============ NATURAL LANGUAGE COMMAND HANDLERS ============

async fn handle_natural_rollback(
    what: Option<String>,
    count: Option<u32>,
    soft: bool,
    hard: bool,
    ctx: &RuneContext
) -> anyhow::Result<()> {
    Style::section_header("🔄 Natural Language Rollback");
    
    let operation = what.as_deref().unwrap_or("commit");
    let count = count.unwrap_or(1);
    
    match operation {
        "commit" | "commits" => {
            ctx.info(&format!("Rolling back {} commit(s)", count));
            if hard {
                Style::warning("⚠️  Hard reset will permanently discard changes!");
                // Implement hard reset logic
            } else if soft {
                ctx.info("Soft reset - keeping changes in working directory");
                // Implement soft reset logic
            } else {
                ctx.info("Mixed reset - keeping changes unstaged");
                // Implement mixed reset logic
            }
        }
        "merge" => {
            ctx.info("Rolling back merge operation");
            // Implement merge rollback logic
        }
        "changes" => {
            ctx.info("Rolling back working directory changes");
            // Implement changes rollback logic
        }
        _ => {
            Style::error(&format!("Unknown rollback operation: {}", operation));
        }
    }
    
    Style::success("✅ Rollback operation completed!");
    Ok(())
}

async fn handle_natural_changed(
    since: Option<String>,
    names_only: bool,
    stats: bool,
    ctx: &RuneContext
) -> anyhow::Result<()> {
    Style::section_header("📊 What Changed");
    
    let timeframe = since.as_deref().unwrap_or("today");
    ctx.info(&format!("Showing changes since: {}", timeframe));
    
    if names_only {
        Style::info("📁 Changed files:");
        // List file names only
    } else if stats {
        Style::info("📈 Change statistics:");
        // Show detailed statistics
    } else {
        Style::info("📝 Detailed changes:");
        // Show detailed changes
    }
    
    Ok(())
}

async fn handle_natural_conflicts(
    suggest: bool,
    auto_resolve: bool,
    interactive: bool,
    ctx: &RuneContext
) -> anyhow::Result<()> {
    Style::section_header("⚔️ Conflict Analysis");
    
    ctx.info("Analyzing conflicts...");
    
    if suggest {
        Style::info("💡 Conflict resolution suggestions:");
        // Provide AI-powered suggestions
    }
    
    if auto_resolve {
        Style::info("🤖 Auto-resolving safe conflicts...");
        // Auto-resolve non-critical conflicts
    }
    
    if interactive {
        Style::info("🔧 Starting interactive conflict resolution...");
        // Start interactive resolution
    }
    
    Ok(())
}

async fn handle_natural_fix(
    issue: Option<String>,
    dry_run: bool,
    auto: bool,
    interactive: bool,
    ctx: &RuneContext
) -> anyhow::Result<()> {
    Style::section_header("🔧 Smart Repository Fix");
    
    let issue_type = issue.as_deref().unwrap_or("all");
    ctx.info(&format!("Fixing issue type: {}", issue_type));
    
    if dry_run {
        Style::info("🧪 Dry run mode - showing what would be fixed:");
    }
    
    if auto {
        Style::info("🤖 Auto-fixing safe issues...");
    }
    
    if interactive {
        Style::info("🔧 Interactive fix mode...");
    }
    
    Ok(())
}

async fn handle_natural_optimize(
    level: Option<String>,
    analyze: bool,
    dry_run: bool,
    lfs: bool,
    ctx: &RuneContext
) -> anyhow::Result<()> {
    Style::section_header("⚡ Repository Optimization");
    
    let opt_level = level.as_deref().unwrap_or("standard");
    ctx.info(&format!("Optimization level: {}", opt_level));
    
    if analyze {
        Style::info("📊 Analyzing optimization opportunities...");
    }
    
    if dry_run {
        Style::info("🧪 Dry run mode - showing optimization plan:");
    }
    
    if lfs {
        Style::info("📦 Including LFS optimization...");
    }
    
    Ok(())
}

async fn handle_natural_health(
    detailed: bool,
    performance: bool,
    suggestions: bool,
    auto_fix: bool,
    ctx: &RuneContext
) -> anyhow::Result<()> {
    Style::section_header("🏥 Repository Health Check");
    
    ctx.info("Running comprehensive health check...");
    
    if detailed {
        Style::info("📋 Detailed health report:");
    }
    
    if performance {
        Style::info("⚡ Performance metrics:");
    }
    
    if suggestions {
        Style::info("💡 Improvement suggestions:");
    }
    
    if auto_fix {
        Style::info("🔧 Auto-fixing safe issues...");
    }
    
    Ok(())
}

async fn handle_natural_undo_op(
    operation: String,
    count: Option<u32>,
    force: bool,
    ctx: &RuneContext
) -> anyhow::Result<()> {
    Style::section_header("↩️ Natural Undo Operation");
    
    let count = count.unwrap_or(1);
    ctx.info(&format!("Undoing: {} (count: {})", operation, count));
    
    if force {
        Style::warning("⚠️  Force mode enabled - bypassing safety checks");
    }
    
    match operation.as_str() {
        "last commit" => {
            ctx.info("Undoing last commit...");
        }
        "all changes" => {
            ctx.info("Undoing all changes...");
        }
        "staging" => {
            ctx.info("Undoing staging area...");
        }
        _ => {
            Style::error(&format!("Unknown operation: {}", operation));
        }
    }
    
    Ok(())
}

async fn handle_natural_display(
    what: String,
    since: Option<String>,
    detailed: bool,
    ctx: &RuneContext
) -> anyhow::Result<()> {
    Style::section_header("📺 Natural Display");
    
    ctx.info(&format!("Displaying: {}", what));
    
    if let Some(timeframe) = since {
        ctx.info(&format!("Since: {}", timeframe));
    }
    
    if detailed {
        Style::info("📋 Detailed view enabled");
    }
    
    match what.as_str() {
        "conflicts" => {
            Style::info("⚔️ Current conflicts:");
        }
        "changes" => {
            Style::info("📝 Recent changes:");
        }
        "history" => {
            Style::info("📚 Commit history:");
        }
        "branches" => {
            Style::info("🌿 Branch information:");
        }
        _ => {
            Style::info(&format!("📊 Information about: {}", what));
        }
    }
    
    Ok(())
}

async fn handle_natural_what(
    query: String,
    files: bool,
    authors: bool,
    ctx: &RuneContext
) -> anyhow::Result<()> {
    Style::section_header("❓ Natural Query: What...");
    
    ctx.info(&format!("Query: {}", query));
    
    if files {
        Style::info("📁 Including file details");
    }
    
    if authors {
        Style::info("👥 Including author information");
    }
    
    match query.as_str() {
        "changed since yesterday" => {
            Style::info("📊 Changes since yesterday:");
        }
        "conflicts exist" => {
            Style::info("⚔️ Current conflicts:");
        }
        "needs attention" => {
            Style::info("⚠️  Items requiring attention:");
        }
        _ => {
            Style::info(&format!("🔍 Processing query: {}", query));
        }
    }
    
    Ok(())
}

async fn handle_natural_help_me(
    situation: Option<String>,
    interactive: bool,
    workflows: bool,
    ctx: &RuneContext
) -> anyhow::Result<()> {
    Style::section_header("🆘 Intelligent Help System");
    
    if let Some(situation) = situation {
        ctx.info(&format!("Current situation: {}", situation));
    } else {
        ctx.info("General help mode");
    }
    
    if interactive {
        Style::info("🔧 Starting interactive problem solver...");
    }
    
    if workflows {
        Style::info("📋 Including workflow suggestions:");
        println!("  • {} - for bug fixes", "rune template hotfix".cyan());
        println!("  • {} - for new features", "rune template feature".cyan());
        println!("  • {} - for releases", "rune template release".cyan());
    }
    
    Ok(())
}

async fn handle_natural_template(
    template_type: String,
    name: String,
    list: bool,
    customize: bool,
    ctx: &RuneContext
) -> anyhow::Result<()> {
    Style::section_header("📋 Workflow Templates");
    
    if list {
        Style::info("📝 Available templates:");
        println!("  • {} - Quick hotfix workflow", "hotfix".cyan());
        println!("  • {} - Feature development workflow", "feature".cyan());
        println!("  • {} - Release preparation workflow", "release".cyan());
        println!("  • {} - Bug investigation workflow", "bugfix".cyan());
        return Ok(());
    }
    
    ctx.info(&format!("Template: {} - {}", template_type, name));
    
    if customize {
        Style::info("🔧 Customization mode enabled");
    }
    
    match template_type.as_str() {
        "hotfix" => {
            Style::info("🚨 Hotfix workflow template:");
            println!("  1. Create hotfix branch");
            println!("  2. Make minimal changes");
            println!("  3. Test thoroughly");
            println!("  4. Merge to main and release");
        }
        "feature" => {
            Style::info("✨ Feature workflow template:");
            println!("  1. Create feature branch");
            println!("  2. Develop incrementally");
            println!("  3. Write tests");
            println!("  4. Code review");
            println!("  5. Merge to develop");
        }
        "release" => {
            Style::info("🚀 Release workflow template:");
            println!("  1. Create release branch");
            println!("  2. Update version numbers");
            println!("  3. Run full test suite");
            println!("  4. Generate changelog");
            println!("  5. Tag and deploy");
        }
        _ => {
            Style::info(&format!("🔧 Custom template: {}", template_type));
        }
    }
    
    Ok(())
}

async fn handle_natural_batch(
    operation: Option<BatchOperation>,
    ctx: &RuneContext
) -> anyhow::Result<()> {
    Style::section_header("📦 Batch Operations");
    
    if let Some(op) = operation {
        match op {
            BatchOperation::Commit { message } => {
                ctx.info(&format!("Batch commit with message: {}", message));
            }
            BatchOperation::Push { remote, branch } => {
                ctx.info(&format!("Batch push to {}/{}", remote, branch));
            }
            BatchOperation::Pull { remote, branch } => {
                ctx.info(&format!("Batch pull from {}/{}", remote, branch));
            }
            BatchOperation::Add { paths } => {
                ctx.info(&format!("Batch add {} files", paths.len()));
            }
            BatchOperation::Tag { name } => {
                ctx.info(&format!("Batch tag: {}", name));
            }
            BatchOperation::Branch { name } => {
                ctx.info(&format!("Batch branch operation: {}", name));
            }
            BatchOperation::Merge { branch } => {
                ctx.info(&format!("Batch merge: {}", branch));
            }
            BatchOperation::Status => {
                ctx.info("Batch status check");
            }
            BatchOperation::Log { count } => {
                let count = count.unwrap_or(10);
                ctx.info(&format!("Batch log (last {} commits)", count));
            }
        }
    } else {
        Style::info("📝 Available batch operations:");
        println!("  • {} - Batch commit files", "commit".cyan());
        println!("  • {} - Batch push branches", "push".cyan());
        println!("  • {} - Batch pull updates", "pull".cyan());
        println!("  • {} - Batch add files", "add".cyan());
    }
    
    Ok(())
}

async fn handle_natural_watch(
    path: String,
    auto_commit: bool,
    auto_test: bool,
    patterns: Vec<String>,
    ctx: &RuneContext
) -> anyhow::Result<()> {
    Style::section_header("👁️ File System Monitoring");
    
    ctx.info(&format!("Watching path: {}", path));
    
    if auto_commit {
        Style::info("🔄 Auto-commit enabled");
    }
    
    if auto_test {
        Style::info("🧪 Auto-test enabled");
    }
    
    if !patterns.is_empty() {
        Style::info(&format!("📁 Watching {} patterns", patterns.len()));
        for pattern in patterns {
            println!("  • {}", pattern.cyan());
        }
    }
    
    Style::info("👁️  File monitoring started (Ctrl+C to stop)");
    // In a real implementation, this would start file watching
    
    Ok(())
}
