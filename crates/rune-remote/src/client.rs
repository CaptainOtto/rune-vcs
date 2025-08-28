use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Remote server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    pub name: String,
    pub url: String,
    pub token: Option<String>,
    pub default: bool,
    pub push_url: Option<String>,
    pub fetch_refs: Vec<String>,
    pub push_refs: Vec<String>,
}

impl Default for RemoteConfig {
    fn default() -> Self {
        Self {
            name: "origin".to_string(),
            url: String::new(),
            token: None,
            default: true,
            push_url: None,
            fetch_refs: vec!["+refs/heads/*:refs/remotes/origin/*".to_string()],
            push_refs: vec!["refs/heads/*:refs/heads/*".to_string()],
        }
    }
}

/// Remote configuration manager
#[derive(Debug)]
pub struct RemoteManager {
    config_path: PathBuf,
    remotes: HashMap<String, RemoteConfig>,
}

impl RemoteManager {
    pub fn new(repo_path: &std::path::Path) -> Result<Self> {
        let config_path = repo_path.join(".rune").join("remotes.yml");
        let mut manager = Self {
            config_path,
            remotes: HashMap::new(),
        };
        
        if manager.config_path.exists() {
            manager.load_config()?;
        }
        
        Ok(manager)
    }
    
    /// Load remote configuration from file
    pub fn load_config(&mut self) -> Result<()> {
        let content = fs::read_to_string(&self.config_path)
            .context("Failed to read remote configuration")?;
        
        self.remotes = serde_yaml::from_str(&content)
            .context("Failed to parse remote configuration")?;
        
        Ok(())
    }
    
    /// Save remote configuration to file
    pub fn save_config(&self) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create configuration directory")?;
        }
        
        let content = serde_yaml::to_string(&self.remotes)
            .context("Failed to serialize remote configuration")?;
        
        fs::write(&self.config_path, content)
            .context("Failed to write remote configuration")?;
        
        Ok(())
    }
    
    /// Add a new remote
    pub fn add_remote(&mut self, name: &str, url: &str, token: Option<String>) -> Result<()> {
        if self.remotes.contains_key(name) {
            anyhow::bail!("Remote '{}' already exists", name);
        }
        
        let mut remote = RemoteConfig {
            name: name.to_string(),
            url: url.to_string(),
            token,
            default: self.remotes.is_empty(), // First remote becomes default
            ..Default::default()
        };
        
        // Auto-configure fetch refs based on remote name
        remote.fetch_refs = vec![format!("+refs/heads/*:refs/remotes/{}/*", name)];
        
        self.remotes.insert(name.to_string(), remote);
        self.save_config()?;
        
        Ok(())
    }
    
    /// Remove a remote
    pub fn remove_remote(&mut self, name: &str) -> Result<()> {
        if !self.remotes.contains_key(name) {
            anyhow::bail!("Remote '{}' does not exist", name);
        }
        
        let was_default = self.remotes.get(name).unwrap().default;
        self.remotes.remove(name);
        
        // If we removed the default remote, make another one default
        if was_default && !self.remotes.is_empty() {
            let first_name = self.remotes.keys().next().unwrap().clone();
            self.remotes.get_mut(&first_name).unwrap().default = true;
        }
        
        self.save_config()?;
        Ok(())
    }
    
    /// Get a remote by name
    pub fn get_remote(&self, name: &str) -> Option<&RemoteConfig> {
        self.remotes.get(name)
    }
    
    /// Get the default remote
    pub fn get_default_remote(&self) -> Option<&RemoteConfig> {
        self.remotes.values().find(|r| r.default)
    }
    
    /// List all remotes
    pub fn list_remotes(&self) -> Vec<&RemoteConfig> {
        self.remotes.values().collect()
    }
    
    /// Set remote URL
    pub fn set_remote_url(&mut self, name: &str, url: &str) -> Result<()> {
        match self.remotes.get_mut(name) {
            Some(remote) => {
                remote.url = url.to_string();
                self.save_config()?;
                Ok(())
            }
            None => anyhow::bail!("Remote '{}' does not exist", name),
        }
    }
    
    /// Set remote authentication token
    pub fn set_remote_token(&mut self, name: &str, token: Option<String>) -> Result<()> {
        match self.remotes.get_mut(name) {
            Some(remote) => {
                remote.token = token;
                self.save_config()?;
                Ok(())
            }
            None => anyhow::bail!("Remote '{}' does not exist", name),
        }
    }
    
    /// Set default remote
    pub fn set_default_remote(&mut self, name: &str) -> Result<()> {
        if !self.remotes.contains_key(name) {
            anyhow::bail!("Remote '{}' does not exist", name);
        }
        
        // Clear existing default
        for remote in self.remotes.values_mut() {
            remote.default = false;
        }
        
        // Set new default
        self.remotes.get_mut(name).unwrap().default = true;
        self.save_config()?;
        
        Ok(())
    }
    
    /// Test remote connection
    pub async fn test_remote(&self, name: &str) -> Result<bool> {
        let remote = self.get_remote(name)
            .ok_or_else(|| anyhow::anyhow!("Remote '{}' does not exist", name))?;
        
        let client = reqwest::Client::new();
        let mut request = client.get(&format!("{}/sync/info", remote.url));
        
        if let Some(token) = &remote.token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }
        
        match request.send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
    
    /// Clone from remote URL
    pub async fn clone_from_url(url: &str, local_path: &std::path::Path, token: Option<String>) -> Result<()> {
        let client = reqwest::Client::new();
        let mut request = client.get(&format!("{}/repositories", url));
        
        if let Some(token) = &token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }
        
        let response = request.send().await
            .context("Failed to connect to remote server")?;
        
        if !response.status().is_success() {
            anyhow::bail!("Failed to access remote repository: {}", response.status());
        }
        
        // Create local repository
        std::fs::create_dir_all(local_path)
            .context("Failed to create local repository directory")?;
        
        // Initialize remote configuration
        let mut remote_manager = RemoteManager::new(local_path)?;
        remote_manager.add_remote("origin", url, token)?;
        
        Ok(())
    }
}

