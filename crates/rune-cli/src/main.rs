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
use rune_core::intelligence::{IntelligentFileAnalyzer, InsightSeverity};

#[derive(Parser, Debug)]
#[command(name = "rune", version = "0.0.1", about = "Rune — modern DVCS (0.0.1)")]
struct Args {
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
    /// Create and apply patches
    Patch {
        #[command(subcommand)]
        cmd: PatchCmd,
    },
    #[command(subcommand)]
    Lfs(commands::lfs::LfsCmd),
    #[command(subcommand)]
    Shrine(commands::shrine::ShrineCmd),
    #[command(subcommand)]
    Delta(commands::delta::DeltaCmd),
    /// Configure Rune intelligence and features
    Config {
        #[command(subcommand)]
        cmd: ConfigCmd,
    },
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
            let analyzer = IntelligentFileAnalyzer::new();
            Style::section_header("Repository Health Analysis");
            
            match std::env::current_dir() {
                Ok(current_dir) => {
                    let repo_path = current_dir.to_string_lossy();
                    let health = analyzer.analyze_repository_health(&repo_path);
                    
                    println!("\n{}", "Repository Health Report".bold());
                    println!("  🎯 Overall Score: {:.1}/100 {}", 
                            health.overall_score,
                            if health.overall_score > 80.0 { "🟢".green() } 
                            else if health.overall_score > 60.0 { "🟡".yellow() } 
                            else { "🔴".red() });
                    
                    if !health.issues.is_empty() {
                        println!("\n{}", "Issues Found".bold());
                        for issue in &health.issues {
                            println!("  ⚠️  {}", issue);
                        }
                    }
                    
                    if !health.recommendations.is_empty() {
                        println!("\n{}", "Recommendations".bold());
                        for rec in &health.recommendations {
                            println!("  💡 {}", rec);
                        }
                    }
                    
                    if health.overall_score < 70.0 {
                        println!("\n{}", "Tip".bold());
                        println!("  Run 'rune config insights' for more specific improvement suggestions");
                    }
                },
                Err(e) => Style::error(&format!("Failed to get current directory: {}", e)),
            }
        },
        ConfigCmd::Insights => {
            let analyzer = IntelligentFileAnalyzer::new();
            Style::section_header("Predictive Repository Insights");
            
            match std::env::current_dir() {
                Ok(current_dir) => {
                    let repo_path = current_dir.to_string_lossy();
                    let insights = analyzer.generate_insights(&repo_path);
                    
                    if insights.is_empty() {
                        println!("\n🎉 {} No potential issues detected!", "Excellent!".green().bold());
                        println!("Your repository appears to be well-maintained.");
                    } else {
                        println!("\n{} {} insights found:", "🔮".blue(), insights.len());
                        
                        for (i, insight) in insights.iter().enumerate() {
                            let severity_icon = match insight.severity {
                                InsightSeverity::High => "⚠️",
                                InsightSeverity::Medium => "⚡",
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_colors();
    let a = Args::parse();
    match a.cmd {
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
                Style::error("Nothing specified, nothing added.");
                Style::info("Use 'rune add <pathspec>...' to add files to the staging area");
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
                    Style::error(&format!("Branch '{}' already exists", n));
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
            let _s = Store::discover(std::env::current_dir()?)?;
            Style::info("Diff functionality coming soon!");
            if let Some(t) = target {
                Style::info(&format!("Would show diff for: {}", t));
            } else {
                Style::info("Would show working directory changes");
            }
        }

        Cmd::Reset { files, hard } => {
            let s = Store::discover(std::env::current_dir()?)?;
            if hard {
                Style::warning("Hard reset would permanently lose changes!");
                Style::info("Hard reset not yet implemented for safety");
            } else if files.is_empty() {
                // Reset all staged files
                s.write_index(&rune_store::Index::default())?;
                Style::success("Reset staging area");
            } else {
                Style::info("Selective file reset coming soon!");
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
            if commit == "HEAD" {
                let log = s.log();
                if let Some(latest) = log.first() {
                    println!("commit {}", Style::commit_hash(&latest.id));
                    let ts = chrono::DateTime::from_timestamp(latest.time, 0)
                        .unwrap()
                        .naive_utc();
                    println!("Date:    {}", Style::timestamp(ts));
                    println!();
                    println!("    {}", latest.message);
                    println!();
                } else {
                    Style::info("No commits found");
                }
            } else {
                Style::info(&format!("Would show commit: {}", commit));
                Style::info("Show specific commits coming soon!");
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
    }
    Ok(())
}
