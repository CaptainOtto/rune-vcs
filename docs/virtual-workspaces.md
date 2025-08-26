# Virtual Workspaces

Rune's Virtual Workspace system provides Perforce-style client views and sparse checkout functionality, allowing teams to work efficiently with large repositories and monorepos.

## Overview

Virtual workspaces allow you to:

- **Focus on relevant code** – Only check out what you need
- **Scale to large monorepos** – Work with massive codebases efficiently
- **Maintain performance** – Automated guardrails prevent slowdowns
- **Support diverse teams** – Different views for different roles

## Key Concepts

### Virtual Roots

A virtual root defines a subset of the repository that should be active in your workspace. Each root has:

- **Name** – A descriptive identifier (e.g., "frontend", "mobile")
- **Pattern** – A glob pattern defining which files to include
- **Status** – Active or inactive

### Performance Guardrails

Automatic protection against:

- **File count limits** – Prevent checking out too many files
- **Size limits** – Block excessively large files
- **Binary detection** – Warn about untracked binary files
- **Extension blocking** – Prevent dangerous file types

## Basic Usage

### Initialize a Workspace

```bash
# Create a new workspace
rune workspace init mobile-development

# Or use current directory name
rune workspace init
```

### Configure Virtual Roots

```bash
# Focus on frontend code only
rune workspace add-root frontend "src/web/**"

# Add backend API code
rune workspace add-root backend "src/api/**"

# Include mobile app code
rune workspace add-root mobile "src/mobile/**"

# Add shared utilities
rune workspace add-root shared "src/shared/**"
```

### Toggle Roots On/Off

```bash
# Temporarily disable mobile development
rune workspace toggle mobile

# Re-enable when needed
rune workspace toggle mobile
```

### Monitor Workspace

```bash
# Check current configuration
rune workspace list

# Validate setup for issues
rune workspace validate

# View/adjust performance limits
rune workspace limits
rune workspace limits --max-files 10000 --max-size 200MB
```

## Common Patterns

### Monorepo Development

```bash
# Large enterprise monorepo
rune workspace init my-feature
rune workspace add-root auth "services/auth/**"
rune workspace add-root shared "libs/shared/**"
rune workspace add-root frontend "apps/web/**"

# Work on authentication feature only
rune workspace toggle frontend  # Disable temporarily
```

### Multi-Platform Projects

```bash
# Game development with multiple platforms
rune workspace init platform-development
rune workspace add-root engine "src/engine/**"
rune workspace add-root ios "platforms/ios/**"
rune workspace add-root android "platforms/android/**"
rune workspace add-root shared "assets/shared/**"

# Focus on iOS development
rune workspace toggle android  # Disable Android
```

### Role-Based Views

```bash
# Designer workspace - assets and UI only
rune workspace init design-view
rune workspace add-root assets "assets/**"
rune workspace add-root ui "src/components/**"
rune workspace add-root styles "src/styles/**"

# Backend developer workspace
rune workspace init backend-dev
rune workspace add-root api "src/api/**"
rune workspace add-root database "migrations/**"
rune workspace add-root shared "src/shared/**"
```

## Performance Optimization

### File Limits

```bash
# Set reasonable limits for your team size
rune workspace limits --max-files 5000   # Small team
rune workspace limits --max-files 15000  # Medium team
rune workspace limits --max-files 50000  # Large team
```

### Size Limits

```bash
# Prevent accidentally committing large files
rune workspace limits --max-size 50MB    # Conservative
rune workspace limits --max-size 100MB   # Standard
rune workspace limits --max-size 500MB   # Permissive
```

### Binary Detection

The workspace automatically detects and warns about binary files that should probably be tracked with LFS:

```bash
# Common patterns automatically detected:
# - Image files (.png, .jpg, .gif, etc.)
# - Video files (.mp4, .mov, .avi, etc.)
# - Archives (.zip, .tar.gz, .rar, etc.)
# - Executables (.exe, .app, .deb, etc.)
# - Office docs (.docx, .xlsx, .pptx, etc.)
```

## Integration with LFS

Virtual workspaces work seamlessly with Rune's LFS system:

```bash
# Set up workspace for creative project
rune workspace init creative-project
rune workspace add-root design "design/**"
rune workspace add-root assets "assets/**"

# Configure LFS for large files
rune lfs track "*.psd"     # Photoshop files
rune lfs track "*.ai"      # Illustrator files
rune lfs track "*.mp4"     # Video files

# Workspace will warn if large files aren't tracked by LFS
```

## Troubleshooting

### Workspace Too Large

```
Error: Workspace exceeds file limit (10,000 files)
```

**Solutions:**

1. Increase limits: `rune workspace limits --max-files 20000`
2. Add more specific patterns to reduce scope
3. Disable unnecessary virtual roots

### Pattern Conflicts

```
Warning: Overlapping patterns detected
```

**Solutions:**

1. Review patterns with `rune workspace list`
2. Make patterns more specific
3. Use `rune workspace validate` to check for issues

### Performance Issues

```
Warning: Large workspace may impact performance
```

**Solutions:**

1. Review active virtual roots with `rune workspace list`
2. Disable unused roots with `rune workspace toggle <name>`
3. Use more targeted patterns
4. Consider splitting into multiple workspaces

## Advanced Configuration

### Workspace Configuration File

Workspaces store configuration in `.rune/workspace.json`:

```json
{
  "name": "mobile-development",
  "virtual_roots": [
    {
      "name": "mobile",
      "pattern": "src/mobile/**",
      "active": true
    },
    {
      "name": "shared",
      "pattern": "src/shared/**",
      "active": true
    }
  ],
  "performance_limits": {
    "max_files": 10000,
    "max_total_size": 104857600,
    "blocked_extensions": [".exe", ".dll", ".so"]
  }
}
```

### Custom Extension Blocking

```bash
# Block specific file types
rune workspace limits --block-ext .tmp,.log,.cache

# Allow previously blocked extensions
rune workspace limits --allow-ext .exe
```

## Best Practices

1. **Start specific** – Begin with narrow patterns and expand as needed
2. **Use descriptive names** – Clear virtual root names help team coordination
3. **Regular validation** – Run `rune workspace validate` periodically
4. **Team coordination** – Share workspace configs for consistent environments
5. **Monitor performance** – Watch for warnings about large workspaces
6. **Combine with LFS** – Use LFS for large files to keep workspaces lean
