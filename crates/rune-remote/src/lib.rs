use anyhow::Result;
use axum::{
    routing::{get, post},
    Json, Router,
};
use tokio::net::TcpListener;
use serde::{Deserialize, Serialize};
use std::{fs, net::SocketAddr, path::PathBuf};

pub mod auth;
pub mod sync;

pub use auth::{AuthService, Permission};
pub use sync::{Commit, Branch, FileChange, FileOperation};

#[derive(Clone)]
pub struct Shrine {
    pub root: PathBuf,
}
#[derive(Serialize, Deserialize)]
pub struct LfsUpload {
    pub oid: String,
    pub chunk: String,
    pub data: Vec<u8>,
}
#[derive(Serialize, Deserialize)]
pub struct LfsDownloadReq {
    pub oid: String,
    pub chunk: String,
}
#[derive(Serialize, Deserialize)]
pub struct HasReq {
    pub oid: String,
    pub chunks: Vec<String>,
}
#[derive(Serialize, Deserialize)]
pub struct LockReq {
    pub path: String,
    pub owner: String,
}

pub async fn run_server(shrine: Shrine, addr: SocketAddr) -> Result<()> {
    let app = Router::new()
        // LFS endpoints
        .route("/lfs/upload", post(lfs_upload))
        .route("/lfs/download", post(lfs_download))
        .route("/lfs/has", post(lfs_has))
        // Lock endpoints
        .route("/locks/list", get(locks_list))
        .route("/locks/lock", post(lock))
        .route("/locks/unlock", post(unlock))
        // Repository sync endpoints
        .route("/sync/info", get(sync::get_repository_info))
        .route("/sync/push", post(sync::push_commits))
        .route("/sync/pull", post(sync::pull_commits))
        .route("/sync/branches", get(sync::get_branches_endpoint))
        .route("/sync/commits/:since", get(sync::get_commits_since))
        .route("/sync/repository/:remote", post(sync::sync_repository))
        .with_state(shrine);
    let listener = TcpListener::bind(addr).await?;
    axum::serve::serve(listener, app.into_make_service()).await?;
    Ok(())
}
async fn lfs_upload(
    axum::extract::State(s): axum::extract::State<Shrine>,
    Json(b): Json<LfsUpload>,
) -> &'static str {
    let dir = s
        .root
        .join(".rune/lfs/objects")
        .join(&b.oid[0..2])
        .join(&b.oid[2..4])
        .join(&b.oid);
    let _ = fs::create_dir_all(&dir);
    let _ = fs::write(dir.join(&b.chunk), &b.data);
    "ok"
}
async fn lfs_download(
    axum::extract::State(s): axum::extract::State<Shrine>,
    Json(b): Json<LfsDownloadReq>,
) -> Json<Vec<u8>> {
    let dir = s
        .root
        .join(".rune/lfs/objects")
        .join(&b.oid[0..2])
        .join(&b.oid[2..4])
        .join(&b.oid);
    let data = fs::read(dir.join(&b.chunk)).unwrap_or_default();
    Json(data)
}
async fn lfs_has(
    axum::extract::State(s): axum::extract::State<Shrine>,
    Json(req): Json<HasReq>,
) -> Json<Vec<String>> {
    let dir = s
        .root
        .join(".rune/lfs/objects")
        .join(&req.oid[0..2])
        .join(&req.oid[2..4])
        .join(&req.oid);
    let missing: Vec<String> = req
        .chunks
        .into_iter()
        .filter(|c| !dir.join(c).exists())
        .collect();
    Json(missing)
}
async fn locks_list(
    axum::extract::State(s): axum::extract::State<Shrine>,
) -> Json<Vec<serde_json::Value>> {
    let lp = s.root.join(".rune/lfs/locks.json");
    let v: Vec<serde_json::Value> = if lp.exists() {
        serde_json::from_str(&fs::read_to_string(lp).unwrap_or_default()).unwrap_or_default()
    } else {
        vec![]
    };
    Json(v)
}
async fn lock(
    axum::extract::State(s): axum::extract::State<Shrine>,
    Json(b): Json<LockReq>,
) -> &'static str {
    let lp = s.root.join(".rune/lfs/locks.json");
    let mut v: Vec<serde_json::Value> = if lp.exists() {
        serde_json::from_str(&fs::read_to_string(&lp).unwrap_or_default()).unwrap_or_default()
    } else {
        vec![]
    };
    v.push(serde_json::json!({"path":b.path,"owner":b.owner,"created_at": chrono::Utc::now().timestamp()}));
    let _ = fs::create_dir_all(lp.parent().unwrap());
    let _ = fs::write(lp, serde_json::to_vec_pretty(&v).unwrap());
    "locked"
}
async fn unlock(
    axum::extract::State(s): axum::extract::State<Shrine>,
    Json(b): Json<LockReq>,
) -> &'static str {
    let lp = s.root.join(".rune/lfs/locks.json");
    let mut v: Vec<serde_json::Value> = if lp.exists() {
        serde_json::from_str(&fs::read_to_string(&lp).unwrap_or_default()).unwrap_or_default()
    } else {
        vec![]
    };
    v.retain(|x| {
        !(x.get("path") == Some(&serde_json::json!(b.path))
            && x.get("owner") == Some(&serde_json::json!(b.owner)))
    });
    let _ = fs::create_dir_all(lp.parent().unwrap());
    let _ = fs::write(lp, serde_json::to_vec_pretty(&v).unwrap());
    "unlocked"
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_shrine_creation() {
        let temp_dir = TempDir::new().unwrap();
        let shrine = Shrine {
            root: temp_dir.path().to_path_buf(),
        };
        assert_eq!(shrine.root, temp_dir.path());
    }

    #[test]
    fn test_lfs_upload_struct() {
        let upload = LfsUpload {
            oid: "test-oid".to_string(),
            chunk: "chunk1".to_string(),
            data: vec![1, 2, 3, 4],
        };
        assert_eq!(upload.oid, "test-oid");
        assert_eq!(upload.chunk, "chunk1");
        assert_eq!(upload.data, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_lock_req_struct() {
        let lock_req = LockReq {
            path: "/test/path".to_string(),
            owner: "test-owner".to_string(),
        };
        assert_eq!(lock_req.path, "/test/path");
        assert_eq!(lock_req.owner, "test-owner");
    }
}
