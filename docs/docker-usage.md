# Docker Usage Guide

This guide covers how to use Rune VCS with Docker for development and deployment.

## Quick Start

### Pull from GitHub Container Registry

```bash
# Pull the latest image
docker pull ghcr.io/captainotto/rune-vcs:latest

# Run Rune doctor to verify installation
docker run --rm ghcr.io/captainotto/rune-vcs:latest rune doctor

# Run Rune with a workspace volume
docker run -it --rm -v $(pwd):/workspace ghcr.io/captainotto/rune-vcs:latest
```

### Build Locally

```bash
# Clone the repository
git clone https://github.com/CaptainOtto/rune-vcs.git
cd rune-vcs

# Build the Docker image
docker build -t rune-vcs:local .

# Run the container
docker run -it --rm -v $(pwd):/workspace rune-vcs:local
```

## Development Workflow

### Using Docker Compose

The repository includes a `docker-compose.yml` file for development:

```bash
# Start development container
docker-compose up -d rune-dev

# Enter the container
docker-compose exec rune-dev bash

# Inside the container, you can use Rune commands
rune init
rune status
rune doctor
```

### Available Services

- **rune-dev**: Development container with source code mounted
- **rune-workspace**: Container with persistent workspace volume
- **rune-test**: Quick testing container

## Use Cases

### 1. Isolated Development Environment

```bash
# Create an isolated development environment
docker run -it --rm \
    -v $(pwd):/workspace \
    -v rune-config:/home/rune/.config/rune \
    ghcr.io/captainotto/rune-vcs:latest \
    /bin/bash
```

### 2. CI/CD Integration

```yaml
# .github/workflows/test.yml
jobs:
  test:
    runs-on: ubuntu-latest
    container:
      image: ghcr.io/captainotto/rune-vcs:latest
    steps:
      - uses: actions/checkout@v4
      - name: Test with Rune
        run: |
          rune init
          rune add .
          rune commit -m "Test commit"
          rune status
```

### 3. Consistent Development Environment

```bash
# Create a development alias
alias rune-dev='docker run -it --rm \
    -v $(pwd):/workspace \
    -v rune-cache:/home/rune/.cache \
    ghcr.io/captainotto/rune-vcs:latest'

# Use it like a local command
rune-dev rune init
rune-dev rune status
```

### 4. Repository Backup and Migration

```bash
# Backup a repository
docker run --rm \
    -v $(pwd):/workspace \
    -v /backup:/backup \
    ghcr.io/captainotto/rune-vcs:latest \
    tar -czf /backup/repo-$(date +%Y%m%d).tar.gz /workspace/.rune

# Migrate from Git
docker run -it --rm \
    -v $(pwd):/workspace \
    ghcr.io/captainotto/rune-vcs:latest \
    bash -c "git log --oneline | head -10 && rune init"
```

## Configuration

### Environment Variables

- `RUNE_CONFIG_HOME`: Configuration directory (default: `/home/rune/.config/rune`)
- `RUNE_DATA_HOME`: Data directory (default: `/home/rune/.local/share/rune`)
- `RUST_LOG`: Logging level (debug, info, warn, error)
- `RUNE_DEVELOPMENT`: Enable development features

### Volumes

- `/workspace`: Working directory for repositories
- `/home/rune/.config/rune`: Configuration persistence
- `/home/rune/.local/share/rune`: Data persistence
- `/home/rune/.cache`: Cache directory

## Multi-Platform Support

The Docker image supports multiple architectures:

- `linux/amd64` (x86_64)
- `linux/arm64` (ARM64/Apple Silicon)

```bash
# Pull specific architecture
docker pull --platform linux/amd64 ghcr.io/captainotto/rune-vcs:latest
docker pull --platform linux/arm64 ghcr.io/captainotto/rune-vcs:latest
```

## Troubleshooting

### Permission Issues

If you encounter permission issues:

```bash
# Run as current user
docker run -it --rm \
    -v $(pwd):/workspace \
    -u $(id -u):$(id -g) \
    ghcr.io/captainotto/rune-vcs:latest
```

### Volume Persistence

For persistent configuration across container restarts:

```bash
# Create named volumes
docker volume create rune-config
docker volume create rune-data

# Use them in runs
docker run -it --rm \
    -v $(pwd):/workspace \
    -v rune-config:/home/rune/.config/rune \
    -v rune-data:/home/rune/.local/share/rune \
    ghcr.io/captainotto/rune-vcs:latest
```

### Debugging

```bash
# Run with debug logging
docker run -it --rm \
    -e RUST_LOG=debug \
    -v $(pwd):/workspace \
    ghcr.io/captainotto/rune-vcs:latest \
    rune doctor

# Get shell access for debugging
docker run -it --rm \
    -v $(pwd):/workspace \
    ghcr.io/captainotto/rune-vcs:latest \
    /bin/bash
```

## Security Considerations

- The container runs as non-root user `rune` for security
- Only necessary tools are included in the runtime image
- Use specific version tags in production rather than `latest`
- Mount only necessary directories to minimize attack surface

## Examples

### Initialize and Work with a Repository

```bash
# Start with current directory as workspace
docker run -it --rm -v $(pwd):/workspace ghcr.io/captainotto/rune-vcs:latest bash

# Inside container:
rune init
echo "Hello Rune" > README.md
rune add README.md
rune commit -m "Initial commit"
rune log
rune doctor
```

### Batch Operations

```bash
# Initialize multiple repositories
for repo in project1 project2 project3; do
    docker run --rm \
        -v $(pwd)/$repo:/workspace \
        ghcr.io/captainotto/rune-vcs:latest \
        rune init
done
```

This Docker setup provides a consistent, isolated environment for using Rune VCS across different systems and use cases.
