use crate::{Tutorial, TutorialStep, TutorialDifficulty};

/// Load all interactive tutorials
pub fn load_all_tutorials() -> Vec<Tutorial> {
    vec![
        create_basics_tutorial(),
        create_branching_tutorial(),
        create_collaboration_tutorial(),
        create_advanced_tutorial(),
    ]
}

/// Basic tutorial for new users
fn create_basics_tutorial() -> Tutorial {
    Tutorial {
        id: "basics".to_string(),
        title: "Rune VCS Basics".to_string(),
        description: "Learn the fundamental concepts and commands of Rune VCS".to_string(),
        difficulty: TutorialDifficulty::Beginner,
        steps: vec![
            TutorialStep {
                title: "Welcome to Rune VCS".to_string(),
                instruction: "Welcome! This tutorial will teach you the basics of Rune VCS. We'll start by creating a new repository.".to_string(),
                command: None,
                expected_files: None,
                verification: None,
            },
            TutorialStep {
                title: "Create a new repository".to_string(),
                instruction: "Let's initialize a new Rune repository. This creates a .rune/ directory to store version control data.".to_string(),
                command: Some("rune init".to_string()),
                expected_files: Some(vec![".rune/".to_string()]),
                verification: Some("Repository initialized successfully!".to_string()),
            },
            TutorialStep {
                title: "Check repository status".to_string(),
                instruction: "The status command shows you what's happening in your repository - what files are modified, staged, etc.".to_string(),
                command: Some("rune status".to_string()),
                expected_files: None,
                verification: Some("Status shows clean working tree".to_string()),
            },
            TutorialStep {
                title: "Create your first file".to_string(),
                instruction: "Let's create a README file for our project. This will be our first tracked file.".to_string(),
                command: Some("echo '# My First Rune Project' > README.md".to_string()),
                expected_files: Some(vec!["README.md".to_string()]),
                verification: Some("README.md file created".to_string()),
            },
            TutorialStep {
                title: "Check status again".to_string(),
                instruction: "Now let's see how the status changed after creating a file. Rune will show it as an untracked file.".to_string(),
                command: Some("rune status".to_string()),
                expected_files: None,
                verification: Some("README.md appears as untracked".to_string()),
            },
            TutorialStep {
                title: "Stage the file".to_string(),
                instruction: "Before committing, we need to stage files. This tells Rune which changes to include in the next commit.".to_string(),
                command: Some("rune add README.md".to_string()),
                expected_files: None,
                verification: Some("README.md is now staged".to_string()),
            },
            TutorialStep {
                title: "Check status after staging".to_string(),
                instruction: "Let's see how staging changed the status. The file should now appear as 'staged for commit'.".to_string(),
                command: Some("rune status".to_string()),
                expected_files: None,
                verification: Some("README.md shows as staged".to_string()),
            },
            TutorialStep {
                title: "Create your first commit".to_string(),
                instruction: "Now let's create a commit - a snapshot of your project at this point in time. Use a descriptive message!".to_string(),
                command: Some("rune commit -m 'Initial commit: add README'".to_string()),
                expected_files: None,
                verification: Some("First commit created successfully".to_string()),
            },
            TutorialStep {
                title: "View commit history".to_string(),
                instruction: "The log command shows you the history of commits. You should see your first commit!".to_string(),
                command: Some("rune log".to_string()),
                expected_files: None,
                verification: Some("Commit history shows initial commit".to_string()),
            },
            TutorialStep {
                title: "Make another change".to_string(),
                instruction: "Let's add more content to our README to practice the workflow again.".to_string(),
                command: Some("echo '\nThis is my first Rune VCS project!' >> README.md".to_string()),
                expected_files: None,
                verification: Some("README.md has been modified".to_string()),
            },
            TutorialStep {
                title: "View differences".to_string(),
                instruction: "The diff command shows you exactly what changed in your files since the last commit.".to_string(),
                command: Some("rune diff".to_string()),
                expected_files: None,
                verification: Some("Diff shows the added line".to_string()),
            },
            TutorialStep {
                title: "Stage and commit changes".to_string(),
                instruction: "Let's stage and commit these changes in one go using the -a flag (stage all modified files).".to_string(),
                command: Some("rune commit -a -m 'docs: expand README with project description'".to_string()),
                expected_files: None,
                verification: Some("Second commit created".to_string()),
            },
            TutorialStep {
                title: "Congratulations!".to_string(),
                instruction: "Great job! You've learned the basic Rune workflow:\n1. Make changes\n2. Stage files (rune add)\n3. Commit changes (rune commit)\n4. Check status and history\n\nTry the branching tutorial next: rune tutorial branching".to_string(),
                command: None,
                expected_files: None,
                verification: None,
            },
        ],
    }
}