/// Git-like remote commands
pub struct RemoteCommands;

impl RemoteCommands {
    /// Add remote (like `git remote add`)
    pub fn add(repo_path: &std::path::Path, name: &str, url: &str, token: Option<String>) -> Result<()> {
        let mut manager = RemoteManager::new(repo_path)?;
        manager.add_remote(name, url, token)?;
        println!("Added remote '{}': {}", name, url);
        Ok(())
    }
    
    /// Remove remote (like `git remote remove`)
    pub fn remove(repo_path: &std::path::Path, name: &str) -> Result<()> {
        let mut manager = RemoteManager::new(repo_path)?;
        manager.remove_remote(name)?;
        println!("Removed remote '{}'", name);
        Ok(())
    }
    
    /// List remotes (like `git remote -v`)
    pub fn list(repo_path: &std::path::Path, verbose: bool) -> Result<()> {
        let manager = RemoteManager::new(repo_path)?;
        let remotes = manager.list_remotes();
        
        if remotes.is_empty() {
            println!("No remotes configured");
            return Ok(());
        }
        
        for remote in remotes {
            if verbose {
                println!("{}\t{} (fetch)", remote.name, remote.url);
                let push_url = remote.push_url.as_ref().unwrap_or(&remote.url);
                println!("{}\t{} (push)", remote.name, push_url);
                if remote.default {
                    println!("\t* default remote");
                }
            } else {
                let marker = if remote.default { "*" } else { " " };
                println!("{} {}", marker, remote.name);
            }
        }
        
        Ok(())
    }
    
    /// Set remote URL (like `git remote set-url`)
    pub fn set_url(repo_path: &std::path::Path, name: &str, url: &str) -> Result<()> {
        let mut manager = RemoteManager::new(repo_path)?;
        manager.set_remote_url(name, url)?;
        println!("Changed remote '{}' URL to: {}", name, url);
        Ok(())
    }
    
    /// Set remote authentication
    pub fn set_auth(repo_path: &std::path::Path, name: &str, token: &str) -> Result<()> {
        let mut manager = RemoteManager::new(repo_path)?;
        manager.set_remote_token(name, Some(token.to_string()))?;
        println!("Set authentication token for remote '{}'", name);
        Ok(())
    }
    
    /// Test remote connection
    pub async fn test(repo_path: &std::path::Path, name: Option<&str>) -> Result<()> {
        let manager = RemoteManager::new(repo_path)?;
        
        let remotes_to_test = if let Some(name) = name {
            vec![manager.get_remote(name).ok_or_else(|| anyhow::anyhow!("Remote '{}' not found", name))?]
        } else {
            manager.list_remotes()
        };
        
        for remote in remotes_to_test {
            print!("Testing remote '{}'... ", remote.name);
            match manager.test_remote(&remote.name).await {
                Ok(true) => println!("✓ Connected"),
                Ok(false) => println!("✗ Failed to connect"),
                Err(e) => println!("✗ Error: {}", e),
            }
        }
        
        Ok(())
    }
    
    /// Clone repository (like `git clone`)
    pub async fn clone(url: &str, path: Option<&str>, token: Option<String>) -> Result<()> {
        let local_path = if let Some(path) = path {
            std::path::PathBuf::from(path)
        } else {
            // Extract repository name from URL
            let name = url.split('/').last().unwrap_or("repository");
            std::path::PathBuf::from(name)
        };
        
        println!("Cloning into '{}'...", local_path.display());
        RemoteManager::clone_from_url(url, &local_path, token).await?;
        println!("Repository cloned successfully");
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_remote_management() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path();
        
        let mut manager = RemoteManager::new(repo_path)?;
        
        // Add remote
        manager.add_remote("origin", "https://git.example.com/repo.git", Some("token123".to_string()))?;
        assert_eq!(manager.list_remotes().len(), 1);
        
        // Get remote
        let remote = manager.get_remote("origin").unwrap();
        assert_eq!(remote.url, "https://git.example.com/repo.git");
        assert_eq!(remote.token, Some("token123".to_string()));
        assert!(remote.default);
        
        // Add second remote
        manager.add_remote("upstream", "https://git.example.com/upstream.git", None)?;
        assert_eq!(manager.list_remotes().len(), 2);
        
        // Remove remote
        manager.remove_remote("origin")?;
        assert_eq!(manager.list_remotes().len(), 1);
        assert!(manager.get_remote("upstream").unwrap().default);
        
        Ok(())
    }
}
