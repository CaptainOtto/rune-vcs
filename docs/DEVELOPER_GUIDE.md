# Rune VCS - Developer Guide

## Architecture Overview

Rune VCS is built with a modular architecture consisting of 8 specialized crates:

### Core Crates

#### `rune-core` - Data Structures
Core data structures and types used throughout the system:
- `Author` - Commit author information
- `Commit` - Commit metadata and relationships
- Intelligence module for smart operations

```rust
use rune_core::{Author, Commit};

let author = Author {
    name: "Developer".to_string(),
    email: "dev@example.com".to_string(),
};
```

#### `rune-store` - Repository Storage
Handles all repository storage operations:
- Repository creation and discovery
- File staging and commit operations
- Branch and reference management
- Configuration management

```rust
use rune_store::{Store, Author};
use std::path::Path;

// Open existing repository
let store = Store::open(Path::new("/path/to/repo"))?;

// Create new repository
let store = Store::open(Path::new("/new/repo"))?;
store.create()?;

// Stage and commit files
store.stage_file("README.md")?;
let commit = store.commit("Initial commit", author)?;
```

### Specialized Crates

#### `rune-delta` - Binary Delta Compression
Efficient binary delta compression for storage optimization:

```rust
use rune_delta::{make, apply};

let base = b"Hello, World!";
let new = b"Hello, Universe!";

// Create delta patch
let patch = make(base, new, 8)?;

// Apply patch to reconstruct new data
let result = apply(base, &patch)?;
assert_eq!(result, new);
```

#### `rune-pack` - Blob Packing
Compression and packing of multiple blobs:

```rust
use rune_pack::{pack_blobs, unpack_blob};

let blobs = vec![
    ("file1.txt".to_string(), b"content1".to_vec()),
    ("file2.txt".to_string(), b"content2".to_vec()),
];

let (pack_data, index) = pack_blobs(blobs)?;

// Extract individual blob
let entry = index.find_entry("file1.txt").unwrap();
let content = unpack_blob(&pack_data, entry)?;
```

#### `rune-lfs` - Large File Storage
Handles large files and file locking:

```rust
use rune_lfs::{LfsConfig, LockManager};

let config = LfsConfig {
    size_threshold: 10 * 1024 * 1024, // 10MB
    patterns: vec!["*.bin".to_string(), "*.dll".to_string()],
};

let lock_manager = LockManager::new();
```

#### `rune-performance` - Performance Optimization
Performance metrics and optimization strategies:

```rust
use rune_performance::{PerformanceEngine, PerformanceConfig};

let config = PerformanceConfig::default();
let engine = PerformanceEngine::new(config);

// Optimize operations
engine.optimize_storage(&data)?;
engine.smart_delta(&base, &new)?;
```

#### `rune-remote` - Network Operations
Handles remote repository operations and shrine integration:

```rust
use rune_remote::Shrine;

let shrine = Shrine::new("https://remote.example.com".to_string());
```

#### `rune-cli` - Command Line Interface
User-facing CLI application built on top of the core libraries.

## Testing Strategy

### Unit Tests (82 total)
- **rune-core**: 5 tests (Author, Commit structures)
- **rune-store**: 18 tests (Repository operations)
- **rune-delta**: 17 tests (Delta compression)
- **rune-pack**: 16 tests (Blob packing)
- **rune-performance**: 14 tests (Performance optimization)
- **rune-lfs**: 6 tests (Large file handling)
- **rune-remote**: 3 tests (Remote operations)
- **rune-cli**: 3 tests (CLI interface)

### Integration Tests
- Complete VCS workflows
- Multi-file operations
- Repository discovery
- Error handling scenarios

### Benchmarks
Performance benchmarks for critical operations:
- Delta compression performance
- Pack operation efficiency
- Store operation speed

Run benchmarks with:
```bash
cargo bench
```

## Development Workflow

### Setting Up Development Environment

1. Clone the repository
2. Install Rust (latest stable)
3. Run tests: `cargo test`
4. Run benchmarks: `cargo bench`
5. Build CLI: `cargo build --release`

### Adding New Features

1. Identify the appropriate crate for your feature
2. Add unit tests for the new functionality
3. Update integration tests if needed
4. Add benchmarks for performance-critical code
5. Update documentation

### Code Quality Standards

- All code must have >= 90% test coverage
- No unsafe code allowed (enforced by lints)
- All public APIs must be documented
- Performance-critical code must have benchmarks

## Performance Characteristics

### Delta Compression
- Efficient for text files and similar binaries
- ~10-90% compression ratio depending on similarity
- O(n*m) complexity where n=base size, m=new size

### Pack Operations
- Zstd compression for optimal size/speed tradeoff
- Batch operations for multiple files
- Incremental packing support

### Storage Operations
- Atomic commits with rollback support
- Concurrent read operations
- Lock-free index operations where possible

## Error Handling

Rune VCS uses `anyhow::Result` for error propagation with context:

```rust
use anyhow::{Result, Context};

fn example_operation() -> Result<()> {
    store.stage_file("file.txt")
        .context("Failed to stage file for commit")?;
    Ok(())
}
```

Common error scenarios:
- Repository not found
- File not found during staging
- Nothing staged for commit
- Network errors during remote operations
- Checksum mismatches in pack operations