/// Branching and merging tutorial
fn create_branching_tutorial() -> Tutorial {
    Tutorial {
        id: "branching".to_string(),
        title: "Branching and Merging".to_string(),
        description: "Learn how to work with branches for parallel development".to_string(),
        difficulty: TutorialDifficulty::Intermediate,
        steps: vec![
            TutorialStep {
                title: "Introduction to Branching".to_string(),
                instruction: "Branches let you work on different features in parallel. You can experiment safely without affecting the main codebase.".to_string(),
                command: None,
                expected_files: None,
                verification: None,
            },
            TutorialStep {
                title: "Check current branch".to_string(),
                instruction: "Let's see what branch we're currently on. By default, Rune starts with a 'main' branch.".to_string(),
                command: Some("rune branch".to_string()),
                expected_files: None,
                verification: Some("Shows current branch (main)".to_string()),
            },
            TutorialStep {
                title: "Create a feature branch".to_string(),
                instruction: "Let's create a new branch for adding a features section to our README. The -b flag creates and switches to the new branch.".to_string(),
                command: Some("rune checkout -b feature/add-features".to_string()),
                expected_files: None,
                verification: Some("Switched to new branch 'feature/add-features'".to_string()),
            },
            TutorialStep {
                title: "Verify branch switch".to_string(),
                instruction: "Let's confirm we're on the new branch. The asterisk (*) shows your current branch.".to_string(),
                command: Some("rune branch".to_string()),
                expected_files: None,
                verification: Some("Current branch is feature/add-features".to_string()),
            },
            TutorialStep {
                title: "Make changes on the feature branch".to_string(),
                instruction: "Now let's add a features section to our README. This change will only exist on this branch.".to_string(),
                command: Some("echo '\n## Features\n- Version control\n- Branching support\n- Smart ignore system' >> README.md".to_string()),
                expected_files: None,
                verification: Some("README.md modified on feature branch".to_string()),
            },
            TutorialStep {
                title: "Commit the feature".to_string(),
                instruction: "Let's commit our new feature to the feature branch.".to_string(),
                command: Some("rune commit -a -m 'feat: add features section to README'".to_string()),
                expected_files: None,
                verification: Some("Feature committed to branch".to_string()),
            },
            TutorialStep {
                title: "Switch back to main".to_string(),
                instruction: "Let's switch back to main to see that our changes don't exist there yet.".to_string(),
                command: Some("rune checkout main".to_string()),
                expected_files: None,
                verification: Some("Switched back to main branch".to_string()),
            },
            TutorialStep {
                title: "Verify main is unchanged".to_string(),
                instruction: "Check the README - the features section shouldn't be there on main branch.".to_string(),
                command: Some("cat README.md".to_string()),
                expected_files: None,
                verification: Some("README doesn't have features section".to_string()),
            },
            TutorialStep {
                title: "Merge the feature branch".to_string(),
                instruction: "Now let's merge our feature branch into main. This brings the changes from the feature branch into main.".to_string(),
                command: Some("rune merge feature/add-features".to_string()),
                expected_files: None,
                verification: Some("Feature branch merged successfully".to_string()),
            },
            TutorialStep {
                title: "Verify the merge".to_string(),
                instruction: "Let's check that our features section is now in main branch.".to_string(),
                command: Some("cat README.md".to_string()),
                expected_files: None,
                verification: Some("README now includes features section".to_string()),
            },
            TutorialStep {
                title: "Clean up merged branch".to_string(),
                instruction: "Since we've merged the feature, we can delete the feature branch to keep things tidy.".to_string(),
                command: Some("rune branch -d feature/add-features".to_string()),
                expected_files: None,
                verification: Some("Feature branch deleted".to_string()),
            },
            TutorialStep {
                title: "View commit history".to_string(),
                instruction: "Let's see how the merge appears in our commit history.".to_string(),
                command: Some("rune log --oneline".to_string()),
                expected_files: None,
                verification: Some("History shows merge commit".to_string()),
            },
            TutorialStep {
                title: "Branching mastery achieved!".to_string(),
                instruction: "Excellent! You now know how to:\n- Create branches (rune checkout -b)\n- Switch branches (rune checkout)\n- Merge branches (rune merge)\n- Clean up branches (rune branch -d)\n\nTry the collaboration tutorial next: rune tutorial collaboration".to_string(),
                command: None,
                expected_files: None,
                verification: None,
            },
        ],
    }
}

