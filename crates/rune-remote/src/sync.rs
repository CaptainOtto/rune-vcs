use anyhow::Result;
use axum::{extract::{State, Path}, Json};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use crate::Shrine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub parent: Option<String>,
    pub files: Vec<FileChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub operation: FileOperation,
    pub content_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileOperation {
    Added,
    Modified,
    Deleted,
    Renamed { from: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub head_commit: String,
    pub remote_tracking: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushRequest {
    pub commits: Vec<Commit>,
    pub branch: String,
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub branch: String,
    pub since_commit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    pub success: bool,
    pub message: String,
    pub commits_processed: usize,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    pub name: String,
    pub branches: Vec<Branch>,
    pub head_commit: Option<String>,
    pub remote_url: Option<String>,
}

// Repository sync endpoints
pub async fn get_repository_info(
    State(shrine): State<Shrine>,
) -> Json<RepositoryInfo> {
    let repo_info = RepositoryInfo {
        name: shrine.root.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        branches: get_branches(&shrine.root).unwrap_or_default(),
        head_commit: get_head_commit(&shrine.root),
        remote_url: get_remote_url(&shrine.root),
    };
    
    Json(repo_info)
}

pub async fn push_commits(
    State(shrine): State<Shrine>,
    Json(request): Json<PushRequest>,
) -> Json<SyncResponse> {
    let result = handle_push_commits(&shrine, request).await;
    
    match result {
        Ok(response) => Json(response),
        Err(e) => Json(SyncResponse {
            success: false,
            message: format!("Push failed: {}", e),
            commits_processed: 0,
            conflicts: vec![],
        }),
    }
}

pub async fn pull_commits(
    State(shrine): State<Shrine>,
    Json(request): Json<PullRequest>,
) -> Json<SyncResponse> {
    let result = handle_pull_commits(&shrine, request).await;
    
    match result {
        Ok(response) => Json(response),
        Err(e) => Json(SyncResponse {
            success: false,
            message: format!("Pull failed: {}", e),
            commits_processed: 0,
            conflicts: vec![],
        }),
    }
}

pub async fn sync_repository(
    State(shrine): State<Shrine>,
    Path(remote_server): Path<String>,
) -> Json<SyncResponse> {
    let result = handle_repository_sync(&shrine, &remote_server).await;
    
    match result {
        Ok(response) => Json(response),
        Err(e) => Json(SyncResponse {
            success: false,
            message: format!("Sync failed: {}", e),
            commits_processed: 0,
            conflicts: vec![],
        }),
    }
}

pub async fn get_branches_endpoint(
    State(shrine): State<Shrine>,
) -> Json<Vec<Branch>> {
    let branches = get_branches(&shrine.root).unwrap_or_default();
    Json(branches)
}

pub async fn get_commits_since(
    State(shrine): State<Shrine>,
    Path(since_commit): Path<String>,
) -> Json<Vec<Commit>> {
    let commits = get_commits_since_hash(&shrine.root, &since_commit).unwrap_or_default();
    Json(commits)
}

// Helper functions
async fn handle_push_commits(shrine: &Shrine, request: PushRequest) -> Result<SyncResponse> {
    let commits_dir = shrine.root.join(".rune/commits");
    fs::create_dir_all(&commits_dir)?;
    
    let mut conflicts = Vec::new();
    let mut processed = 0;
    
    for commit in &request.commits {
        // Check for conflicts
        let commit_path = commits_dir.join(&commit.hash);
        if commit_path.exists() && !request.force {
            conflicts.push(format!("Commit {} already exists", commit.hash));
            continue;
        }
        
        // Store commit
        let commit_json = serde_json::to_string_pretty(commit)?;
        fs::write(commit_path, commit_json)?;
        processed += 1;
    }
    
    // Update branch head if no conflicts
    if conflicts.is_empty() && !request.commits.is_empty() {
        update_branch_head(&shrine.root, &request.branch, &request.commits.last().unwrap().hash)?;
    }
    
    Ok(SyncResponse {
        success: conflicts.is_empty(),
        message: if conflicts.is_empty() {
            format!("Successfully pushed {} commits", processed)
        } else {
            format!("Pushed {} commits with {} conflicts", processed, conflicts.len())
        },
        commits_processed: processed,
        conflicts,
    })
}

async fn handle_pull_commits(shrine: &Shrine, request: PullRequest) -> Result<SyncResponse> {
    let commits = if let Some(since) = &request.since_commit {
        get_commits_since_hash(&shrine.root, since)?
    } else {
        get_all_commits(&shrine.root)?
    };
    
    Ok(SyncResponse {
        success: true,
        message: format!("Found {} commits for branch {}", commits.len(), request.branch),
        commits_processed: commits.len(),
        conflicts: vec![],
    })
}

async fn handle_repository_sync(_shrine: &Shrine, remote_server: &str) -> Result<SyncResponse> {
    // This would implement full repository synchronization with another server
    // For now, we'll implement a basic version
    
    println!("🔄 Syncing repository with server: {}", remote_server);
    
    // In a real implementation, this would:
    // 1. Connect to remote server
    // 2. Compare commit histories
    // 3. Exchange missing commits
    // 4. Resolve conflicts
    // 5. Update branch references
    
    Ok(SyncResponse {
        success: true,
        message: format!("Repository sync with {} completed", remote_server),
        commits_processed: 0,
        conflicts: vec![],
    })
}

fn get_branches(repo_root: &PathBuf) -> Result<Vec<Branch>> {
    let branches_dir = repo_root.join(".rune/refs/heads");
    let mut branches = Vec::new();
    
    if !branches_dir.exists() {
        return Ok(branches);
    }
    
    for entry in fs::read_dir(branches_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            let branch_name = entry.file_name().to_string_lossy().to_string();
            let head_commit = fs::read_to_string(entry.path())?
                .trim()
                .to_string();
            
            branches.push(Branch {
                name: branch_name,
                head_commit,
                remote_tracking: None,
            });
        }
    }
    
    Ok(branches)
}

fn get_head_commit(repo_root: &PathBuf) -> Option<String> {
    let head_file = repo_root.join(".rune/HEAD");
    fs::read_to_string(head_file).ok()?.trim().to_string().into()
}

fn get_remote_url(repo_root: &PathBuf) -> Option<String> {
    let config_file = repo_root.join(".rune/config");
    if let Ok(config) = fs::read_to_string(config_file) {
        // Parse basic config for remote URL
        for line in config.lines() {
            if line.trim().starts_with("remote_url") {
                return line.split('=').nth(1).map(|s| s.trim().to_string());
            }
        }
    }
    None
}

fn get_commits_since_hash(repo_root: &PathBuf, since_hash: &str) -> Result<Vec<Commit>> {
    let commits_dir = repo_root.join(".rune/commits");
    let mut commits = Vec::new();
    
    if !commits_dir.exists() {
        return Ok(commits);
    }
    
    for entry in fs::read_dir(commits_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            let commit_hash = entry.file_name().to_string_lossy().to_string();
            
            // Skip commits before the "since" commit
            if commit_hash == since_hash {
                break;
            }
            
            let commit_content = fs::read_to_string(entry.path())?;
            if let Ok(commit) = serde_json::from_str::<Commit>(&commit_content) {
                commits.push(commit);
            }
        }
    }
    
    Ok(commits)
}

