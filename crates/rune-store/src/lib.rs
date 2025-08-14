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

    /// Merge a branch into the current branch
    pub fn merge_branch(&self, branch_name: &str, no_ff: bool) -> Result<()> {
        let current_branch = self.current_branch()
            .ok_or_else(|| anyhow::anyhow!("Not on a branch"))?;
        
        let current_commit_id = self.read_ref(&format!("refs/heads/{}", current_branch))
            .ok_or_else(|| anyhow::anyhow!("Current branch has no commits"))?;
        
        let merge_commit_id = self.read_ref(&format!("refs/heads/{}", branch_name))
            .ok_or_else(|| anyhow::anyhow!("Branch '{}' has no commits", branch_name))?;
        
        // Check if this is a fast-forward merge (merge commit is ahead of current)
        let is_fast_forward = self.is_ancestor(&current_commit_id, &merge_commit_id)?;
        
        if is_fast_forward && !no_ff {
            // Fast-forward merge: just update the current branch to point to the merge commit
            self.write_ref(&format!("refs/heads/{}", current_branch), &merge_commit_id)?;
        } else {
            // Create a merge commit
            let merge_commit = self.create_merge_commit(&current_commit_id, &merge_commit_id, 
                &format!("Merge branch '{}' into {}", branch_name, current_branch))?;
            self.write_ref(&format!("refs/heads/{}", current_branch), &merge_commit)?;
        }
        
        Ok(())
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
        Ok(c)
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