/// Collaboration and remote repositories tutorial
fn create_collaboration_tutorial() -> Tutorial {
    Tutorial {
        id: "collaboration".to_string(),
        title: "Collaboration and Remotes".to_string(),
        description: "Learn how to work with remote repositories and collaborate with others".to_string(),
        difficulty: TutorialDifficulty::Intermediate,
        steps: vec![
            TutorialStep {
                title: "Introduction to Collaboration".to_string(),
                instruction: "Rune makes it easy to collaborate with others using remote repositories. Let's learn the key commands for sharing code.".to_string(),
                command: None,
                expected_files: None,
                verification: None,
            },
            TutorialStep {
                title: "Simulate a remote repository".to_string(),
                instruction: "For this tutorial, we'll create a 'remote' repository on your local machine to simulate collaboration.".to_string(),
                command: Some("mkdir ../remote-repo && cd ../remote-repo && rune init --bare".to_string()),
                expected_files: None,
                verification: Some("Remote repository created".to_string()),
            },
            TutorialStep {
                title: "Clone the remote repository".to_string(),
                instruction: "Let's clone our 'remote' repository to simulate a fresh checkout that a collaborator might do.".to_string(),
                command: Some("cd .. && rune clone remote-repo collaborative-project && cd collaborative-project".to_string()),
                expected_files: None,
                verification: Some("Repository cloned successfully".to_string()),
            },
            TutorialStep {
                title: "Set up initial content".to_string(),
                instruction: "Let's add some initial content to our collaborative project.".to_string(),
                command: Some("echo '# Collaborative Project\n\nThis project demonstrates Rune collaboration features.' > README.md".to_string()),
                expected_files: Some(vec!["README.md".to_string()]),
                verification: Some("Initial content created".to_string()),
            },
            TutorialStep {
                title: "Commit and push initial content".to_string(),
                instruction: "Let's commit our initial content and push it to the remote repository.".to_string(),
                command: Some("rune add README.md && rune commit -m 'Initial commit: project setup'".to_string()),
                expected_files: None,
                verification: Some("Initial commit created".to_string()),
            },
            TutorialStep {
                title: "Push to remote".to_string(),
                instruction: "Now let's push our commits to the remote repository so others can access them.".to_string(),
                command: Some("rune push".to_string()),
                expected_files: None,
                verification: Some("Changes pushed to remote".to_string()),
            },
            TutorialStep {
                title: "Simulate another developer".to_string(),
                instruction: "Let's simulate another developer by cloning the repository to a different location.".to_string(),
                command: Some("cd .. && rune clone remote-repo developer-2 && cd developer-2".to_string()),
                expected_files: None,
                verification: Some("Second clone created".to_string()),
            },
            TutorialStep {
                title: "Make changes as second developer".to_string(),
                instruction: "As the second developer, let's add a contributing guide.".to_string(),
                command: Some("echo '# Contributing\n\nWelcome contributors! Please follow these guidelines...' > CONTRIBUTING.md".to_string()),
                expected_files: Some(vec!["CONTRIBUTING.md".to_string()]),
                verification: Some("Contributing guide created".to_string()),
            },
            TutorialStep {
                title: "Commit and push from second developer".to_string(),
                instruction: "Let's commit and push the contributing guide.".to_string(),
                command: Some("rune add CONTRIBUTING.md && rune commit -m 'docs: add contributing guidelines' && rune push".to_string()),
                expected_files: None,
                verification: Some("Contributing guide pushed".to_string()),
            },
            TutorialStep {
                title: "Switch back to first developer".to_string(),
                instruction: "Now let's switch back to the first developer's workspace.".to_string(),
                command: Some("cd ../collaborative-project".to_string()),
                expected_files: None,
                verification: Some("Back in first developer workspace".to_string()),
            },
            TutorialStep {
                title: "Pull latest changes".to_string(),
                instruction: "The first developer needs to pull the latest changes to get the contributing guide.".to_string(),
                command: Some("rune pull".to_string()),
                expected_files: Some(vec!["CONTRIBUTING.md".to_string()]),
                verification: Some("Latest changes pulled successfully".to_string()),
            },
            TutorialStep {
                title: "Verify collaboration".to_string(),
                instruction: "Let's verify that we now have both files from both developers.".to_string(),
                command: Some("ls -la".to_string()),
                expected_files: Some(vec!["README.md".to_string(), "CONTRIBUTING.md".to_string()]),
                verification: Some("Both files present".to_string()),
            },
            TutorialStep {
                title: "View collaborative history".to_string(),
                instruction: "Let's see the commit history showing contributions from both developers.".to_string(),
                command: Some("rune log --oneline".to_string()),
                expected_files: None,
                verification: Some("History shows both commits".to_string()),
            },
            TutorialStep {
                title: "Collaboration complete!".to_string(),
                instruction: "Perfect! You now understand the collaboration workflow:\n1. Clone repositories (rune clone)\n2. Make changes and commit\n3. Push changes (rune push)\n4. Pull others' changes (rune pull)\n\nThis is the foundation of team development with Rune!".to_string(),
                command: None,
                expected_files: None,
                verification: None,
            },
        ],
    }
}

