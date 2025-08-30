use anyhow::Result;
use chrono::Utc;
use rune_core::{Author, Commit};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};
// ...existing code...

#[derive(Default, Serialize, Deserialize)]
pub struct Index {
    pub entries: BTreeMap<String, i64>,
} // path -> mtime

#[derive(Debug, Clone)]
pub struct Status {
    pub staging: Vec<String>,
    pub working: Vec<String>,
}

/// Result of a merge operation
#[derive(Debug, Clone)]
pub enum MergeResult {
    /// Merge completed successfully with a merge commit
    Success,
    /// Fast-forward merge completed (no merge commit needed)
    FastForward,
    /// Merge has conflicts that need to be resolved
    Conflicts(Vec<String>),
}

pub struct Store {
    pub root: PathBuf,
    pub rune_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuneConfig {
    #[serde(default)]
    pub core: CoreCfg,
    #[serde(default)]
    pub lfs: LfsCfg,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreCfg {
    #[serde(default = "def_branch")]
    pub default_branch: String,
}

impl Default for CoreCfg {
    fn default() -> Self {
        Self {
            default_branch: def_branch(),
        }
    }
}

fn def_branch() -> String {
    "main".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LfsCfg {
    #[serde(default = "def_chunk")]
    pub chunk_size: usize,
    pub remote: Option<String>,
    #[serde(default)]
    pub track: Vec<TrackCfg>,
}

impl Default for LfsCfg {
    fn default() -> Self {
        Self {
            chunk_size: def_chunk(),
            remote: None,
            track: Vec::new(),
        }
    }
}
fn def_chunk() -> usize {
    8 * 1024 * 1024
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackCfg {
    pub pattern: String,
}

impl Store {
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let rd = root.join(".rune");
        fs::create_dir_all(rd.join("objects"))?;
        Ok(Self { root, rune_dir: rd })
    }
    pub fn discover(start: impl AsRef<Path>) -> Result<Self> {
        let mut cur = Some(start.as_ref());
        while let Some(d) = cur {
            let rd = d.join(".rune");
            if rd.exists() {
                return Self::open(d);
            }
            cur = d.parent();
        }
        anyhow::bail!("not a rune repo (no .rune found)")
    }

    pub fn config_path(&self) -> PathBuf {
        self.rune_dir.join("config.toml")
    }
    pub fn config(&self) -> RuneConfig {
        let p = self.config_path();
        if let Ok(s) = fs::read_to_string(p) {
            toml::from_str(&s).unwrap_or_else(|_| RuneConfig {
                core: CoreCfg::default(),
                lfs: LfsCfg::default(),
            })
        } else {
            RuneConfig {
                core: CoreCfg::default(),
                lfs: LfsCfg::default(),
            }
        }
    }
    pub fn write_config(&self, cfg: &RuneConfig) -> anyhow::Result<()> {
        fs::write(self.config_path(), toml::to_string_pretty(cfg)?)?;
        Ok(())
    }

    pub fn head_ref(&self) -> String {
        fs::read_to_string(self.rune_dir.join("HEAD"))
            .ok()
            .and_then(|s| s.strip_prefix("ref: ").map(|x| x.trim().to_string()))
            .unwrap_or_else(|| "refs/heads/main".to_string())
    }
    pub fn set_head(&self, r: &str) -> Result<()> {
        fs::write(self.rune_dir.join("HEAD"), format!("ref: {}", r))?;
        Ok(())
    }
    pub fn read_ref(&self, r: &str) -> Option<String> {
        fs::read_to_string(self.rune_dir.join(r))
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()) // Filter out empty strings
    }
    pub fn write_ref(&self, r: &str, id: &str) -> Result<()> {
        let p = self.rune_dir.join(r);
        if let Some(pp) = p.parent() {
            fs::create_dir_all(pp)?;
        }
        fs::write(p, id.as_bytes())?;
        Ok(())
    }

    /// Create a new branch pointing to the current HEAD
    pub fn create_branch(&self, name: &str) -> Result<()> {
        let current_head = self.head_ref();
        let current_commit_id = self.read_ref(&current_head)
            .ok_or_else(|| anyhow::anyhow!("Current branch has no commits"))?;
        
        let branch_ref = format!("refs/heads/{}", name);
        self.write_ref(&branch_ref, &current_commit_id)?;
        Ok(())
    }

    /// List all branches
    pub fn list_branches(&self) -> Result<Vec<String>> {
        let mut branches = Vec::new();
        let heads_dir = self.rune_dir.join("refs/heads");
        
        if heads_dir.exists() {
            for entry in walkdir::WalkDir::new(&heads_dir) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    // Get the relative path from refs/heads/ to get the full branch name
                    if let Ok(relative_path) = entry.path().strip_prefix(&heads_dir) {
                        branches.push(relative_path.to_string_lossy().to_string());
                    }
                }
            }
        }
        
        Ok(branches)
    }

    /// Check if a branch exists
    pub fn branch_exists(&self, name: &str) -> bool {
        let branch_ref = format!("refs/heads/{}", name);
        self.read_ref(&branch_ref).is_some()
    }

    /// Checkout (switch to) a branch
    pub fn checkout_branch(&self, name: &str) -> Result<()> {
        let branch_ref = format!("refs/heads/{}", name);
        
        // Check if branch exists
        if !self.branch_exists(name) {
            return Err(anyhow::anyhow!("Branch '{}' does not exist", name));
        }
        
        // Set HEAD to point to the new branch
        self.set_head(&branch_ref)?;
        Ok(())
    }

    /// Get the current branch name from HEAD
    pub fn current_branch(&self) -> Option<String> {
        let head_ref = self.head_ref();
        if head_ref.starts_with("refs/heads/") {
            Some(head_ref.strip_prefix("refs/heads/")?.to_string())
        } else {
            None
        }
    }

