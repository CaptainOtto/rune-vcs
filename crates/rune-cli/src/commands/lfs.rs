use anyhow::Result;
use clap::Subcommand;
use rune_lfs::{Lfs, Pointer};

#[derive(Subcommand, Debug)]
pub enum LfsCmd {
    /// Track file patterns for LFS
    Track {
        patterns: Vec<String>,
    },
    /// Untrack file patterns from LFS
    Untrack {
        patterns: Vec<String>,
    },
    /// List currently tracked patterns
    Ls,
    /// Convert file to LFS pointer (clean)
    Smudge {
        path: std::path::PathBuf,
    },
    /// Convert LFS pointer back to file (smudge)
    Clean {
        path: std::path::PathBuf,
    },
    /// Configure LFS settings
    Config {
        #[arg(long)]
        remote: Option<String>,
        #[arg(long)]
        chunk_size: Option<usize>,
        #[arg(long)]
        migration_threshold: Option<String>,
        #[arg(long)]
        list: bool,
    },
    /// Upload LFS file to remote
    Push {
        path: std::path::PathBuf,
    },
    /// Download LFS file from remote
    Pull {
        oid: String,
        out: std::path::PathBuf,
    },
    /// Migrate existing large files to LFS
    Migrate {
        #[arg(long, default_value = "10MB")]
        min_size: String,
        #[arg(long)]
        dry_run: bool,
        /// Migrate entire directory
        #[arg(long)]
        directory: Option<std::path::PathBuf>,
    },
    /// Show LFS status and statistics
    Status,
    /// Sync with remote LFS server
    Sync,
    /// Lock file for editing
    Lock {
        #[arg(long)]
        path: String,
        #[arg(long, default_value_t=String::from("anon"))]
        owner: String,
        #[arg(long)]
        unlock: bool,
    },
    /// List file locks
    ListLocks,
}