/// Advanced features tutorial
fn create_advanced_tutorial() -> Tutorial {
    Tutorial {
        id: "advanced".to_string(),
        title: "Advanced Rune Features".to_string(),
        description: "Explore Rune's advanced features like smart ignore system and optimization tools".to_string(),
        difficulty: TutorialDifficulty::Advanced,
        steps: vec![
            TutorialStep {
                title: "Advanced Features Overview".to_string(),
                instruction: "This tutorial covers Rune's advanced features that give it an edge over traditional VCS tools.".to_string(),
                command: None,
                expected_files: None,
                verification: None,
            },
            TutorialStep {
                title: "Initialize smart ignore system".to_string(),
                instruction: "Rune's ignore system is much more powerful than traditional .gitignore. Let's set it up.".to_string(),
                command: Some("rune ignore init".to_string()),
                expected_files: Some(vec![".runeignore.yml".to_string()]),
                verification: Some("Smart ignore system initialized".to_string()),
            },
            TutorialStep {
                title: "Examine auto-detected patterns".to_string(),
                instruction: "Rune automatically detected project type and applied appropriate ignore patterns. Let's see what it found.".to_string(),
                command: Some("rune ignore list".to_string()),
                expected_files: None,
                verification: Some("Auto-detected patterns listed".to_string()),
            },
            TutorialStep {
                title: "Create test files".to_string(),
                instruction: "Let's create some files to test the ignore system.".to_string(),
                command: Some("mkdir -p build logs && echo 'temp' > build/output.bin && echo 'debug info' > logs/debug.log && echo 'source' > src/main.txt".to_string()),
                expected_files: Some(vec!["build/output.bin".to_string(), "logs/debug.log".to_string(), "src/main.txt".to_string()]),
                verification: Some("Test files created".to_string()),
            },
            TutorialStep {
                title: "Test ignore patterns".to_string(),
                instruction: "Let's check which files would be ignored by our smart ignore system.".to_string(),
                command: Some("rune ignore check build/output.bin logs/debug.log src/main.txt".to_string()),
                expected_files: None,
                verification: Some("Ignore status shown for all files".to_string()),
            },
            TutorialStep {
                title: "Debug ignore decisions".to_string(),
                instruction: "Let's use debug mode to understand exactly why files are being ignored.".to_string(),
                command: Some("rune ignore check --debug logs/debug.log".to_string()),
                expected_files: None,
                verification: Some("Debug information shows rule matching".to_string()),
            },
            TutorialStep {
                title: "Add custom ignore pattern".to_string(),
                instruction: "Let's add a custom ignore pattern for temporary files.".to_string(),
                command: Some("rune ignore add '**/*.tmp' --description 'Temporary files' --priority 90".to_string()),
                expected_files: None,
                verification: Some("Custom pattern added".to_string()),
            },
            TutorialStep {
                title: "Test custom pattern".to_string(),
                instruction: "Let's create a temporary file and verify it's ignored.".to_string(),
                command: Some("echo 'temporary' > test.tmp && rune ignore check test.tmp".to_string()),
                expected_files: Some(vec!["test.tmp".to_string()]),
                verification: Some("Temporary file is ignored".to_string()),
            },
            TutorialStep {
                title: "Check repository status with ignores".to_string(),
                instruction: "Let's see how the ignore system affects repository status.".to_string(),
                command: Some("rune status".to_string()),
                expected_files: None,
                verification: Some("Only non-ignored files shown in status".to_string()),
            },
            TutorialStep {
                title: "View ignore configuration".to_string(),
                instruction: "Let's examine the ignore configuration file that Rune created.".to_string(),
                command: Some("cat .runeignore.yml".to_string()),
                expected_files: None,
                verification: Some("YAML configuration displayed".to_string()),
            },
            TutorialStep {
                title: "Explore documentation system".to_string(),
                instruction: "Rune has built-in documentation. Let's explore what's available.".to_string(),
                command: Some("rune help".to_string()),
                expected_files: None,
                verification: Some("Help system shows available commands".to_string()),
            },
            TutorialStep {
                title: "Get command-specific help".to_string(),
                instruction: "Let's get detailed help for a specific command with examples.".to_string(),
                command: Some("rune help commit".to_string()),
                expected_files: None,
                verification: Some("Detailed help with examples shown".to_string()),
            },
            TutorialStep {
                title: "Explore examples".to_string(),
                instruction: "Rune includes extensive examples for common workflows.".to_string(),
                command: Some("rune examples".to_string()),
                expected_files: None,
                verification: Some("Example categories listed".to_string()),
            },
            TutorialStep {
                title: "Advanced mastery achieved!".to_string(),
                instruction: "Congratulations! You've mastered Rune's advanced features:\n- Smart ignore system with priority rules\n- Debug mode for understanding ignore decisions\n- Built-in documentation and examples\n- YAML-based configuration\n\nYou're now ready to use Rune for serious development work!".to_string(),
                command: None,
                expected_files: None,
                verification: None,
            },
        ],
    }
}
