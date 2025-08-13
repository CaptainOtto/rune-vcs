# 🎯 Practical Examples: Real Repository Workflows

This document shows step-by-step examples of using Rune VCS in real projects.

## Example 1: Personal Software Project

Let's create a web application with Rune:

```bash
# 1. Set up the project
mkdir my-web-app
cd my-web-app
rune init

# 2. Create initial files
echo "# My Web App" > README.md
mkdir src assets docs
echo 'console.log("Hello, world!");' > src/app.js
echo "body { font-family: Arial; }" > assets/style.css

# 3. Add and commit
rune add .
rune commit -m "Initial project setup"

# 4. Check status
rune status
# Output: Clean working directory

# 5. Make some changes
echo "## Features\n- User authentication\n- Real-time updates" >> README.md
echo "const users = [];" >> src/app.js

# 6. Check what changed
rune status
# Output: Modified files: README.md, src/app.js

# 7. Stage and commit
rune add README.md src/app.js
rune commit -m "Add features list and user array"

# 8. View history
rune log
# Shows commit history with timestamps and messages
```

## Example 2: Team Game Development

Setting up a game project with large assets:

```bash
# 1. Initialize project
mkdir awesome-game
cd awesome-game
rune init

# 2. Configure LFS for game assets
rune lfs track "*.fbx"      # 3D models
rune lfs track "*.texture"  # Textures
rune lfs track "*.wav"      # Sound effects
rune lfs track "*.mp3"      # Music
rune lfs track "*.mp4"      # Cutscenes
rune lfs track "*.blend"    # Blender source files

# 3. Create project structure
mkdir -p src/{scripts,ui} assets/{models,textures,audio,videos}
echo "using UnityEngine;" > src/scripts/Player.cs
echo "{}" > assets/game-config.json

# 4. Add initial files
rune add src/ assets/game-config.json
rune commit -m "Initial game project structure"

# 5. Add a large asset (simulated)
echo "This represents a large 3D model file" > assets/models/player.fbx
# In real life, this would be a multi-MB file from Blender/Maya

# 6. Add the large file (automatically goes to LFS)
rune add assets/models/player.fbx
rune commit -m "Add player 3D model"

# 7. Start team server
rune api --with-shrine --addr 0.0.0.0:7421 --shrine-addr 0.0.0.0:7420
# Team members can now access:
# - API: http://your-server:7421
# - Large files: http://your-server:7420
```

### Game Development: Asset Locking Workflow

```bash
# Artist wants to edit a model
rune lfs lock --path assets/models/player.fbx --owner artist@studio.com

# Check locked files
curl http://your-server:7421/v1/locks
# Shows: player.fbx locked by artist@studio.com

# Artist makes changes and commits
# ... edit file in Blender ...
rune add assets/models/player.fbx
rune commit -m "Update player model with new animations"

# Release the lock
rune lfs unlock --path assets/models/player.fbx --owner artist@studio.com

# Push to team storage
rune lfs push assets/models/player.fbx
```

## Example 3: Design Agency Client Work

Managing a client project with design files:

```bash
# 1. Set up client project
mkdir acme-corp-rebrand
cd acme-corp-rebrand
rune init

# 2. Configure for design files
rune lfs track "*.psd"      # Photoshop
rune lfs track "*.ai"       # Illustrator
rune lfs track "*.sketch"   # Sketch
rune lfs track "*.fig"      # Figma exports
rune lfs track "*.indd"     # InDesign
rune lfs track "*.pdf"      # Client deliverables

# 3. Create project structure
mkdir -p design/{logos,assets,mockups} deliverables client-feedback
echo "# ACME Corp Rebranding Project" > README.md
echo "## Timeline\n- Logo concepts: Week 1\n- Website mockups: Week 2" >> README.md

# 4. Initial commit
rune add README.md design/ deliverables/ client-feedback/
rune commit -m "Project setup and timeline"

# 5. Designer starts working on logo
rune branch logo-concepts
rune checkout logo-concepts

# 6. Lock the logo file for editing
rune lfs lock --path design/logos/logo-main.psd --owner sarah@agency.com

# 7. Create and add logo (simulated)
echo "Photoshop logo file content" > design/logos/logo-main.psd
rune add design/logos/logo-main.psd
rune commit -m "Initial logo concept - modern minimal style"

# 8. Export for client review
echo "PNG export for client" > deliverables/logo-concept-v1.png
rune add deliverables/logo-concept-v1.png
rune commit -m "Export logo concept for client review"

# 9. Release lock after work session
rune lfs unlock --path design/logos/logo-main.psd --owner sarah@agency.com

# 10. Switch back to main branch
rune checkout main

# 11. Client feedback comes in
echo "Client feedback: Love the concept! Can you make it 20% larger?" > client-feedback/feedback-round1.txt
rune add client-feedback/feedback-round1.txt
rune commit -m "Client feedback: Round 1"
```

## Example 4: Open Source Project

Contributing to a collaborative open source project:

```bash
# 1. Set up the project
mkdir open-source-tool
cd open-source-tool
rune init

# 2. Create typical open source structure
mkdir -p src tests docs examples
echo "# Amazing Open Source Tool" > README.md
echo "MIT" > LICENSE
echo "node_modules/\n*.log\n.env" > .runeignore  # Like .gitignore

# 3. Set up for documentation assets
rune lfs track "*.png"      # Screenshots
rune lfs track "*.gif"      # Demo animations
rune lfs track "*.mp4"      # Video tutorials

# 4. Initial project files
cat > src/main.rs << EOF
fn main() {
    println!("Hello, open source world!");
}
EOF

cat > Cargo.toml << EOF
[package]
name = "amazing-tool"
version = "0.1.0"
edition = "2021"

[dependencies]
EOF

# 5. Add initial commit
rune add .
rune commit -m "Initial project setup with Rust structure"

# 6. Create feature branch
rune branch feature-config-loader
rune checkout feature-config-loader

# 7. Implement feature
cat > src/config.rs << EOF
use std::fs;

pub fn load_config(path: &str) -> Result<String, std::io::Error> {
    fs::read_to_string(path)
}
EOF

echo "mod config;" >> src/main.rs

# 8. Add tests
cat > tests/config_test.rs << EOF
#[test]
fn test_config_loading() {
    // Test implementation
    assert!(true);
}
EOF

# 9. Commit feature
rune add src/config.rs tests/config_test.rs src/main.rs
rune commit -m "Add configuration file loading feature"

# 10. Add documentation with screenshot
echo "Demo screenshot placeholder" > docs/config-example.png
rune add docs/config-example.png
rune commit -m "Add configuration documentation with example"

# 11. Back to main for "merging" (manual for now)
rune checkout main
# In the future, Rune would have built-in merge functionality
```

## Example 5: CI/CD Integration

Using Rune in automated workflows:

```bash
#!/bin/bash
# ci-script.sh - Example CI integration

# 1. Check repository status via API
rune api --addr 127.0.0.1:7421 &
API_PID=$!
sleep 2

# 2. Get current status
STATUS=$(curl -s http://127.0.0.1:7421/v1/status)
echo "Repository status: $STATUS"

# 3. Get recent commits for changelog
COMMITS=$(curl -s http://127.0.0.1:7421/v1/log)
echo "Recent commits: $COMMITS"

# 4. Check for locked files (might affect build)
LOCKS=$(curl -s http://127.0.0.1:7421/v1/locks)
if [ "$LOCKS" != "[]" ]; then
    echo "Warning: Some files are locked: $LOCKS"
fi

# 5. Build project
cargo build --release

# 6. If build succeeds, tag the commit
if [ $? -eq 0 ]; then
    rune commit -m "CI: Successful build $(date)"
fi

# 7. Clean up
kill $API_PID
```

## Example 6: Branch Management

Working with branches for different features:

```bash
# 1. List current branches
rune branch
# Output: * main

# 2. Create and switch to feature branch
rune branch user-authentication
rune checkout user-authentication

# 3. Work on the feature
mkdir src/auth
echo "Authentication logic here" > src/auth/mod.rs
rune add src/auth/
rune commit -m "Add authentication module structure"

# 4. Continue development
echo "Login function implementation" >> src/auth/mod.rs
rune add src/auth/mod.rs
rune commit -m "Implement login functionality"

# 5. Switch back to main
rune checkout main

# 6. Create another feature branch
rune branch payment-system
rune checkout payment-system

# 7. Work on different feature
mkdir src/payments
echo "Payment processing code" > src/payments/mod.rs
rune add src/payments/
rune commit -m "Add payment processing module"

# 8. View all branches
rune branch
# Output:
# * payment-system
#   user-authentication
#   main

# 9. Switch between branches
rune checkout user-authentication  # Continue auth work
rune checkout main                 # Back to main
```

## Example 7: Team Server Setup

Production server configuration:

```bash
# 1. On the server machine
mkdir -p /var/rune/{repos,lfs,logs}
cd /var/rune/repos

# 2. Initialize main repository
rune init

# 3. Start services (production)
# Terminal 1: API Server
rune api --addr 0.0.0.0:7421 > /var/rune/logs/api.log 2>&1 &

# Terminal 2: Shrine Server (Large Files)
rune shrine serve --addr 0.0.0.0:7420 > /var/rune/logs/shrine.log 2>&1 &

# 4. Or start embedded mode
rune api --with-shrine \
  --addr 0.0.0.0:7421 \
  --shrine-addr 0.0.0.0:7420 \
  > /var/rune/logs/combined.log 2>&1 &

# 5. Team members connect
export RUNE_API_URL="http://your-server:7421"
export RUNE_SHRINE_URL="http://your-server:7420"

# 6. Test connection
curl $RUNE_API_URL/v1/status
```

## Tips for Each Workflow

### Development Best Practices

- Use descriptive commit messages
- Track large files with LFS early
- Lock binary files before editing
- Use branches for features

### Performance Tips

- Track file patterns broadly (`*.psd`) rather than individual files
- Use `--format=json` for scripting
- Start servers with appropriate resource limits

### Team Collaboration

- Establish locking conventions for binary assets
- Use meaningful branch names
- Communicate about large file changes
- Set up automated backups of LFS storage

---

_For more advanced configuration and API usage, see the other documentation files._