pub async fn run(cmd: LfsCmd) -> Result<()> {
    match cmd {
        LfsCmd::Track { patterns } => {
            let lfs = Lfs::open(std::env::current_dir()?)?;
            for pattern in patterns {
                lfs.add_pattern(&pattern)?;
            }
        }
        LfsCmd::Untrack { patterns } => {
            let lfs = Lfs::open(std::env::current_dir()?)?;
            for pattern in patterns {
                lfs.remove_pattern(&pattern)?;
            }
        }
        LfsCmd::Ls => {
            let lfs = Lfs::open(std::env::current_dir()?)?;
            let cfg = lfs.config()?;
            println!("📋 LFS Tracked Patterns:");
            if cfg.patterns.is_empty() {
                println!("  (no patterns configured)");
            } else {
                for pattern in cfg.patterns {
                    println!("  📁 {}", pattern);
                }
            }
        }
        LfsCmd::Smudge { path } => {
            let lfs = Lfs::open(std::env::current_dir()?)?;
            let rel = path.to_string_lossy().to_string();
            if lfs.smudge_from_pointer(&rel)? {
                println!("✅ Smudged {}", rel);
            } else {
                println!("ℹ️  Not a pointer: {}", rel);
            }
        }
        LfsCmd::Clean { path } => {
            let lfs = Lfs::open(std::env::current_dir()?)?;
            let rel = path.to_string_lossy().to_string();
            if let Some(ptr) = lfs.clean_to_pointer(&rel)? {
                println!(
                    "✅ Cleaned {}; oid={} size={} chunks={}",
                    rel,
                    ptr.oid,
                    ptr.size,
                    ptr.chunks.len()
                );
            } else {
                println!("ℹ️  Not tracked: {}", rel);
            }
        }
        LfsCmd::Config { remote, chunk_size, migration_threshold, list } => {
            let lfs = Lfs::open(std::env::current_dir()?)?;
            
            if list {
                let cfg = lfs.config()?;
                println!("📋 LFS Configuration:");
                println!("  Remote: {:?}", cfg.remote.unwrap_or_else(|| "Not set".to_string()));
                println!("  Chunk size: {} bytes", cfg.chunk_size);
                println!("  Migration threshold: {} bytes", cfg.migration_threshold);
                println!("  Upload enabled: {}", cfg.upload_enabled);
                println!("  Download enabled: {}", cfg.download_enabled);
                return Ok(());
            }
            
            if let Some(r) = remote {
                lfs.set_remote(&r)?;
            }
            if let Some(c) = chunk_size {
                lfs.set_chunk_size(c)?;
            }
            if let Some(t) = migration_threshold {
                let threshold = parse_size(&t)?;
                lfs.set_migration_threshold(threshold)?;
            }
        }
        LfsCmd::Migrate { min_size, dry_run, directory } => {
            let lfs = Lfs::open(std::env::current_dir()?)?;
            let threshold = parse_size(&min_size)?;
            
            // Update migration threshold
            let mut cfg = lfs.config()?;
            cfg.migration_threshold = threshold;
            lfs.write_config(&cfg)?;
            
            if let Some(dir) = directory {
                println!("🔄 Migrating directory: {}", dir.display());
                if !dry_run {
                    let migrated = lfs.migrate_directory(&dir)?;
                    println!("✅ Migrated {} files to LFS", migrated.len());
                    for file in migrated {
                        println!("  📁 {}", file);
                    }
                } else {
                    println!("🔍 Dry run - would migrate files larger than {} bytes", threshold);
                }
            } else {
                println!("🔄 Migrating current directory...");
                if !dry_run {
                    let migrated = lfs.migrate_directory(&std::env::current_dir()?)?;
                    println!("✅ Migrated {} files to LFS", migrated.len());
                    for file in migrated {
                        println!("  📁 {}", file);
                    }
                } else {
                    println!("🔍 Dry run - would migrate files larger than {} bytes", threshold);
                }
            }
        }
        LfsCmd::Status => {
            let lfs = Lfs::open(std::env::current_dir()?)?;
            let stats = lfs.get_stats()?;
            let cfg = lfs.config()?;
            
            println!("📊 LFS Status:");
            println!("  Total files: {}", stats.total_files);
            println!("  Total size: {} bytes", stats.total_size);
            println!("  Tracked patterns: {}", stats.tracked_patterns);
            println!("  Remote files: {}", stats.remote_files);
            println!("  Local only: {}", stats.local_only_files);
            println!("  Remote server: {:?}", cfg.remote.unwrap_or_else(|| "Not configured".to_string()));
        }
        LfsCmd::Sync => {
            let lfs = Lfs::open(std::env::current_dir()?)?;
            lfs.sync_with_server()?;
        }
        LfsCmd::Push { path } => {
            push(path).await?;
        }
        LfsCmd::Pull { oid, out } => {
            pull(oid, out).await?;
        }
        LfsCmd::Lock {
            path,
            owner,
            unlock,
        } => {
            lock_cmd(path, owner, unlock).await?;
        }
        LfsCmd::ListLocks => {
            list_locks().await?;
        }
    }
    Ok(())
}

// Helper function to parse size strings like "10MB", "1GB", etc.
fn parse_size(size_str: &str) -> Result<u64> {
    let size_str = size_str.to_uppercase();
    let (number_part, unit_part) = if size_str.ends_with("GB") {
        (size_str.trim_end_matches("GB"), 1024 * 1024 * 1024)
    } else if size_str.ends_with("MB") {
        (size_str.trim_end_matches("MB"), 1024 * 1024)
    } else if size_str.ends_with("KB") {
        (size_str.trim_end_matches("KB"), 1024)
    } else if size_str.ends_with("B") {
        (size_str.trim_end_matches("B"), 1)
    } else {
        (size_str.as_str(), 1)
    };
    
    let number: u64 = number_part.parse()?;
    Ok(number * unit_part)
}

