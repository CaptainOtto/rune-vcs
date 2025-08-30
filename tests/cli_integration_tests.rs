use std::process::Command;
use std::fs;
use tempfile::TempDir;
use std::path::Path;

fn get_rune_binary() -> String {
    // Get the workspace root (two levels up from the crates/rune-cli directory)
    let manifest_dir = env!("CARGO_MANIFEST_DIR"); // This will be crates/rune-cli
    let workspace_dir = std::path::Path::new(manifest_dir)
        .parent() // crates/
        .unwrap()
        .parent() // workspace root
        .unwrap();
    
    let binary_path = workspace_dir.join("target/debug/rune");
    let binary_path_str = binary_path.to_string_lossy().to_string();
    
    // Build the binary if it doesn't exist
    if !binary_path.exists() {
        let output = Command::new("cargo")
            .args(&["build", "--bin", "rune"])
            .current_dir(workspace_dir)
            .output()
            .expect("Failed to build rune binary");
        
        if !output.status.success() {
            panic!("Failed to build rune: {}", String::from_utf8_lossy(&output.stderr));
        }
    }
    
    binary_path_str
}

fn run_rune_command(args: &[&str], working_dir: &Path) -> std::process::Output {
    let rune_binary = get_rune_binary();
    Command::new(rune_binary)
        .args(args)
        .current_dir(working_dir)
        .output()
        .expect("Failed to execute rune command")
}

#[test]
fn test_init_and_status_commands() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Test init command
    let output = run_rune_command(&["init"], temp_dir.path());
    assert!(output.status.success(), "Init command should succeed. stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    // Verify .rune directory was created
    assert!(temp_dir.path().join(".rune").exists());
    
    // Test status command  
    let output = run_rune_command(&["status"], temp_dir.path());
    assert!(output.status.success(), "Status command should succeed. stderr: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_add_and_commit_workflow() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let repo_path = temp_dir.path();
    
    // Initialize repository
    let output = run_rune_command(&["init"], repo_path);
    assert!(output.status.success(), "Init failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Verify .rune directory was created
    assert!(repo_path.join(".rune").exists());
    
    // Create a test file
    fs::write(repo_path.join("test.txt"), "Hello, Rune!").unwrap();
    
    // Add the file
    let output = run_rune_command(&["add", "test.txt"], repo_path);
    assert!(output.status.success(), "Add command should succeed. stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    // Commit the file
    let output = run_rune_command(&["commit", "-m", "Initial commit"], repo_path);
    assert!(output.status.success(), "Commit command should succeed. stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    // Check log
    let log_output = run_rune_command(&["log"], repo_path);
    assert!(log_output.status.success(), "Log command should succeed. stderr: {}", String::from_utf8_lossy(&log_output.stderr));
    let log_str = String::from_utf8_lossy(&log_output.stdout);
    assert!(log_str.contains("Initial commit"), "Log should contain commit message");
}

#[test]
fn test_error_scenarios() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Test status command in non-repository
    let output = run_rune_command(&["status"], temp_dir.path());
    assert!(!output.status.success(), "Status should fail in non-repository");
    
    // Initialize for remaining tests
    let repo_path = temp_dir.path();
    let output = run_rune_command(&["init"], repo_path);
    assert!(output.status.success(), "Init should succeed");
    
    // Test adding non-existent file
    let output = run_rune_command(&["add", "nonexistent.txt"], repo_path);
    assert!(!output.status.success(), "Add should fail for non-existent file");
    
    // Test commit with nothing staged
    let output = run_rune_command(&["commit", "-m", "Empty commit"], repo_path);
    assert!(!output.status.success(), "Commit should fail with nothing staged");
}

#[test]
fn test_help_and_version() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Test help command
    let output = run_rune_command(&["--help"], temp_dir.path());
    assert!(output.status.success(), "Help command should succeed. stderr: {}", String::from_utf8_lossy(&output.stderr));
    let help_text = String::from_utf8_lossy(&output.stdout);
    assert!(help_text.contains("Rune"), "Help should contain 'Rune'");
    
    // Test version command
    let output = run_rune_command(&["--version"], temp_dir.path());
    assert!(output.status.success(), "Version command should succeed. stderr: {}", String::from_utf8_lossy(&output.stderr));
    let version_text = String::from_utf8_lossy(&output.stdout);
    assert!(version_text.contains("0.3.0-alpha.4"), "Version should contain version number");
}
