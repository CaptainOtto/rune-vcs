use std::path::Path;

fn main() {
    let store_path = Path::new("/tmp/test-rune/.rune");
    
    println!("Store path exists: {}", store_path.exists());
    println!("Store path: {:?}", store_path);
    
    let heads_dir = store_path.join("refs/heads");
    println!("Heads dir exists: {}", heads_dir.exists());
    println!("Heads dir: {:?}", heads_dir);
    
    let new_branch_path = heads_dir.join("feature/test");
    println!("New branch path: {:?}", new_branch_path);
    println!("New branch parent: {:?}", new_branch_path.parent());
    
    if let Some(parent) = new_branch_path.parent() {
        println!("Parent exists: {}", parent.exists());
        match std::fs::create_dir_all(parent) {
            Ok(()) => println!("create_dir_all succeeded"),
            Err(e) => println!("create_dir_all failed: {}", e),
        }
    }
    
    match std::fs::write(&new_branch_path, "test-commit-id") {
        Ok(()) => println!("write succeeded"),
        Err(e) => println!("write failed: {}", e),
    }
}