async fn push(path: std::path::PathBuf) -> Result<()> {
    let lfs = Lfs::open(std::env::current_dir()?)?;
    let cfg = lfs.config()?;
    let remote = cfg
        .remote
        .clone()
        .ok_or_else(|| anyhow::anyhow!("set remote with `rune lfs config --remote <URL>`"))?;
    let rel = path.to_string_lossy().to_string();
    let s = std::fs::read_to_string(&rel).unwrap_or_default();
    if !s.starts_with("version https://rune-lfs/v1") {
        anyhow::bail!(
            "{} is not a pointer. Run `rune lfs clean {}` first.",
            rel,
            rel
        );
    }
    let oid = s
        .lines()
        .find(|l| l.starts_with("oid "))
        .unwrap()
        .trim_start_matches("oid ")
        .to_string();
    let dir = lfs
        .root
        .join(".rune/lfs/objects")
        .join(&oid[0..2])
        .join(&oid[2..4])
        .join(&oid);
    let pj = std::fs::read(dir.join("pointer.json"))?;
    let ptr: Pointer = serde_json::from_slice(&pj)?;
    // Ask server which chunks it already has (resumable uploads)
    let client = reqwest::Client::new();
    let missing: Vec<String> = client
        .post(format!("{}/lfs/has", remote))
        .json(&serde_json::json!({"oid": &oid, "chunks": ptr.chunks}))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    // Always ensure pointer.json exists remotely
    client
        .post(format!("{}/lfs/upload", remote))
        .json(&serde_json::json!({"oid": &oid, "chunk": "pointer.json", "data": pj}))
        .send()
        .await?
        .error_for_status()?;
    for cid in &missing {
        let data = std::fs::read(dir.join(&cid))?;
        client
            .post(format!("{}/lfs/upload", remote))
            .json(&serde_json::json!({"oid": &oid, "chunk": cid, "data": data}))
            .send()
            .await?
            .error_for_status()?;
    }
    println!("pushed {}; missing uploaded: {}", oid, missing.len());
    Ok(())
}
async fn pull(oid: String, out: std::path::PathBuf) -> Result<()> {
    let lfs = Lfs::open(std::env::current_dir()?)?;
    let cfg = lfs.config()?;
    let remote = cfg
        .remote
        .clone()
        .ok_or_else(|| anyhow::anyhow!("set remote with `rune lfs config --remote <URL>`"))?;
    let client = reqwest::Client::new();
    let pj: Vec<u8> = client
        .post(format!("{}/lfs/download", remote))
        .json(&serde_json::json!({"oid": &oid, "chunk": "pointer.json"}))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let ptr: Pointer = serde_json::from_slice(&pj)?;
    let mut outbuf = Vec::with_capacity(ptr.size as usize);
    for cid in ptr.chunks {
        let part: Vec<u8> = client
            .post(format!("{}/lfs/download", remote))
            .json(&serde_json::json!({"oid": &oid, "chunk": cid}))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        outbuf.extend_from_slice(&part);
    }
    if let Some(pp) = out.parent() {
        std::fs::create_dir_all(pp)?;
    }
    std::fs::write(&out, &outbuf)?;
    println!("pulled {} -> {}", oid, out.display());
    Ok(())
}
async fn list_locks() -> Result<()> {
    let url = std::env::var("RUNE_SHRINE").unwrap_or_else(|_| "http://127.0.0.1:7420".into());
    let v: serde_json::Value = reqwest::get(format!("{}/locks/list", url))
        .await?
        .json()
        .await?;
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}
async fn lock_cmd(path: String, owner: String, unlock: bool) -> Result<()> {
    let url = std::env::var("RUNE_SHRINE").unwrap_or_else(|_| "http://127.0.0.1:7420".into());
    let c = reqwest::Client::new();
    let route = if unlock { "unlock" } else { "lock" };
    c.post(format!("{}/locks/{}", url, route))
        .json(&serde_json::json!({"path": path, "owner": owner}))
        .send()
        .await?
        .error_for_status()?;
    println!("{} {}", if unlock { "unlocked" } else { "locked" }, path);
    Ok(())
}
