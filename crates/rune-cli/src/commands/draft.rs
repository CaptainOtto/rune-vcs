use anyhow::Result;
use clap::{ArgAction, Args, Subcommand, ValueEnum};
use rune_draft::DraftManager;
use rune_store::Store;
use crate::style::Style;

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}

#[derive(Debug, Args)]
pub struct DraftArgs {
    #[command(subcommand)]
    pub command: DraftCmd,
}

#[derive(Debug, Subcommand)]
pub enum DraftCmd {
    /// Create a new draft from current working directory changes
    Create {
        /// Name for the draft
        name: String,
        /// Optional description
        #[arg(short, long)]
        description: Option<String>,
        /// Tags to apply to the draft
        #[arg(short, long, action = ArgAction::Append)]
        tags: Vec<String>,
    },
    /// List all drafts
    List {
        /// Filter by tags
        #[arg(short, long, action = ArgAction::Append)]
        tags: Vec<String>,
        /// Show only active drafts
        #[arg(short, long)]
        active: bool,
        /// Output format
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
    /// Apply a draft to the working directory
    Apply {
        /// Draft ID or name to apply
        draft: String,
    },
    /// Shelve (remove) an active draft from working directory
    Shelve {
        /// Draft ID or name to shelve
        draft: String,
    },
    /// Update an existing draft with current changes
    Update {
        /// Draft ID or name to update
        draft: String,
    },
    /// Delete a draft permanently
    Delete {
        /// Draft ID or name to delete
        draft: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
    /// Show detailed information about a draft
    Show {
        /// Draft ID or name to show
        draft: String,
        /// Show file contents
        #[arg(short, long)]
        content: bool,
    },
    /// Create an automatic checkpoint
    Checkpoint {
        /// Optional name for the checkpoint
        name: Option<String>,
    },
    /// Clean up old drafts
    Cleanup {
        /// Days to keep drafts (overrides config)
        #[arg(long)]
        keep_days: Option<u32>,
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
    /// Add tags to a draft
    Tag {
        /// Draft ID or name
        draft: String,
        /// Tags to add
        #[arg(action = ArgAction::Append)]
        tags: Vec<String>,
    },
    /// Remove tags from a draft
    Untag {
        /// Draft ID or name
        draft: String,
        /// Tags to remove
        #[arg(action = ArgAction::Append)]
        tags: Vec<String>,
    },
}

pub fn execute_draft_command(args: DraftArgs) -> Result<()> {
    let store = Store::discover(&std::env::current_dir()?)?;
    let mut draft_manager = DraftManager::new(store)?;

    match args.command {
        DraftCmd::Create { name, description, tags } => {
            let draft_id = draft_manager.create_draft(name.clone(), description)?;
            
            if !tags.is_empty() {
                draft_manager.add_tags(&draft_id, tags)?;
            }
            
            Style::success(&format!("Created draft '{}' ({})", name, &draft_id[..8]));
        }

        DraftCmd::List { tags, active, format } => {
            let drafts = draft_manager.list_drafts()?;
            
            let filtered_drafts: Vec<_> = drafts
                .into_iter()
                .filter(|d| {
                    if active && !d.is_active {
                        return false;
                    }
                    if !tags.is_empty() {
                        return tags.iter().any(|tag| d.tags.contains(tag));
                    }
                    true
                })
                .collect();

            match format {
                OutputFormat::Table => {
                    if filtered_drafts.is_empty() {
                        println!("No drafts found");
                        return Ok(());
                    }

                    println!("{:<10} {:<20} {:<15} {:<8} {:<12} {}",
                        "ID", "Name", "Author", "Files", "Created", "Tags");
                    println!("{}", "-".repeat(80));

                    for draft in filtered_drafts {
                        let id_short = &draft.id[..8];
                        let created = draft.created_at.format("%Y-%m-%d").to_string();
                        let active_marker = if draft.is_active { "●" } else { " " };
                        let tags_str = draft.tags.join(", ");

                        println!("{}{} {:<20} {:<15} {:<8} {:<12} {}",
                            active_marker,
                            id_short,
                            draft.name,
                            draft.author.name,
                            draft.files.len(),
                            created,
                            tags_str
                        );
                    }
                }
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string_pretty(&filtered_drafts)?);
                }
            }
        }

        DraftCmd::Apply { draft } => {
            let draft_id = resolve_draft_identifier(&draft_manager, &draft)?;
            draft_manager.apply_draft(&draft_id)?;
            Style::success(&format!("Applied draft '{}'", draft));
        }

        DraftCmd::Shelve { draft } => {
            let draft_id = resolve_draft_identifier(&draft_manager, &draft)?;
            draft_manager.shelve_draft(&draft_id)?;
            Style::success(&format!("Shelved draft '{}'", draft));
        }

        DraftCmd::Update { draft } => {
            let draft_id = resolve_draft_identifier(&draft_manager, &draft)?;
            draft_manager.update_draft(&draft_id)?;
            Style::success(&format!("Updated draft '{}'", draft));
        }

        DraftCmd::Delete { draft, force } => {
            let draft_id = resolve_draft_identifier(&draft_manager, &draft)?;
            let draft_info = draft_manager.get_draft(&draft_id)?;

            if !force {
                print!("Delete draft '{}' permanently? [y/N]: ", draft_info.name);
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().to_lowercase().starts_with('y') {
                    println!("Cancelled");
                    return Ok(());
                }
            }

            draft_manager.delete_draft(&draft_id)?;
            Style::success(&format!("Deleted draft '{}'", draft_info.name));
        }

        DraftCmd::Show { draft, content } => {
            let draft_id = resolve_draft_identifier(&draft_manager, &draft)?;
            let draft_info = draft_manager.get_draft(&draft_id)?;

            println!("Draft: {}", draft_info.name);
            println!("ID: {}", draft_info.id);
            if let Some(desc) = &draft_info.description {
                println!("Description: {}", desc);
            }
            println!("Author: {}", draft_info.author.name);
            println!("Created: {}", draft_info.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
            println!("Updated: {}", draft_info.updated_at.format("%Y-%m-%d %H:%M:%S UTC"));
            println!("Base Branch: {}", draft_info.base_branch);
            println!("Base Commit: {}", draft_info.base_commit);
            println!("Active: {}", if draft_info.is_active { "Yes" } else { "No" });
            println!("Tags: {}", draft_info.tags.join(", "));
            println!("Files ({}):", draft_info.files.len());

            for (path, file) in &draft_info.files {
                let status = if file.is_deleted {
                    "D"
                } else if file.is_new {
                    "A"
                } else {
                    "M"
                };
                
                println!("  {} {}", status, path.display());
                
                if content && !file.is_deleted {
                    println!("    Content ({} bytes):", file.content.len());
                    if let Ok(text) = String::from_utf8(file.content.clone()) {
                        for line in text.lines().take(10) {
                            println!("    | {}", line);
                        }
                        if text.lines().count() > 10 {
                            println!("    | ... ({} more lines)", text.lines().count() - 10);
                        }
                    } else {
                        println!("    | (binary content)");
                    }
                }
            }
        }

        DraftCmd::Checkpoint { name } => {
            let draft_id = draft_manager.create_checkpoint(name)?;
            Style::success(&format!("Created checkpoint ({})", &draft_id[..8]));
        }

        DraftCmd::Cleanup { keep_days, force } => {
            let drafts = draft_manager.list_drafts()?;
            let cleanup_days = keep_days.unwrap_or(draft_manager.config().auto_cleanup_days);
            
            let old_drafts: Vec<_> = drafts
                .iter()
                .filter(|d| {
                    let cutoff = chrono::Utc::now() - chrono::Duration::days(cleanup_days as i64);
                    d.created_at < cutoff && !d.is_active
                })
                .collect();

            if old_drafts.is_empty() {
                println!("No old drafts to clean up");
                return Ok(());
            }

            if !force {
                println!("Will delete {} old drafts:", old_drafts.len());
                for draft in &old_drafts {
                    println!("  - {} ({})", draft.name, draft.created_at.format("%Y-%m-%d"));
                }
                print!("Continue? [y/N]: ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().to_lowercase().starts_with('y') {
                    println!("Cancelled");
                    return Ok(());
                }
            }

            let cleaned = draft_manager.cleanup_old_drafts()?;
            Style::success(&format!("Cleaned up {} old drafts", cleaned));
        }

        DraftCmd::Tag { draft, tags } => {
            let draft_id = resolve_draft_identifier(&draft_manager, &draft)?;
            draft_manager.add_tags(&draft_id, tags.clone())?;
            Style::success(&format!("Added tags to draft '{}': {}", draft, tags.join(", ")));
        }

        DraftCmd::Untag { draft, tags } => {
            let draft_id = resolve_draft_identifier(&draft_manager, &draft)?;
            draft_manager.remove_tags(&draft_id, tags.clone())?;
            Style::success(&format!("Removed tags from draft '{}': {}", draft, tags.join(", ")));
        }
    }

    Ok(())
}

/// Resolve a draft identifier (name or ID) to a full draft ID
fn resolve_draft_identifier(manager: &DraftManager, identifier: &str) -> Result<String> {
    let drafts = manager.list_drafts()?;
    
    // First try exact ID match
    if drafts.iter().any(|d| d.id == identifier) {
        return Ok(identifier.to_string());
    }
    
    // Try partial ID match
    if let Some(draft) = drafts.iter().find(|d| d.id.starts_with(identifier)) {
        return Ok(draft.id.clone());
    }
    
    // Try name match
    if let Some(draft) = drafts.iter().find(|d| d.name == identifier) {
        return Ok(draft.id.clone());
    }
    
    anyhow::bail!("No draft found with identifier '{}'", identifier);
}
