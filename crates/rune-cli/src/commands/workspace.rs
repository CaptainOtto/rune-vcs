use anyhow::Result;
use clap::Subcommand;
use rune_workspace::WorkspaceManager;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum WorkspaceCmd {
    /// Initialize virtual workspace
    Init {
        #[arg(help = "Workspace name")]
        name: String,
    },
    /// Add virtual root to workspace
    AddRoot {
        #[arg(help = "Virtual root name")]
        name: String,
        #[arg(help = "Path relative to repository root")]
        path: PathBuf,
        #[arg(long, help = "Include patterns (e.g., *.rs, src/**)", action = clap::ArgAction::Append)]
        include: Vec<String>,
    },
    /// Remove virtual root
    RemoveRoot {
        #[arg(help = "Virtual root name")]
        name: String,
    },
    /// List virtual roots
    List,
    /// Activate/deactivate virtual root
    Toggle {
        #[arg(help = "Virtual root name")]
        name: String,
        #[arg(long, help = "Activate the virtual root")]
        activate: bool,
        #[arg(long, help = "Deactivate the virtual root")]
        deactivate: bool,
    },
    /// Show current workspace view (files that would be included)
    View {
        #[arg(long, help = "Show file count only")]
        count_only: bool,
    },
    /// Add global include pattern
    Include {
        #[arg(help = "Pattern to include (e.g., *.rs, src/**)", action = clap::ArgAction::Append)]
        patterns: Vec<String>,
    },
    /// Add global exclude pattern
    Exclude {
        #[arg(help = "Pattern to exclude (e.g., target/**, *.tmp)", action = clap::ArgAction::Append)]
        patterns: Vec<String>,
    },
    /// Validate files against performance guardrails
    Validate {
        #[arg(help = "Files to validate")]
        files: Vec<PathBuf>,
    },
    /// Configure performance limits
    Limits {
        #[arg(long, help = "Maximum file size in MB")]
        max_file_size: Option<u64>,
        #[arg(long, help = "Maximum files per commit")]
        max_files: Option<usize>,
        #[arg(long, help = "Maximum binary files per commit")]
        max_binary_files: Option<usize>,
        #[arg(long, help = "Warning threshold for file size in MB")]
        warn_size: Option<u64>,
        #[arg(long, help = "Show current limits")]
        show: bool,
    },
}

