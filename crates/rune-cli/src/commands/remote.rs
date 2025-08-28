use anyhow::Result;
use clap::{Args, Subcommand};
use rune_remote::{RemoteCommands, RemoteManager};
use std::path::Path;

#[derive(Debug, Args)]
pub struct RemoteArgs {
    #[command(subcommand)]
    pub command: RemoteCommand,
}

#[derive(Debug, Subcommand, Clone)]
pub enum RemoteCommand {
    /// Add a remote repository
    Add {
        /// Remote name (e.g., 'origin')
        name: String,
        /// Remote URL (e.g., 'https://git.example.com/repo')
        url: String,
        /// Authentication token
        #[arg(short, long)]
        token: Option<String>,
    },
    /// Remove a remote repository
    Remove {
        /// Remote name to remove
        name: String,
    },
    /// List remote repositories
    List {
        /// Show URLs (verbose output)
        #[arg(short, long)]
        verbose: bool,
    },
    /// Set remote URL
    SetUrl {
        /// Remote name
        name: String,
        /// New URL
        url: String,
    },
    /// Set remote authentication
    SetAuth {
        /// Remote name
        name: String,
        /// Authentication token
        token: String,
    },
    /// Test remote connection
    Test {
        /// Remote name (tests all if not specified)
        name: Option<String>,
    },
    /// Show remote information
    Show {
        /// Remote name
        name: String,
    },
}

pub async fn handle_remote_command(args: RemoteArgs) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    
    // Check if we're in a Rune repository
    if !current_dir.join(".rune").exists() {
        anyhow::bail!("Not in a Rune repository. Use 'rune init' to create one.");
    }
    
    match args.command {
        RemoteCommand::Add { name, url, token } => {
            RemoteCommands::add(&current_dir, &name, &url, token)?;
        }
        RemoteCommand::Remove { name } => {
            RemoteCommands::remove(&current_dir, &name)?;
        }
        RemoteCommand::List { verbose } => {
            RemoteCommands::list(&current_dir, verbose)?;
        }
        RemoteCommand::SetUrl { name, url } => {
            RemoteCommands::set_url(&current_dir, &name, &url)?;
        }
        RemoteCommand::SetAuth { name, token } => {
            RemoteCommands::set_auth(&current_dir, &name, &token)?;
        }
        RemoteCommand::Test { name } => {
            RemoteCommands::test(&current_dir, name.as_deref()).await?;
        }
        RemoteCommand::Show { name } => {
            show_remote_info(&current_dir, &name)?;
        }
    }
    
    Ok(())
}

fn show_remote_info(repo_path: &Path, name: &str) -> Result<()> {
    let manager = RemoteManager::new(repo_path)?;
    
    match manager.get_remote(name) {
        Some(remote) => {
            println!("Remote: {}", remote.name);
            println!("  URL: {}", remote.url);
            if let Some(push_url) = &remote.push_url {
                println!("  Push URL: {}", push_url);
            }
            println!("  Default: {}", if remote.default { "yes" } else { "no" });
            println!("  Authentication: {}", if remote.token.is_some() { "configured" } else { "none" });
            println!("  Fetch refs:");
            for ref_spec in &remote.fetch_refs {
                println!("    {}", ref_spec);
            }
            println!("  Push refs:");
            for ref_spec in &remote.push_refs {
                println!("    {}", ref_spec);
            }
        }
        None => {
            anyhow::bail!("Remote '{}' not found", name);
        }
    }
    
    Ok(())
}
