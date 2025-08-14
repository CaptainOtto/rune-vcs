use std::fs;
use tempfile::TempDir;
use rune_store::Store;
use rune_core::{Author, Commit};

/// Integration test for a complete VCS workflow
#[test]
fn test_complete_vcs_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let store = Store::open(temp_dir.path()).unwrap();
    store.create().unwrap();
    
    let author = Author {
        name: "Integration Test".to_string(),
        email: "test@integration.com".to_string(),
    };
    
    // 1. Initial commit
    fs::write(store.root.join("README.md"), "# My Project").unwrap();
    store.stage_file("README.md").unwrap();
    let _commit1 = store.commit("Initial commit", author.clone()).unwrap();
    
    // 2. Add more files
    fs::create_dir_all(store.root.join("src")).unwrap();
    fs::write(store.root.join("src/main.rs"), "fn main() {}").unwrap();
    store.stage_file("src/main.rs").unwrap();
    let _commit2 = store.commit("Add initial project files", author.clone()).unwrap();
    
    // 3. Modify README and commit
    fs::write(store.root.join("README.md"), "# My Project\n\nUpdated content with more details.").unwrap();
    store.stage_file("README.md").unwrap();
    let _commit3 = store.commit("Update README", author.clone()).unwrap();
    
    // Verify commit history
    let log = store.log();
    assert_eq!(log.len(), 3);
    
    // Verify commit relationships (order is chronological - oldest first)
    let commits: Vec<&Commit> = log.iter().collect();
    // The log is in chronological order (oldest first)
    assert_eq!(commits[0].message, "Initial commit"); // Oldest first
    assert_eq!(commits[1].message, "Add initial project files");
    assert_eq!(commits[2].message, "Update README"); // Most recent last
    
    // Verify parent relationships structure
    assert!(commits[0].parent.is_none()); // First commit has no parent
    assert!(commits[1].parent.is_some()); // Second commit has a parent
    assert!(commits[2].parent.is_some()); // Latest commit has a parent
    
    // Verify files in commits
    assert_eq!(commits[0].files, vec!["README.md"]); // First commit
    assert_eq!(commits[1].files, vec!["src/main.rs"]); // Second commit
    assert_eq!(commits[2].files, vec!["README.md"]); // Third commit
}

#[test]
fn test_multi_file_staging_and_commit() {
    let temp_dir = TempDir::new().unwrap();
    let store = Store::open(temp_dir.path()).unwrap();
    store.create().unwrap();
    
    let author = Author {
        name: "Multi File Test".to_string(),
        email: "multi@test.com".to_string(),
    };
    
    // Create multiple files
    fs::write(store.root.join("file1.txt"), "Content 1").unwrap();
    fs::write(store.root.join("file2.txt"), "Content 2").unwrap();
    fs::write(store.root.join("file3.txt"), "Content 3").unwrap();
    
    // Stage them all
    store.stage_file("file1.txt").unwrap();
    store.stage_file("file2.txt").unwrap();
    store.stage_file("file3.txt").unwrap();
    
    // Commit all together
    let commit = store.commit("Add multiple files", author).unwrap();
    
    // Verify commit contains all files
    let mut expected_files = vec!["file1.txt", "file2.txt", "file3.txt"];
    expected_files.sort();
    let mut actual_files = commit.files.clone();
    actual_files.sort();
    assert_eq!(actual_files, expected_files);
    
    // Verify log
    let log = store.log();
    assert_eq!(log.len(), 1);
    assert_eq!(log[0].message, "Add multiple files");
}

#[test]
fn test_repository_discovery() {
    let temp_dir = TempDir::new().unwrap();
    
    // Before initialization, discover should fail
    assert!(Store::discover(temp_dir.path()).is_err());
    
    // After initialization, discover should work
    let store = Store::open(temp_dir.path()).unwrap();
    store.create().unwrap();
    
    // Discovery should work from root
    assert!(Store::discover(temp_dir.path()).is_ok());
    
    // Discovery should work from subdirectory
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir_all(&subdir).unwrap();
    assert!(Store::discover(&subdir).is_ok());
    
    // Discovery should find the same repository
    let discovered = Store::discover(&subdir).unwrap();
    assert_eq!(discovered.root, store.root);
}

#[test]
fn test_error_handling_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let store = Store::open(temp_dir.path()).unwrap();
    store.create().unwrap();
    
    let author = Author {
        name: "Error Test".to_string(),
        email: "error@test.com".to_string(),
    };
    
    // Try to stage non-existent file
    assert!(store.stage_file("nonexistent.txt").is_err());
    
    // Try to commit with nothing staged
    assert!(store.commit("Empty commit", author.clone()).is_err());
    
    // Stage a file and verify commit works
    fs::write(store.root.join("test.txt"), "Test content").unwrap();
    store.stage_file("test.txt").unwrap();
    assert!(store.commit("Test commit", author).is_ok());
    
    // Verify log
    let log = store.log();
    assert_eq!(log.len(), 1);
}
