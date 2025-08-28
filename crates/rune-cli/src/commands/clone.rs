use anyhow::Result;
use clap::Args;
use rune_remote::RemoteCommands;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct CloneArgs {
    /// Repository URL to clone
    pub url: String,
    /// Local directory name (optional)
    pub directory: Option<String>,
    /// Authentication token
    #[arg(short, long)]
    pub token: Option<String>,
    /// Clone recursively (for submodules)
    #[arg(short, long)]
    pub recursive: bool,
    /// Create a bare repository
    #[arg(long)]
    pub bare: bool,
    /// Branch to clone
    #[arg(short, long)]
    pub branch: Option<String>,
    /// Depth of history to clone
    #[arg(long)]
    pub depth: Option<u32>,
}

pub async fn handle_clone_command(args: CloneArgs) -> Result<()> {
    println!("🚀 Cloning repository from: {}", args.url);
    
    // Determine local directory name
    let local_dir = if let Some(dir) = args.directory {
        dir
    } else {
        // Extract repository name from URL
        extract_repo_name(&args.url)?
    };
    
    let local_path = PathBuf::from(&local_dir);
    
    // Check if directory already exists
    if local_path.exists() {
        anyhow::bail!("Directory '{}' already exists", local_dir);
    }
    
    // Clone the repository
    RemoteCommands::clone(&args.url, Some(&local_dir), args.token).await?;
    
    println!("✅ Repository cloned successfully into '{}'", local_dir);
    println!("");
    println!("Next steps:");
    println!("  cd {}", local_dir);
    println!("  rune status");
    
    Ok(())
}

fn extract_repo_name(url: &str) -> Result<String> {
    // Handle various URL formats:
    // - https://git.example.com/user/repo.git
    // - https://git.example.com/user/repo
    // - git@git.example.com:user/repo.git
    // - /path/to/local/repo
    
    let path_part = if url.contains("://") {
        // HTTP/HTTPS URL
        url.split("://").nth(1).unwrap_or(url)
    } else if url.contains('@') {
        // SSH URL
        url.split(':').nth(1).unwrap_or(url)
    } else {
        // Local path
        url
    };
    
    let name = path_part
        .split('/')
        .last()
        .unwrap_or("repository")
        .trim_end_matches(".git");
    
    if name.is_empty() {
        anyhow::bail!("Could not determine repository name from URL: {}", url);
    }
    
    Ok(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_repo_name() {
        assert_eq!(extract_repo_name("https://git.example.com/user/repo.git").unwrap(), "repo");
        assert_eq!(extract_repo_name("https://git.example.com/user/repo").unwrap(), "repo");
        assert_eq!(extract_repo_name("git@git.example.com:user/repo.git").unwrap(), "repo");
        assert_eq!(extract_repo_name("/path/to/local/repo").unwrap(), "repo");
        assert_eq!(extract_repo_name("my-project").unwrap(), "my-project");
    }
}