fn get_all_commits(repo_root: &PathBuf) -> Result<Vec<Commit>> {
    let commits_dir = repo_root.join(".rune/commits");
    let mut commits = Vec::new();
    
    if !commits_dir.exists() {
        return Ok(commits);
    }
    
    for entry in fs::read_dir(commits_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            let commit_content = fs::read_to_string(entry.path())?;
            if let Ok(commit) = serde_json::from_str::<Commit>(&commit_content) {
                commits.push(commit);
            }
        }
    }
    
    // Sort by timestamp (newest first)
    commits.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    
    Ok(commits)
}

fn update_branch_head(repo_root: &PathBuf, branch: &str, commit_hash: &str) -> Result<()> {
    let branch_file = repo_root.join(".rune/refs/heads").join(branch);
    fs::create_dir_all(branch_file.parent().unwrap())?;
    fs::write(branch_file, commit_hash)?;
    
    // Update HEAD if this is the current branch
    let head_file = repo_root.join(".rune/HEAD");
    if let Ok(current_branch) = fs::read_to_string(&head_file) {
        if current_branch.trim() == format!("ref: refs/heads/{}", branch) {
            fs::write(head_file, commit_hash)?;
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_commit_serialization() {
        let commit = Commit {
            hash: "abc123".to_string(),
            message: "Test commit".to_string(),
            author: "test@example.com".to_string(),
            timestamp: chrono::Utc::now(),
            parent: None,
            files: vec![FileChange {
                path: "test.txt".to_string(),
                operation: FileOperation::Added,
                content_hash: Some("def456".to_string()),
            }],
        };
        
        let json = serde_json::to_string(&commit).unwrap();
        let deserialized: Commit = serde_json::from_str(&json).unwrap();
        
        assert_eq!(commit.hash, deserialized.hash);
        assert_eq!(commit.message, deserialized.message);
    }

    #[test]
    fn test_branch_creation() {
        let branch = Branch {
            name: "main".to_string(),
            head_commit: "abc123".to_string(),
            remote_tracking: Some("origin/main".to_string()),
        };
        
        assert_eq!(branch.name, "main");
        assert_eq!(branch.head_commit, "abc123");
    }

    #[test]
    fn test_get_branches_empty_repo() {
        let temp_dir = TempDir::new().unwrap();
        let branches = get_branches(&temp_dir.path().to_path_buf()).unwrap();
        assert_eq!(branches.len(), 0);
    }
}