pub fn run(cmd: WorkspaceCmd) -> Result<()> {
    let current_dir = std::env::current_dir()?;

    match cmd {
        WorkspaceCmd::Init { name } => {
            let mut workspace = WorkspaceManager::new(current_dir, name.clone())?;
            workspace.save()?;
            println!("✓ Initialized virtual workspace: {}", name);
            println!("  Configuration saved to: .rune/workspace/config.json");
        }

        WorkspaceCmd::AddRoot { name, path, include } => {
            let mut workspace = WorkspaceManager::load(current_dir)?;
            let patterns = if include.is_empty() {
                vec!["**/*".to_string()]
            } else {
                include
            };
            workspace.add_virtual_root(name, path, patterns)?;
        }

        WorkspaceCmd::RemoveRoot { name } => {
            let mut workspace = WorkspaceManager::load(current_dir)?;
            workspace.remove_virtual_root(&name)?;
        }

        WorkspaceCmd::List => {
            let workspace = WorkspaceManager::load(current_dir)?;
            let roots = workspace.list_virtual_roots();
            
            if roots.is_empty() {
                println!("No virtual roots configured");
                return Ok(());
            }

            println!("📁 Virtual Roots:");
            for (name, root) in roots {
                let status = if root.active { "✓ Active" } else { "⏸ Inactive" };
                println!("  {} {} -> {}", status, name, root.path.display());
                
                if !root.include_patterns.is_empty() {
                    println!("    Include: {}", root.include_patterns.join(", "));
                }
                if !root.exclude_patterns.is_empty() {
                    println!("    Exclude: {}", root.exclude_patterns.join(", "));
                }
                if !root.dependencies.is_empty() {
                    println!("    Dependencies: {}", root.dependencies.join(", "));
                }
            }
        }

        WorkspaceCmd::Toggle { name, activate, deactivate } => {
            let mut workspace = WorkspaceManager::load(current_dir)?;
            
            if activate && deactivate {
                anyhow::bail!("Cannot both activate and deactivate");
            }
            
            if activate {
                workspace.set_virtual_root_active(&name, true)?;
            } else if deactivate {
                workspace.set_virtual_root_active(&name, false)?;
            } else {
                anyhow::bail!("Must specify either --activate or --deactivate");
            }
        }

        WorkspaceCmd::View { count_only } => {
            let workspace = WorkspaceManager::load(current_dir)?;
            let files = workspace.get_workspace_files()?;
            
            if count_only {
                println!("📊 Workspace contains {} files", files.len());
            } else {
                println!("📁 Workspace view ({} files):", files.len());
                let mut sorted_files: Vec<_> = files.into_iter().collect();
                sorted_files.sort();
                
                for file in sorted_files.iter().take(50) {
                    println!("  {}", file.display());
                }
                
                if sorted_files.len() > 50 {
                    println!("  ... and {} more files", sorted_files.len() - 50);
                }
            }
        }

        WorkspaceCmd::Include { patterns } => {
            let mut workspace = WorkspaceManager::load(current_dir)?;
            for pattern in patterns {
                workspace.add_include_pattern(pattern)?;
            }
        }

        WorkspaceCmd::Exclude { patterns } => {
            let mut workspace = WorkspaceManager::load(current_dir)?;
            for pattern in patterns {
                workspace.add_exclude_pattern(pattern)?;
            }
        }

        WorkspaceCmd::Validate { files } => {
            let workspace = WorkspaceManager::load(current_dir)?;
            let validation = workspace.validate_commit_files(&files)?;
            
            println!("📋 Validation Results:");
            println!("  Files: {}", validation.file_count);
            println!("  Binary files: {}", validation.binary_count);
            
            if !validation.warnings.is_empty() {
                println!("\n⚠️  Warnings:");
                for warning in &validation.warnings {
                    println!("  {}", warning);
                }
            }
            
            if !validation.errors.is_empty() {
                println!("\n❌ Errors:");
                for error in &validation.errors {
                    println!("  {}", error);
                }
            }
            
            if validation.valid {
                println!("\n✅ All files pass validation");
            } else {
                println!("\n❌ Validation failed - fix errors before committing");
                std::process::exit(1);
            }
        }

        WorkspaceCmd::Limits { 
            max_file_size, 
            max_files, 
            max_binary_files, 
            warn_size, 
            show 
        } => {
            let mut workspace = WorkspaceManager::load(current_dir)?;
            
            if show {
                let limits = &workspace.config.performance_limits;
                println!("📊 Performance Limits:");
                println!("  Max file size: {} MB", limits.max_file_size_mb);
                println!("  Warning file size: {} MB", limits.warn_file_size_mb);
                println!("  Max files per commit: {}", limits.max_files_per_commit);
                println!("  Max binary files per commit: {}", limits.max_binary_files_per_commit);
                println!("  Blocked extensions: {}", limits.blocked_extensions.join(", "));
                println!("  Tracked extensions: {}", limits.tracked_extensions.join(", "));
                return Ok(());
            }
            
            let mut limits = workspace.config.performance_limits.clone();
            let mut updated = false;
            
            if let Some(size) = max_file_size {
                limits.max_file_size_mb = size;
                updated = true;
            }
            
            if let Some(files) = max_files {
                limits.max_files_per_commit = files;
                updated = true;
            }
            
            if let Some(binary_files) = max_binary_files {
                limits.max_binary_files_per_commit = binary_files;
                updated = true;
            }
            
            if let Some(warn) = warn_size {
                limits.warn_file_size_mb = warn;
                updated = true;
            }
            
            if updated {
                workspace.update_performance_limits(limits)?;
            } else {
                println!("No limits specified. Use --show to see current limits.");
            }
        }
    }

    Ok(())
}
