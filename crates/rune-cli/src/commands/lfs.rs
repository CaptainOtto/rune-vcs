use anyhow::Result;
use clap::Subcommand;
use rune_lfs::{Lfs, Pointer};

#[derive(Subcommand, Debug)]
pub enum LfsCmd {
    Track {
        patterns: Vec<String>,
    },
    Smudge {
        path: std::path::PathBuf,
    },
    Clean {
        path: std::path::PathBuf,
    },
    Config {
        #[arg(long)]
        remote: Option<String>,
        #[arg(long)]
        chunk_size: Option<usize>,
    },
    Push {
        path: std::path::PathBuf,
    },
    Pull {
        oid: String,
        out: std::path::PathBuf,
    },
    Lock {
        #[arg(long)]
        path: String,
        #[arg(long, default_value_t=String::from("anon"))]
        owner: String,
        #[arg(long)]
        unlock: bool,
    },
    ListLocks,
}

pub async fn run(cmd: LfsCmd) -> Result<()> {
    match cmd {
        LfsCmd::Track { patterns } => {
            let lfs = Lfs::open(std::env::current_dir()?)?;
            let mut cfg = lfs.config()?;
            for p in patterns {
                if !cfg.patterns.contains(&p) {
                    cfg.patterns.push(p);
                }
            }
            lfs.write_config(&cfg)?;
            println!("tracked: {}", cfg.patterns.join(", "));
        }
        LfsCmd::Smudge { path } => {
            let lfs = Lfs::open(std::env::current_dir()?)?;
            let rel = path.to_string_lossy().to_string();
            if lfs.smudge_from_pointer(&rel)? {
                println!("smudged {}", rel);
            } else {
                println!("not a pointer: {}", rel);
            }
        }
        LfsCmd::Clean { path } => {
            let lfs = Lfs::open(std::env::current_dir()?)?;
            let rel = path.to_string_lossy().to_string();
            if let Some(ptr) = lfs.clean_to_pointer(&rel)? {
                println!(
                    "cleaned {}; oid={} size={} chunks={} ",
                    rel,
                    ptr.oid,
                    ptr.size,
                    ptr.chunks.len()
                );
            } else {
                println!("not tracked: {}", rel);
            }
        }
        LfsCmd::Config { remote, chunk_size } => {
            let lfs = Lfs::open(std::env::current_dir()?)?;
            let mut cfg = lfs.config()?;
            if let Some(r) = remote {
                cfg.remote = Some(r);
            }
            if let Some(c) = chunk_size {
                cfg.chunk_size = c;
            }
            lfs.write_config(&cfg)?;
            println!(
                "config: remote={:?} chunk_size={}B",
                cfg.remote, cfg.chunk_size
            );
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
