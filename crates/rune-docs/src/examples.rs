use crate::{Example};
use std::collections::HashMap;

/// Load all code examples organized by category
pub fn load_all_examples() -> HashMap<String, Vec<Example>> {
    let mut examples = HashMap::new();
    
    // Basic operations
    examples.insert("basic".to_string(), vec![
        Example {
            title: "Initialize a new repository".to_string(),
            description: "Create a new Rune repository in the current directory".to_string(),
            commands: vec![
                "rune init".to_string(),
            ],
            expected_output: Some("✓ Initialized empty Rune repository in .rune/".to_string()),
            category: "basic".to_string(),
        },
        Example {
            title: "Add files and create first commit".to_string(),
            description: "Stage files and create your first commit".to_string(),
            commands: vec![
                "echo '# My Project' > README.md".to_string(),
                "rune add README.md".to_string(),
                "rune commit -m 'Initial commit'".to_string(),
            ],
            expected_output: Some("✓ Committed: Initial commit".to_string()),
            category: "basic".to_string(),
        },
        Example {
            title: "Check repository status".to_string(),
            description: "See what files are modified, staged, or untracked".to_string(),
            commands: vec![
                "rune status".to_string(),
            ],
            expected_output: Some("On branch main\nNothing to commit, working tree clean".to_string()),
            category: "basic".to_string(),
        },
        Example {
            title: "View commit history".to_string(),
            description: "See the history of commits in your repository".to_string(),
            commands: vec![
                "rune log".to_string(),
                "rune log --oneline".to_string(),
            ],
            expected_output: None,
            category: "basic".to_string(),
        },
    ]);
    
    // Branching examples
    examples.insert("branching".to_string(), vec![
        Example {
            title: "Create and switch to a new branch".to_string(),
            description: "Create a feature branch for new development".to_string(),
            commands: vec![
                "rune checkout -b feature/user-auth".to_string(),
            ],
            expected_output: Some("✓ Switched to a new branch 'feature/user-auth'".to_string()),
            category: "branching".to_string(),
        },
        Example {
            title: "List all branches".to_string(),
            description: "See all local branches and which one is currently active".to_string(),
            commands: vec![
                "rune branch".to_string(),
            ],
            expected_output: Some("  main\n* feature/user-auth".to_string()),
            category: "branching".to_string(),
        },
        Example {
            title: "Switch between branches".to_string(),
            description: "Switch back and forth between branches".to_string(),
            commands: vec![
                "rune checkout main".to_string(),
                "rune checkout feature/user-auth".to_string(),
            ],
            expected_output: None,
            category: "branching".to_string(),
        },
        Example {
            title: "Merge a feature branch".to_string(),
            description: "Merge your feature branch back into main".to_string(),
            commands: vec![
                "rune checkout main".to_string(),
                "rune merge feature/user-auth".to_string(),
            ],
            expected_output: Some("✓ Merged branch 'feature/user-auth' into main".to_string()),
            category: "branching".to_string(),
        },
        Example {
            title: "Delete a merged branch".to_string(),
            description: "Clean up branches that have been merged".to_string(),
            commands: vec![
                "rune branch -d feature/user-auth".to_string(),
            ],
            expected_output: Some("✓ Deleted branch 'feature/user-auth'".to_string()),
            category: "branching".to_string(),
        },
    ]);
    
    // Remote operations
    examples.insert("remote".to_string(), vec![
        Example {
            title: "Clone a repository".to_string(),
            description: "Create a local copy of a remote repository".to_string(),
            commands: vec![
                "rune clone https://github.com/user/repo.git".to_string(),
                "cd repo".to_string(),
            ],
            expected_output: Some("✓ Cloned repository to 'repo'".to_string()),
            category: "remote".to_string(),
        },
        Example {
            title: "Push changes to remote".to_string(),
            description: "Upload your local commits to the remote repository".to_string(),
            commands: vec![
                "rune push".to_string(),
            ],
            expected_output: Some("✓ Pushed to origin/main".to_string()),
            category: "remote".to_string(),
        },
        Example {
            title: "Pull latest changes".to_string(),
            description: "Download and merge changes from the remote repository".to_string(),
            commands: vec![
                "rune pull".to_string(),
            ],
            expected_output: Some("✓ Pulled latest changes from origin/main".to_string()),
            category: "remote".to_string(),
        },
        Example {
            title: "Fetch without merging".to_string(),
            description: "Download remote changes without merging them".to_string(),
            commands: vec![
                "rune fetch".to_string(),
            ],
            expected_output: Some("✓ Fetched from origin".to_string()),
            category: "remote".to_string(),
        },
    ]);
    
    // Ignore system examples
    examples.insert("ignore".to_string(), vec![
        Example {
            title: "Initialize smart ignore system".to_string(),
            description: "Set up intelligent ignore patterns for your project".to_string(),
            commands: vec![
                "rune ignore init".to_string(),
            ],
            expected_output: Some("✓ Smart ignore configuration initialized\nℹ Auto-detected project templates: rust".to_string()),
            category: "ignore".to_string(),
        },
        Example {
            title: "Add custom ignore pattern".to_string(),
            description: "Add a new pattern to ignore specific files".to_string(),
            commands: vec![
                "rune ignore add '**/*.log' --description 'Log files' --priority 80".to_string(),
            ],
            expected_output: Some("✓ Added ignore pattern '**/*.log' to project configuration".to_string()),
            category: "ignore".to_string(),
        },
        Example {
            title: "Check if files are ignored".to_string(),
            description: "Test whether specific files would be ignored".to_string(),
            commands: vec![
                "rune ignore check app.log build/output.txt README.md".to_string(),
            ],
            expected_output: Some("📁 app.log: ❌ IGNORED\n📁 build/output.txt: ❌ IGNORED\n📁 README.md: ✅ TRACKED".to_string()),
            category: "ignore".to_string(),
        },
        Example {
            title: "Debug ignore decisions".to_string(),
            description: "Understand why files are being ignored".to_string(),
            commands: vec![
                "rune ignore check --debug app.log".to_string(),
            ],
            expected_output: Some("📁 app.log: ❌ IGNORED\n  📋 Matched Rules:\n    🔸 **/*.log (priority: 80) - Log files\n  🎯 Final Decision: **/*.log - Log files".to_string()),
            category: "ignore".to_string(),
        },
        Example {
            title: "List all ignore rules".to_string(),
            description: "See all active ignore patterns and templates".to_string(),
            commands: vec![
                "rune ignore list".to_string(),
            ],
            expected_output: None,
            category: "ignore".to_string(),
        },
    ]);
    
    // File operations
    examples.insert("files".to_string(), vec![
        Example {
            title: "Stage specific files".to_string(),
            description: "Add only certain files to the staging area".to_string(),
            commands: vec![
                "rune add src/main.rs tests/test_main.rs".to_string(),
            ],
            expected_output: Some("✓ Added files to staging area".to_string()),
            category: "files".to_string(),
        },
        Example {
            title: "Stage all changes".to_string(),
            description: "Add all modified files to staging area".to_string(),
            commands: vec![
                "rune add .".to_string(),
            ],
            expected_output: Some("✓ Added all changes to staging area".to_string()),
            category: "files".to_string(),
        },
        Example {
            title: "View file differences".to_string(),
            description: "See what changed in your files".to_string(),
            commands: vec![
                "rune diff".to_string(),
                "rune diff src/main.rs".to_string(),
            ],
            expected_output: None,
            category: "files".to_string(),
        },
        Example {
            title: "Reset staged files".to_string(),
            description: "Remove files from staging area without losing changes".to_string(),
            commands: vec![
                "rune reset src/main.rs".to_string(),
            ],
            expected_output: Some("✓ Reset src/main.rs from staging area".to_string()),
            category: "files".to_string(),
        },
        Example {
            title: "Discard working changes".to_string(),
            description: "Permanently discard changes to files (dangerous!)".to_string(),
            commands: vec![
                "rune reset --hard HEAD".to_string(),
            ],
            expected_output: Some("⚠️  This will permanently discard all uncommitted changes!\n✓ Reset to HEAD".to_string()),
            category: "files".to_string(),
        },
    ]);
    
    // Workflow examples
    examples.insert("workflow".to_string(), vec![
        Example {
            title: "Feature development workflow".to_string(),
            description: "Complete workflow for developing a new feature".to_string(),
            commands: vec![
                "rune checkout -b feature/new-parser".to_string(),
                "# ... make changes ...".to_string(),
                "rune add .".to_string(),
                "rune commit -m 'feat(parser): add JSON support'".to_string(),
                "rune checkout main".to_string(),
                "rune merge feature/new-parser".to_string(),
                "rune branch -d feature/new-parser".to_string(),
            ],
            expected_output: None,
            category: "workflow".to_string(),
        },
        Example {
            title: "Hotfix workflow".to_string(),
            description: "Quick workflow for critical bug fixes".to_string(),
            commands: vec![
                "rune checkout -b hotfix/security-fix".to_string(),
                "# ... make critical fix ...".to_string(),
                "rune add .".to_string(),
                "rune commit -m 'fix(security): patch XSS vulnerability'".to_string(),
                "rune checkout main".to_string(),
                "rune merge hotfix/security-fix".to_string(),
                "rune push".to_string(),
            ],
            expected_output: None,
            category: "workflow".to_string(),
        },
        Example {
            title: "Daily development routine".to_string(),
            description: "Typical daily workflow for developers".to_string(),
            commands: vec![
                "rune status".to_string(),
                "rune pull".to_string(),
                "# ... work on code ...".to_string(),
                "rune add .".to_string(),
                "rune commit -m 'feat: implement user dashboard'".to_string(),
                "rune push".to_string(),
            ],
            expected_output: None,
            category: "workflow".to_string(),
        },
    ]);
    
    // Migration examples
    examples.insert("migration".to_string(), vec![
        Example {
            title: "Convert Git repository to Rune".to_string(),
            description: "Migrate an existing Git repository to Rune".to_string(),
            commands: vec![
                "cd existing-git-repo".to_string(),
                "rune init".to_string(),
                "rune ignore init".to_string(),
                "rune add .".to_string(),
                "rune commit -m 'migrate: convert to Rune VCS'".to_string(),
            ],
            expected_output: None,
            category: "migration".to_string(),
        },
        Example {
            title: "Git command equivalents".to_string(),
            description: "Common Git commands and their Rune equivalents".to_string(),
            commands: vec![
                "# Git: git status".to_string(),
                "rune status".to_string(),
                "# Git: git add .".to_string(),
                "rune add .".to_string(),
                "# Git: git commit -m 'message'".to_string(),
                "rune commit -m 'message'".to_string(),
                "# Git: git log --oneline".to_string(),
                "rune log --oneline".to_string(),
            ],
            expected_output: None,
            category: "migration".to_string(),
        },
    ]);
    
    // Troubleshooting examples
    examples.insert("troubleshooting".to_string(), vec![
        Example {
            title: "Fix merge conflicts".to_string(),
            description: "Resolve conflicts when merging branches".to_string(),
            commands: vec![
                "rune status".to_string(),
                "# Edit conflicted files manually".to_string(),
                "rune add <resolved-files>".to_string(),
                "rune commit -m 'resolve merge conflicts'".to_string(),
            ],
            expected_output: None,
            category: "troubleshooting".to_string(),
        },
        Example {
            title: "Undo last commit".to_string(),
            description: "Safely undo the most recent commit".to_string(),
            commands: vec![
                "rune reset --soft HEAD~1".to_string(),
                "# Make changes...".to_string(),
                "rune add .".to_string(),
                "rune commit -m 'corrected commit'".to_string(),
            ],
            expected_output: None,
            category: "troubleshooting".to_string(),
        },
        Example {
            title: "Check ignore patterns".to_string(),
            description: "Debug why files aren't being tracked".to_string(),
            commands: vec![
                "rune ignore check --debug suspicious_file.txt".to_string(),
                "rune ignore list".to_string(),
            ],
            expected_output: None,
            category: "troubleshooting".to_string(),
        },
    ]);
    
    examples
}