    /// Get repository status (staging and working directory changes)
    pub fn status(&self) -> Result<Status> {
        let index = self.read_index().unwrap_or_default();
        let mut staging = Vec::new();
        let mut working = Vec::new();
        
        // Check staged files
        for (path, _) in &index.entries {
            staging.push(path.clone());
        }
        
        // Check working directory for modifications
        // This is a simplified implementation
        for entry in walkdir::WalkDir::new(&self.root) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let file_path = entry.path();
                if let Ok(relative_path) = file_path.strip_prefix(&self.root) {
                    let relative_str = relative_path.to_string_lossy().to_string();
                    
                    // Skip .rune directory
                    if relative_str.starts_with(".rune") {
                        continue;
                    }
                    
                    // Check if file is modified but not staged
                    if !staging.contains(&relative_str) {
                        working.push(relative_str);
                    }
                }
            }
        }
        
        Ok(Status { staging, working })
    }

    /// Merge a branch into the current branch
    pub fn merge_branch(&self, branch_name: &str, no_ff: bool, strategy: Option<&str>) -> Result<MergeResult> {
        let current_branch = self.current_branch()
            .ok_or_else(|| anyhow::anyhow!("Not on a branch"))?;
        
        let current_commit_id = self.read_ref(&format!("refs/heads/{}", current_branch))
            .ok_or_else(|| anyhow::anyhow!("Current branch has no commits"))?;
        
        let merge_commit_id = self.read_ref(&format!("refs/heads/{}", branch_name))
            .ok_or_else(|| anyhow::anyhow!("Branch '{}' has no commits", branch_name))?;
        
        // Check if this is a fast-forward merge (merge commit is ahead of current)
        let is_fast_forward = self.is_ancestor(&current_commit_id, &merge_commit_id)?;
        
        // Check for uncommitted changes
        let status = self.status()?;
        if !status.working.is_empty() || !status.staging.is_empty() {
            return Err(anyhow::anyhow!(
                "Please commit or stash your changes before merging.\nUncommitted changes in working directory"
            ));
        }
        
        if is_fast_forward && !no_ff {
            // Fast-forward merge: just update the current branch to point to the merge commit
            self.write_ref(&format!("refs/heads/{}", current_branch), &merge_commit_id)?;
            return Ok(MergeResult::FastForward);
        } else {
            // Check for potential conflicts before starting merge
            let conflicts = self.detect_merge_conflicts(&current_commit_id, &merge_commit_id)?;
            
            if !conflicts.is_empty() {
                // Save merge state for abort/continue
                self.save_merge_state(branch_name, &current_commit_id, &merge_commit_id, strategy)?;
                // Apply conflicted files to working directory
                self.apply_merge_conflicts(&conflicts)?;
                return Ok(MergeResult::Conflicts(conflicts));
            }
            
            // Create a merge commit (no conflicts)
            let mut message = format!("Merge branch '{}' into {}", branch_name, current_branch);
            if let Some(strat) = strategy {
                message.push_str(&format!(" (strategy: {})", strat));
            }
            
            let merge_commit = self.create_merge_commit(&current_commit_id, &merge_commit_id, &message)?;
            self.write_ref(&format!("refs/heads/{}", current_branch), &merge_commit)?;
            return Ok(MergeResult::Success);
        }
    }

    /// Check if commit_a is an ancestor of commit_b (for fast-forward detection)
    fn is_ancestor(&self, commit_a: &str, commit_b: &str) -> Result<bool> {
        // For now, we'll implement a simple check
        // In a real implementation, we'd traverse the commit graph
        Ok(commit_a != commit_b) // Simplified: if they're different, assume fast-forward possible
    }

    /// Create a merge commit with two parents
    fn create_merge_commit(&self, parent1: &str, _parent2: &str, message: &str) -> Result<String> {
        use chrono::Utc;
        use std::io::Write;
        
        // Get current index (staged files) - for merge, we'll use current files
        let index = self.read_index().unwrap_or_default();
        let current_branch = self.current_branch().unwrap_or_else(|| "main".to_string());
        
        // Create a simple author (in a real implementation, this would come from config)
        let author = Author {
            name: "Rune User".to_string(),
            email: "user@example.com".to_string(),
        };
        
        let files = index.entries.keys().cloned().collect::<Vec<_>>();
        let hash = blake3::hash(
            format!(
                "{}{}{:?}{}",
                message,
                author.email,
                files,
                Utc::now().timestamp()
            )
            .as_bytes(),
        );
        let id = hex::encode(hash.as_bytes());
        
        // Create commit with the merge parent (parent1 is current, parent2 is merged branch)
        // Note: The current Commit struct only supports one parent, so we'll use parent1
        // and record the merge in the message. TODO: Extend Commit to support multiple parents
        let c = Commit {
            id: id.clone(),
            message: message.to_string(),
            author,
            time: Utc::now().timestamp(),
            parent: Some(parent1.to_string()),
            files,
            branch: format!("refs/heads/{}", current_branch),
        };
        
        // Write commit to log
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.rune_dir.join("log.jsonl"))?;
        writeln!(f, "{}", serde_json::to_string(&c)?)?;
        
        Ok(id)
    }

    /// Delete a branch
    pub fn delete_branch(&self, name: &str) -> Result<()> {
        let branch_ref = format!("refs/heads/{}", name);
        let branch_file = self.rune_dir.join(&branch_ref);
        
        if !branch_file.exists() {
            return Err(anyhow::anyhow!("Branch '{}' does not exist", name));
        }
        
        std::fs::remove_file(branch_file)?;
        Ok(())
    }

    /// Rename a branch
    pub fn rename_branch(&self, old_name: &str, new_name: &str) -> Result<()> {
        let old_ref = format!("refs/heads/{}", old_name);
        let new_ref = format!("refs/heads/{}", new_name);
        let old_file = self.rune_dir.join(&old_ref);
        let new_file = self.rune_dir.join(&new_ref);
        
        if !old_file.exists() {
            return Err(anyhow::anyhow!("Branch '{}' does not exist", old_name));
        }
        
        // Ensure directory exists for new branch
        if let Some(parent) = new_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Copy the branch reference
        std::fs::copy(&old_file, &new_file)?;
        std::fs::remove_file(old_file)?;
        
        // Update HEAD if we're renaming the current branch
        if let Some(current) = self.current_branch() {
            if current == old_name {
                self.set_head(&new_ref)?;
            }
        }
        
        Ok(())
    }

    /// Get the HEAD commit ID
    pub fn head_commit(&self) -> Option<String> {
        let head_ref = self.head_ref();
        self.read_ref(&head_ref)
    }

    /// Check if a tag exists
    pub fn tag_exists(&self, name: &str) -> bool {
        let tag_file = self.rune_dir.join("refs/tags").join(name);
        tag_file.exists()
    }

    /// Create a lightweight tag
    pub fn create_lightweight_tag(&self, name: &str, commit: &str) -> Result<()> {
        let tags_dir = self.rune_dir.join("refs/tags");
        std::fs::create_dir_all(&tags_dir)?;
        
        let tag_file = tags_dir.join(name);
        std::fs::write(tag_file, commit)?;
        Ok(())
    }

    /// Create an annotated tag
    pub fn create_annotated_tag(&self, name: &str, commit: &str, message: &str) -> Result<()> {
        let tags_dir = self.rune_dir.join("refs/tags");
        std::fs::create_dir_all(&tags_dir)?;
        
        // For now, we'll store annotated tags the same as lightweight tags
        // In a full implementation, we'd create a tag object with the message
        let tag_file = tags_dir.join(name);
        std::fs::write(tag_file, format!("{}\n{}", commit, message))?;
        Ok(())
    }

    /// Delete a tag
    pub fn delete_tag(&self, name: &str) -> Result<()> {
        let tag_file = self.rune_dir.join("refs/tags").join(name);
        
        if !tag_file.exists() {
            return Err(anyhow::anyhow!("Tag '{}' does not exist", name));
        }
        
        std::fs::remove_file(tag_file)?;
        Ok(())
    }

    /// List all tags
    pub fn list_tags(&self) -> Result<Vec<String>> {
        let mut tags = Vec::new();
        let tags_dir = self.rune_dir.join("refs/tags");
        
        if tags_dir.exists() {
            for entry in std::fs::read_dir(tags_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    tags.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
        
        tags.sort();
        Ok(tags)
    }

    /// Get the commit ID that a tag points to
    pub fn tag_commit(&self, name: &str) -> Option<String> {
        let tag_file = self.rune_dir.join("refs/tags").join(name);
        
        if let Ok(content) = std::fs::read_to_string(tag_file) {
            // For lightweight tags, the file contains just the commit ID
            // For annotated tags, the first line is the commit ID
            Some(content.lines().next()?.to_string())
        } else {
            None
        }
    }

    /// Show differences between working directory and staging area, or between commits
    pub fn diff(&self, target: Option<&str>) -> Result<String> {
        if let Some(target) = target {
            if target.contains("..") {
                // Commit range diff (e.g., "commit1..commit2")
                let parts: Vec<&str> = target.split("..").collect();
                if parts.len() == 2 {
                    self.diff_commits(parts[0], parts[1])
                } else {
                    Err(anyhow::anyhow!("Invalid range format. Use commit1..commit2"))
                }
            } else {
                // Single commit diff (show changes from parent to this commit)
                self.diff_commit(target)
            }
        } else {
            // Working directory diff
            self.diff_working_directory()
        }
    }

    /// Show differences between working directory and the latest commit
    fn diff_working_directory(&self) -> Result<String> {
        let mut diff_output = String::new();
        let current_branch = self.head_ref();
        let latest_commit_id = self.read_ref(&current_branch);
        
        if latest_commit_id.is_none() {
            return Ok("No commits yet. All files are new.".to_string());
        }

        // Get all files in working directory
        let mut working_files = std::collections::HashSet::new();
        self.collect_files(&self.root, &mut working_files)?;
        
        // For simplicity, show a basic status-like diff for now
        let index = self.read_index()?;
        
        for file_path in &working_files {
            if file_path.starts_with(".rune/") {
                continue; // Skip .rune directory
            }
            
            let relative_path = file_path.strip_prefix(&self.root)
                .unwrap_or(file_path.as_path())
                .to_string_lossy();
            
            if index.entries.contains_key(&relative_path.to_string()) {
                diff_output.push_str(&format!("M  {}\n", relative_path));
            } else {
                diff_output.push_str(&format!("??  {}\n", relative_path));
            }
        }
        
        if diff_output.is_empty() {
            Ok("No changes in working directory.".to_string())
        } else {
            Ok(format!("Changes in working directory:\n{}", diff_output))
        }
    }

    /// Show differences for a specific commit (compared to its parent)
    fn diff_commit(&self, commit_id: &str) -> Result<String> {
        let commits = self.log();
        let commit = commits.iter()
            .find(|c| c.id.starts_with(commit_id))
            .ok_or_else(|| anyhow::anyhow!("Commit '{}' not found", commit_id))?;
        
        let mut diff_output = format!("commit {}\n", commit.id);
        diff_output.push_str(&format!("Author: {} <{}>\n", commit.author.name, commit.author.email));
        diff_output.push_str(&format!("Date: {}\n\n", 
            chrono::DateTime::<chrono::Utc>::from_timestamp(commit.time, 0)
                .unwrap_or_default()
                .format("%Y-%m-%d %H:%M:%S UTC")));
        diff_output.push_str(&format!("    {}\n\n", commit.message));
        
        for file in &commit.files {
            diff_output.push_str(&format!("+++ {}\n", file));
        }
        
        Ok(diff_output)
    }

    /// Show differences between two commits
    fn diff_commits(&self, commit1: &str, commit2: &str) -> Result<String> {
        let commits = self.log();
        
        let c1 = commits.iter()
            .find(|c| c.id.starts_with(commit1))
            .ok_or_else(|| anyhow::anyhow!("Commit '{}' not found", commit1))?;
            
        let c2 = commits.iter()
            .find(|c| c.id.starts_with(commit2))
            .ok_or_else(|| anyhow::anyhow!("Commit '{}' not found", commit2))?;
        
        let mut diff_output = format!("diff {}..{}\n", c1.id, c2.id);
        
        // Simple implementation: show files that changed between commits
        let files1: std::collections::HashSet<_> = c1.files.iter().collect();
        let files2: std::collections::HashSet<_> = c2.files.iter().collect();
        
        // Files only in commit2 (added)
        for file in files2.difference(&files1) {
            diff_output.push_str(&format!("+++ {}\n", file));
        }
        
        // Files only in commit1 (removed)
        for file in files1.difference(&files2) {
            diff_output.push_str(&format!("--- {}\n", file));
        }
        
        // Files in both (potentially modified - simplified)
        for file in files1.intersection(&files2) {
            diff_output.push_str(&format!("    {}\n", file));
        }
        
        Ok(diff_output)
    }

    /// Helper method to collect all files in a directory
    fn collect_files(&self, dir: &std::path::Path, files: &mut std::collections::HashSet<std::path::PathBuf>) -> Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if !path.file_name().unwrap().to_string_lossy().starts_with('.') {
                        self.collect_files(&path, files)?;
                    }
                } else {
                    files.insert(path);
                }
            }
        }
        Ok(())
    }

    pub fn read_index(&self) -> Result<Index> {
        let p = self.rune_dir.join("index.json");
        if p.exists() {
            Ok(serde_json::from_str(&fs::read_to_string(p)?)?)
        } else {
            Ok(Index::default())
        }
    }
    pub fn write_index(&self, idx: &Index) -> Result<()> {
        fs::write(
            self.rune_dir.join("index.json"),
            serde_json::to_vec_pretty(idx)?,
        )?;
        Ok(())
    }

    pub fn stage_file(&self, rel: &str) -> Result<()> {
        let mut idx = self.read_index()?;
        let meta = fs::metadata(self.root.join(rel))?;
        let mtime = meta
            .modified()?
            .elapsed()
            .map(|e| -(e.as_secs() as i64))
            .unwrap_or(0);
        idx.entries.insert(rel.to_string(), mtime);
        self.write_index(&idx)
    }

    pub fn commit(&self, msg: &str, author: Author) -> Result<Commit> {
        let idx = self.read_index()?;
        if idx.entries.is_empty() {
            anyhow::bail!("nothing to commit");
        }
        let branch = self.head_ref();
        let branch_head = self.read_ref(&branch);
        let files = idx.entries.keys().cloned().collect::<Vec<_>>();
        let hash = blake3::hash(
            format!(
                "{}{}{:?}{}",
                msg,
                author.email,
                files,
                Utc::now().timestamp()
            )
            .as_bytes(),
        );
        let id = hex::encode(hash.as_bytes());
        let c = Commit {
            id: id.clone(),
            message: msg.to_string(),
            author,
            time: Utc::now().timestamp(),
            parent: branch_head,
            files,
            branch: branch.clone(),
        };
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.rune_dir.join("log.jsonl"))?;
        writeln!(f, "{}", serde_json::to_string(&c)?)?;
        self.write_ref(&branch, &id)?;
        self.write_index(&Index::default())?;
        
        // Update reflog entry
        self.update_reflog(&branch, &id, &format!("commit: {}", msg))?;
        
        Ok(c)
    }

    pub fn commit_amend(&self, msg: &str, edit_message: bool, author: Author) -> Result<Commit> {
        let idx = self.read_index()?;
        let mut log = self.log();
        
        if log.is_empty() {
            anyhow::bail!("no commits to amend");
        }
        
        // Check if merge is in progress
        if self.rune_dir.join("MERGE_HEAD").exists() {
            anyhow::bail!("cannot amend during merge");
        }
        
        let last_commit = &log[0];
        let branch = self.head_ref();
        
        // Use provided message if edit_message is true, otherwise keep original
        let commit_message = if edit_message {
            msg.to_string()
        } else {
            last_commit.message.clone()
        };
        
        // If index is empty, use files from last commit
        let files = if idx.entries.is_empty() {
            last_commit.files.clone()
        } else {
            idx.entries.keys().cloned().collect::<Vec<_>>()
        };
        
        // Create new commit hash
        let hash = blake3::hash(
            format!(
                "{}{}{:?}{}",
                commit_message,
                author.email,
                files,
                Utc::now().timestamp()
            )
            .as_bytes(),
        );
        let id = hex::encode(hash.as_bytes());
        
        // Create amended commit with same parent as original
        let amended_commit = Commit {
            id: id.clone(),
            message: commit_message.clone(),
            author,
            time: Utc::now().timestamp(),
            parent: last_commit.parent.clone(),
            files,
            branch: branch.clone(),
        };
        
        // Remove the last commit from log and add amended commit
        log.remove(0);
        log.insert(0, amended_commit.clone());
        
        // Rewrite the entire log file
        let log_path = self.rune_dir.join("log.jsonl");
        fs::remove_file(&log_path).ok(); // Remove old log
        
        let mut f = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&log_path)?;
        
        for commit in log.iter().rev() {
            writeln!(f, "{}", serde_json::to_string(commit)?)?;
        }
        
        // Update branch ref to point to amended commit
        self.write_ref(&branch, &id)?;
        
        // Clear index if it had changes
        if !idx.entries.is_empty() {
            self.write_index(&Index::default())?;
        }
        
        // Update reflog entry
        self.update_reflog(&branch, &id, &format!("commit (amend): {}", commit_message))?;
        
        Ok(amended_commit)
    }

    fn update_reflog(&self, ref_name: &str, commit_id: &str, message: &str) -> Result<()> {
        let reflog_dir = self.rune_dir.join("logs");
        fs::create_dir_all(&reflog_dir)?;
        
        let reflog_path = reflog_dir.join(ref_name.replace("/", "_"));
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(reflog_path)?;
        
        writeln!(f, "{} {} {}", 
            Utc::now().timestamp(), 
            commit_id, 
            message
        )?;
        
        Ok(())
    }

    pub fn revert_commit(&self, commit_id: &str, mainline: Option<usize>, no_commit: bool, author: Author) -> Result<Commit> {
        let log = self.log();
        
        // Find the commit to revert
        let target_commit = log.iter()
            .find(|c| c.id == commit_id || c.id.starts_with(commit_id))
            .ok_or_else(|| anyhow::anyhow!("commit '{}' not found", commit_id))?;
        
        // Check if it's a merge commit and handle mainline
        if target_commit.parent.is_some() && mainline.is_some() {
            // TODO: Handle merge commits with multiple parents
            // For now, we'll treat it as a regular commit
        }
        
        // Get the parent commit to see what was there before
        let parent_files = if let Some(ref parent_id) = target_commit.parent {
            log.iter()
                .find(|c| c.id == *parent_id)
                .map(|c| c.files.clone())
                .unwrap_or_default()
        } else {
            Vec::new() // If no parent, this was the initial commit
        };
        
        // Create inverse changes:
        // 1. Files that were added in target_commit should be removed
        // 2. Files that were removed in target_commit should be restored
        // 3. Files that were modified should be reverted to parent state
        
        let mut revert_files = Vec::new();
        let mut staged_files = std::collections::BTreeMap::new();
        
        // Files in target commit that weren't in parent = added files (should be removed)
        for file in &target_commit.files {
            if !parent_files.contains(file) {
                // This file was added, so we remove it in revert
                let file_path = self.root.join(file);
                if file_path.exists() {
                    fs::remove_file(&file_path).ok();
                }
            } else {
                // This file was modified, we need to restore parent version
                // For now, we'll just mark it as needing attention
                revert_files.push(file.clone());
                // Stage the current state for the revert commit
                let metadata = fs::metadata(self.root.join(file))?;
                staged_files.insert(file.clone(), metadata.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64);
            }
        }
        
        // Files in parent that aren't in target = removed files (should be restored)
        for file in &parent_files {
            if !target_commit.files.contains(file) {
                // This file was removed, we need to restore it
                // For now, create a placeholder
                let file_path = self.root.join(file);
                if let Some(parent_dir) = file_path.parent() {
                    fs::create_dir_all(parent_dir)?;
                }
                fs::write(&file_path, format!("# Restored file: {}\n", file))?;
                revert_files.push(file.clone());
                let metadata = fs::metadata(&file_path)?;
                staged_files.insert(file.clone(), metadata.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64);
            }
        }
        
        if no_commit {
            // Just apply changes to working directory and index
            let index = Index { entries: staged_files };
            self.write_index(&index)?;
            return Ok(Commit {
                id: "no-commit".to_string(),
                message: format!("Revert \"{}\"", target_commit.message),
                author,
                time: Utc::now().timestamp(),
                parent: None,
                files: revert_files,
                branch: self.head_ref(),
            });
        }
        
        // Create revert commit
        let revert_message = format!("Revert \"{}\"", target_commit.message);
        let branch = self.head_ref();
        let branch_head = self.read_ref(&branch);
        
        let hash = blake3::hash(
            format!(
                "{}{}{:?}{}",
                revert_message,
                author.email,
                revert_files,
                Utc::now().timestamp()
            )
            .as_bytes(),
        );
        let id = hex::encode(hash.as_bytes());
        
        let revert_commit = Commit {
            id: id.clone(),
            message: revert_message.clone(),
            author,
            time: Utc::now().timestamp(),
            parent: branch_head,
            files: revert_files,
            branch: branch.clone(),
        };
        
        // Add to log
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.rune_dir.join("log.jsonl"))?;
        writeln!(f, "{}", serde_json::to_string(&revert_commit)?)?;
        
        // Update branch ref
        self.write_ref(&branch, &id)?;
        
        // Clear index
        self.write_index(&Index::default())?;
        
        // Update reflog
        self.update_reflog(&branch, &id, &format!("revert: {}", target_commit.message))?;
        
        Ok(revert_commit)
    }

    pub fn log(&self) -> Vec<Commit> {
        let p = self.rune_dir.join("log.jsonl");
        if !p.exists() {
            return vec![];
        }
        fs::read_to_string(p)
            .unwrap_or_default()
            .lines()
            .filter_map(|l| serde_json::from_str::<Commit>(l).ok())
            .collect()
    }

    /// Reset staging area and optionally working directory
    pub fn reset(&self, files: &[std::path::PathBuf], hard: bool) -> Result<()> {
        if files.is_empty() {
            // Reset entire staging area
            self.reset_staging_area()?;
            
            if hard {
                self.reset_working_directory()?;
            }
        } else {
            // Reset specific files
            for file in files {
                self.reset_file(file, hard)?;
            }
        }
        
        Ok(())
    }

    /// Reset the entire staging area
    fn reset_staging_area(&self) -> Result<()> {
        self.write_index(&Index::default())?;
        Ok(())
    }

    /// Reset working directory to match HEAD (destructive)
    fn reset_working_directory(&self) -> Result<()> {
        let head_ref = self.head_ref();
        let head_commit_id = self.read_ref(&head_ref)
            .ok_or_else(|| anyhow::anyhow!("No commits found - cannot reset working directory"))?;
        
        let commit = self.get_commit(&head_commit_id)?;
        
        // For our simplified implementation, just recreate the files from commit
        // In a real VCS, we would restore the exact blob contents
        for file_path in &commit.files {
            let file_full_path = self.root.join(file_path);
            
            // If the file doesn't exist, create a placeholder (this is simplified)
            if !file_full_path.exists() {
                if let Some(parent) = file_full_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                // Create file with basic content (simplified for demo)
                fs::write(file_full_path, format!("Content for {} (restored from commit {})", file_path, &head_commit_id[..8]))?;
            }
        }
        
        Ok(())
    }

    /// Reset a specific file from staging and optionally working directory
    fn reset_file(&self, file_path: &std::path::Path, hard: bool) -> Result<()> {
        let rel_path = file_path.strip_prefix(&self.root)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();
        
        // Remove from staging area
        let mut index = self.read_index()?;
        index.entries.remove(&rel_path);
        self.write_index(&index)?;
        
        if hard {
            // Reset file in working directory to HEAD version
            let head_ref = self.head_ref();
            if let Some(head_commit_id) = self.read_ref(&head_ref) {
                self.restore_file_from_commit_str(&rel_path, &head_commit_id)?;
            } else {
                // No commits yet, just remove the file
                let full_path = self.root.join(&rel_path);
                if full_path.exists() {
                    fs::remove_file(full_path)?;
                }
            }
        }
        
        Ok(())
    }

    /// Clean working directory (remove all files except .rune)
    fn clean_working_directory(&self) -> Result<()> {
        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.file_name() == Some(std::ffi::OsStr::new(".rune")) {
                continue; // Skip .rune directory
            }
            
            if path.is_file() {
                fs::remove_file(path)?;
            } else if path.is_dir() {
                fs::remove_dir_all(path)?;
            }
        }
        
        Ok(())
    }

    /// Restore a file from a specific commit
    pub fn restore_file_from_commit(&self, commit_id: &str, file_path: &std::path::Path) -> Result<()> {
        let file_path_str = file_path.to_string_lossy();
        self.restore_file_from_commit_str(&file_path_str, commit_id)
    }

    /// Restore a file from a specific commit (internal implementation)
    fn restore_file_from_commit_str(&self, file_path: &str, commit_id: &str) -> Result<()> {
        let commit = self.get_commit(commit_id)?;
        
        if commit.files.contains(&file_path.to_string()) {
            // Read the blob content from the objects directory
            let blob_path = self.rune_dir.join("objects").join(format!("{}.blob", file_path.replace("/", "_")));
            if blob_path.exists() {
                let content = fs::read(blob_path)?;
                let dest_path = self.root.join(file_path);
                
                // Create parent directories if they don't exist
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                
                fs::write(dest_path, content)?;
            }
        }
        
        Ok(())
    }

    /// Get a commit by ID (helper method)
    fn get_commit(&self, commit_id: &str) -> Result<Commit> {
        let log = self.log();
        log.into_iter()
            .find(|c| c.id == commit_id || c.id.starts_with(commit_id))
            .ok_or_else(|| anyhow::anyhow!("Commit '{}' not found", commit_id))
    }

    pub fn create(&self) -> Result<()> {
        // Create directories (this is safe even if they exist)
        fs::create_dir_all(self.rune_dir.join("objects"))?;
        fs::create_dir_all(self.rune_dir.join("refs/heads"))?;
        
        // Only create main branch if it doesn't exist
        let main_ref = self.rune_dir.join("refs/heads/main");
        if !main_ref.exists() {
            fs::write(main_ref, b"")?;
        }
        
        // Only set HEAD if it doesn't exist
        let head_file = self.rune_dir.join("HEAD");
        if !head_file.exists() {
            self.set_head("refs/heads/main")?;
        }
        
        // Only create index if it doesn't exist
        let index_file = self.rune_dir.join("index.json");
        if !index_file.exists() {
            self.write_index(&Index::default())?;
        }
        
        Ok(())
    }

    /// Detect merge conflicts between two commits
    fn detect_merge_conflicts(&self, _current_commit: &str, _merge_commit: &str) -> Result<Vec<String>> {
        // Simplified implementation - in a real system, this would compare file trees
        // For now, we'll simulate some conflicts for demonstration
        Ok(vec![]) // No conflicts for now
    }

    /// Save merge state for abort/continue operations
    fn save_merge_state(&self, branch_name: &str, current_commit: &str, merge_commit: &str, strategy: Option<&str>) -> Result<()> {
        #[derive(Serialize)]
        struct MergeState {
            branch_name: String,
            current_commit: String,
            merge_commit: String,
            strategy: Option<String>,
        }

        let merge_state = MergeState {
            branch_name: branch_name.to_string(),
            current_commit: current_commit.to_string(),
            merge_commit: merge_commit.to_string(),
            strategy: strategy.map(|s| s.to_string()),
        };

        let merge_file = self.rune_dir.join("MERGE_STATE");
        let json = serde_json::to_string_pretty(&merge_state)?;
        fs::write(merge_file, json)?;
        Ok(())
    }

    /// Apply merge conflicts to working directory
    fn apply_merge_conflicts(&self, conflicts: &[String]) -> Result<()> {
        // In a real implementation, this would write conflict markers to files
        for file in conflicts {
            let file_path = self.root.join(file);
            let conflict_content = format!(
                "<<<<<<< HEAD\n(current branch content)\n=======\n(merge branch content)\n>>>>>>> branch\n"
            );
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(file_path, conflict_content)?;
        }
        Ok(())
    }

    /// Abort an in-progress merge
    pub fn abort_merge(&self) -> Result<()> {
        let merge_file = self.rune_dir.join("MERGE_STATE");
        if !merge_file.exists() {
            return Err(anyhow::anyhow!("No merge in progress"));
        }

        // Remove merge state file
        fs::remove_file(merge_file)?;

        // Reset working directory to current branch state
        self.clean_working_directory()?;

        Ok(())
    }

    /// Continue a merge after resolving conflicts
    pub fn continue_merge(&self) -> Result<()> {
        let merge_file = self.rune_dir.join("MERGE_STATE");
        if !merge_file.exists() {
            return Err(anyhow::anyhow!("No merge in progress"));
        }

        #[derive(Deserialize)]
        struct MergeState {
            branch_name: String,
            current_commit: String,
            merge_commit: String,
            strategy: Option<String>,
        }

        // Read merge state
        let json = fs::read_to_string(&merge_file)?;
        let merge_state: MergeState = serde_json::from_str(&json)?;

        // Check if all conflicts are resolved (no files with conflict markers)
        if self.has_unresolved_conflicts()? {
            return Err(anyhow::anyhow!("Please resolve all conflicts before continuing"));
        }

        // Create merge commit
        let current_branch = self.current_branch()
            .ok_or_else(|| anyhow::anyhow!("Not on a branch"))?;
        
        let mut message = format!("Merge branch '{}' into {}", merge_state.branch_name, current_branch);
        if let Some(strategy) = merge_state.strategy {
            message.push_str(&format!(" (strategy: {})", strategy));
        }

        let merge_commit = self.create_merge_commit(&merge_state.current_commit, &merge_state.merge_commit, &message)?;
        self.write_ref(&format!("refs/heads/{}", current_branch), &merge_commit)?;

        // Remove merge state file
        fs::remove_file(merge_file)?;

        Ok(())
    }

    /// Check if there are unresolved conflicts in working directory
    fn has_unresolved_conflicts(&self) -> Result<bool> {
        // Simplified: check if any tracked files contain conflict markers
        let entries = fs::read_dir(&self.root)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if content.contains("<<<<<<<") || content.contains(">>>>>>>") {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    /// Abort an in-progress rebase
    pub fn abort_rebase(&self) -> Result<()> {
        let rebase_file = self.rune_dir.join("REBASE_STATE");
        if !rebase_file.exists() {
            return Err(anyhow::anyhow!("No rebase in progress"));
        }

        // Remove rebase state file
        fs::remove_file(rebase_file)?;

        // Reset working directory to original state
        self.clean_working_directory()?;

        Ok(())
    }

    /// Continue a rebase after resolving conflicts
    pub fn continue_rebase(&self) -> Result<()> {
        let rebase_file = self.rune_dir.join("REBASE_STATE");
        if !rebase_file.exists() {
            return Err(anyhow::anyhow!("No rebase in progress"));
        }

        #[derive(Deserialize, Serialize)]
        struct RebaseState {
            target_commit: String,
            current_commit: String,
            remaining_commits: Vec<String>,
        }

        // Read rebase state
        let json = fs::read_to_string(&rebase_file)?;
        let mut rebase_state: RebaseState = serde_json::from_str(&json)?;

        // Check if all conflicts are resolved
        if self.has_unresolved_conflicts()? {
            return Err(anyhow::anyhow!("Please resolve all conflicts before continuing"));
        }

        // Apply current commit
        if !rebase_state.current_commit.is_empty() {
            // Create a new commit with resolved changes
            let current_branch = self.current_branch()
                .ok_or_else(|| anyhow::anyhow!("Not on a branch"))?;
            
            // For now, just update the branch ref (simplified)
            self.write_ref(&format!("refs/heads/{}", current_branch), &rebase_state.current_commit)?;
        }

        // Continue with remaining commits or finish rebase
        if rebase_state.remaining_commits.is_empty() {
            // Rebase complete
            fs::remove_file(rebase_file)?;
        } else {
            // Update rebase state with next commit
            rebase_state.current_commit = rebase_state.remaining_commits.remove(0);
            let json = serde_json::to_string_pretty(&rebase_state)?;
            fs::write(rebase_file, json)?;
        }

        Ok(())
    }

    /// Skip current commit during rebase
    pub fn skip_rebase_commit(&self) -> Result<()> {
        let rebase_file = self.rune_dir.join("REBASE_STATE");
        if !rebase_file.exists() {
            return Err(anyhow::anyhow!("No rebase in progress"));
        }

        #[derive(Deserialize, Serialize)]
        struct RebaseState {
            target_commit: String,
            current_commit: String,
            remaining_commits: Vec<String>,
        }

        // Read rebase state
        let json = fs::read_to_string(&rebase_file)?;
        let mut rebase_state: RebaseState = serde_json::from_str(&json)?;

        // Skip current commit and move to next
        if rebase_state.remaining_commits.is_empty() {
            // No more commits, finish rebase
            fs::remove_file(rebase_file)?;
        } else {
            // Move to next commit
            rebase_state.current_commit = rebase_state.remaining_commits.remove(0);
            let json = serde_json::to_string_pretty(&rebase_state)?;
            fs::write(rebase_file, json)?;
        }

        Ok(())
    }

    /// Show content of a file at a specific commit
    pub fn show_file_at_commit(&self, commit_id: &str, file_path: &str) -> Result<String> {
        // Find the commit
        let commits = self.log();
        let commit = commits.iter()
            .find(|c| c.id == commit_id || c.id.starts_with(commit_id))
            .ok_or_else(|| anyhow::anyhow!("Commit '{}' not found", commit_id))?;

        // Check if file exists in this commit
        if !commit.files.contains(&file_path.to_string()) {
            return Err(anyhow::anyhow!("File '{}' not found in commit {}", file_path, commit_id));
        }

        // For now, we'll try to read from the current working directory
        // In a real implementation, this would read from the commit's file tree
        let file_full_path = self.root.join(file_path);
        
        if file_full_path.exists() {
            Ok(fs::read_to_string(file_full_path)?)
        } else {
            // File doesn't exist in working directory, return placeholder
            Ok(format!("(File '{}' content at commit {})\n[Content not available - file may have been deleted or moved]", file_path, commit_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_initialized_store() -> (TempDir, Store) {
        let temp_dir = TempDir::new().unwrap();
        let store = Store::open(temp_dir.path()).unwrap();
        store.create().unwrap();
        (temp_dir, store)
    }

    #[test]
    fn test_store_open() {
        let temp_dir = TempDir::new().unwrap();
        let store = Store::open(temp_dir.path()).unwrap();
        
        assert_eq!(store.root, temp_dir.path());
        assert_eq!(store.rune_dir, temp_dir.path().join(".rune"));
    }

    #[test]
    fn test_store_discover() {
        let (_temp_dir, store) = create_initialized_store();
        
        // Create subdirectory and test discovery
        let subdir = store.root.join("subdir");
        fs::create_dir_all(&subdir).unwrap();
        
        let discovered = Store::discover(&subdir).unwrap();
        assert_eq!(discovered.root, store.root);
    }

    #[test]
    fn test_store_discover_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let result = Store::discover(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_store_create() {
        let temp_dir = TempDir::new().unwrap();
        let store = Store::open(temp_dir.path()).unwrap();
        
        store.create().unwrap();
        
        // Verify directory structure
        assert!(store.rune_dir.join("objects").exists());
        assert!(store.rune_dir.join("refs/heads").exists());
        assert!(store.rune_dir.join("HEAD").exists());
        assert!(store.rune_dir.join("index.json").exists());
        assert!(store.rune_dir.join("refs/heads/main").exists());
    }

    #[test]
    fn test_config_operations() {
        let (_temp_dir, store) = create_initialized_store();
        
        // Test default config
        let config = store.config();
        assert_eq!(config.core.default_branch, "main");
        assert_eq!(config.lfs.chunk_size, 8 * 1024 * 1024);
        
        // Test writing and reading config
        let new_config = RuneConfig {
            core: CoreCfg {
                default_branch: "develop".to_string(),
            },
            lfs: LfsCfg {
                chunk_size: 1024,
                remote: None,
                track: vec![],
            },
        };
        
        store.write_config(&new_config).unwrap();
        let read_config = store.config();
        
        assert_eq!(read_config.core.default_branch, "develop");
        assert_eq!(read_config.lfs.chunk_size, 1024);
    }

    #[test]
    fn test_head_ref_operations() {
        let (_temp_dir, store) = create_initialized_store();
        
        // Test default head ref
        let head_ref = store.head_ref();
        assert_eq!(head_ref, "refs/heads/main");
        
        // Test setting new head ref
        store.set_head("refs/heads/feature").unwrap();
        let new_head_ref = store.head_ref();
        assert_eq!(new_head_ref, "refs/heads/feature");
    }

    #[test]
    fn test_ref_operations() {
        let (_temp_dir, store) = create_initialized_store();
        
        let ref_name = "refs/heads/test";
        let commit_id = "abc123def456";
        
        // Test writing and reading ref
        store.write_ref(ref_name, commit_id).unwrap();
        let read_id = store.read_ref(ref_name).unwrap();
        
        assert_eq!(read_id, commit_id);
        
        // Test reading non-existent ref
        let non_existent = store.read_ref("refs/heads/nonexistent");
        assert!(non_existent.is_none());
    }

    #[test]
    fn test_index_operations() {
        let (_temp_dir, store) = create_initialized_store();
        
        // Test default empty index
        let index = store.read_index().unwrap();
        assert!(index.entries.is_empty());
        
        // Test writing and reading index
        let mut new_index = Index::default();
        new_index.entries.insert("file1.txt".to_string(), 1234567890);
        new_index.entries.insert("file2.txt".to_string(), 1234567891);
        
        store.write_index(&new_index).unwrap();
        let read_index = store.read_index().unwrap();
        
        assert_eq!(read_index.entries.len(), 2);
        assert_eq!(read_index.entries.get("file1.txt"), Some(&1234567890));
        assert_eq!(read_index.entries.get("file2.txt"), Some(&1234567891));
    }

    #[test]
    fn test_stage_file() {
        let (_temp_dir, store) = create_initialized_store();
        
        // Create a test file
        let test_file = "test.txt";
        let test_content = "Hello, World!";
        fs::write(store.root.join(test_file), test_content).unwrap();
        
        // Stage the file
        store.stage_file(test_file).unwrap();
        
        // Verify file was staged
        let index = store.read_index().unwrap();
        assert!(index.entries.contains_key(test_file));
    }

    #[test]
    fn test_stage_nonexistent_file() {
        let (_temp_dir, store) = create_initialized_store();
        
        let result = store.stage_file("nonexistent.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_commit() {
        let (_temp_dir, store) = create_initialized_store();
        
        // Create and stage a test file
        let test_file = "test.txt";
        let test_content = "Hello, World!";
        fs::write(store.root.join(test_file), test_content).unwrap();
        store.stage_file(test_file).unwrap();
        
        // Create commit
        let author = Author {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        let commit = store.commit("Initial commit", author.clone()).unwrap();
        
        assert_eq!(commit.message, "Initial commit");
        assert_eq!(commit.author.name, "Test User");
        assert_eq!(commit.author.email, "test@example.com");
        assert_eq!(commit.files, vec![test_file.to_string()]);
        assert!(commit.parent.is_none()); // First commit has no parent
        
        // Verify commit was logged
        let log = store.log();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].id, commit.id);
    }

    #[test]
    fn test_commit_nothing_staged() {
        let (_temp_dir, store) = create_initialized_store();
        
        let author = Author {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        let result = store.commit("Empty commit", author);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("nothing to commit"));
    }

    #[test]
    fn test_multiple_commits() {
        let (_temp_dir, store) = create_initialized_store();
        
        let author = Author {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        // First commit
        fs::write(store.root.join("file1.txt"), "Content 1").unwrap();
        store.stage_file("file1.txt").unwrap();
        let commit1 = store.commit("First commit", author.clone()).unwrap();
        
        // Second commit
        fs::write(store.root.join("file2.txt"), "Content 2").unwrap();
        store.stage_file("file2.txt").unwrap();
        let commit2 = store.commit("Second commit", author).unwrap();
        
        // Verify commit history
        let log = store.log();
        assert_eq!(log.len(), 2);
        
        // Find commits in log (order may vary)
        let commit1_in_log = log.iter().find(|c| c.id == commit1.id).unwrap();
        let commit2_in_log = log.iter().find(|c| c.id == commit2.id).unwrap();
        
        assert_eq!(commit2_in_log.parent, Some(commit1.id.clone()));
        assert!(commit1_in_log.parent.is_none());
    }

    #[test]
    fn test_empty_log() {
        let (_temp_dir, store) = create_initialized_store();
        
        let log = store.log();
        assert!(log.is_empty());
    }

    #[test]
    fn test_track_config() {
        let track_cfg = TrackCfg {
            pattern: "*.large".to_string(),
        };
        
        assert_eq!(track_cfg.pattern, "*.large");
    }

    #[test]
    fn test_index_ordering() {
        let mut index = Index::default();
        index.entries.insert("z_file.txt".to_string(), 1);
        index.entries.insert("a_file.txt".to_string(), 2);
        index.entries.insert("m_file.txt".to_string(), 3);
        
        // BTreeMap should maintain ordering
        let keys: Vec<_> = index.entries.keys().collect();
        assert_eq!(keys, vec!["a_file.txt", "m_file.txt", "z_file.txt"]);
    }

    #[test]
    fn test_core_config_defaults() {
        let core_cfg = CoreCfg::default();
        assert_eq!(core_cfg.default_branch, "main");
    }

    #[test]
    fn test_lfs_config_defaults() {
        let lfs_cfg = LfsCfg::default();
        assert_eq!(lfs_cfg.chunk_size, 8 * 1024 * 1024);
        assert!(lfs_cfg.remote.is_none());
        assert!(lfs_cfg.track.is_empty());
    }
}
