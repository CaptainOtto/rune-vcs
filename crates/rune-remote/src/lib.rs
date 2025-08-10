
use anyhow::Result;
use axum::{routing::{get, post}, Router, Json};
use serde::{Serialize,Deserialize};
use std::{net::SocketAddr, fs, path::PathBuf};

#[derive(Clone)] pub struct Shrine { pub root: PathBuf }
#[derive(Serialize,Deserialize)] pub struct LfsUpload { pub oid:String, pub chunk:String, pub data: Vec<u8> }
#[derive(Serialize,Deserialize)] pub struct LfsDownloadReq { pub oid:String, pub chunk:String }
#[derive(Serialize,Deserialize)] pub struct HasReq { pub oid:String, pub chunks: Vec<String> }
#[derive(Serialize,Deserialize)] pub struct LockReq { pub path:String, pub owner:String }

pub async fn serve(shrine: Shrine, addr: SocketAddr) -> Result<()> {
    let app = Router::new()
        .route("/lfs/upload", post(lfs_upload))
        .route("/lfs/download", post(lfs_download))
        .route("/lfs/has", post(lfs_has))
        .route("/locks/list", get(locks_list))
        .route("/locks/lock", post(lock))
        .route("/locks/unlock", post(unlock))
        .with_state(shrine);
    axum::Server::bind(&addr).serve(app.into_make_service()).await?; Ok(())
}
async fn lfs_upload(axum::extract::State(s):axum::extract::State<Shrine>, Json(b):Json<LfsUpload>) -> &'static str {
    let dir = s.root.join(".rune/lfs/objects").join(&b.oid[0..2]).join(&b.oid[2..4]).join(&b.oid); let _=fs::create_dir_all(&dir); let _=fs::write(dir.join(&b.chunk), &b.data); "ok"
}
async fn lfs_download(axum::extract::State(s):axum::extract::State<Shrine>, Json(b):Json<LfsDownloadReq>) -> Json<Vec<u8>> {
    let dir = s.root.join(".rune/lfs/objects").join(&b.oid[0..2]).join(&b.oid[2..4]).join(&b.oid); let data = fs::read(dir.join(&b.chunk)).unwrap_or_default(); Json(data)
}
async fn lfs_has(axum::extract::State(s):axum::extract::State<Shrine>, Json(req):Json<HasReq>) -> Json<Vec<String>> {
    let dir = s.root.join(".rune/lfs/objects").join(&req.oid[0..2]).join(&req.oid[2..4]).join(&req.oid);
    let missing: Vec<String> = req.chunks.into_iter().filter(|c| !dir.join(c).exists()).collect();
    Json(missing)
}
async fn locks_list(axum::extract::State(s):axum::extract::State<Shrine>) -> Json<Vec<serde_json::Value>> {
    let lp = s.root.join(".rune/lfs/locks.json"); let v:Vec<serde_json::Value> = if lp.exists(){ serde_json::from_str(&fs::read_to_string(lp).unwrap_or_default()).unwrap_or_default() } else { vec![] }; Json(v)
}
async fn lock(axum::extract::State(s):axum::extract::State<Shrine>, Json(b):Json<LockReq>) -> &'static str {
    let lp = s.root.join(".rune/lfs/locks.json"); let mut v:Vec<serde_json::Value> = if lp.exists(){ serde_json::from_str(&fs::read_to_string(&lp).unwrap_or_default()).unwrap_or_default() } else { vec![] };
    v.push(serde_json::json!({"path":b.path,"owner":b.owner,"created_at": chrono::Utc::now().timestamp()})); let _=fs::create_dir_all(lp.parent().unwrap()); let _=fs::write(lp, serde_json::to_vec_pretty(&v).unwrap()); "locked"
}
async fn unlock(axum::extract::State(s):axum::extract::State<Shrine>, Json(b):Json<LockReq>) -> &'static str {
    let lp = s.root.join(".rune/lfs/locks.json"); let mut v:Vec<serde_json::Value> = if lp.exists(){ serde_json::from_str(&fs::read_to_string(&lp).unwrap_or_default()).unwrap_or_default() } else { vec![] };
    v.retain(|x| !(x.get("path")==Some(&serde_json::json!(b.path)) && x.get("owner")==Some(&serde_json::json!(b.owner)))); let _=fs::create_dir_all(lp.parent().unwrap()); let _=fs::write(lp, serde_json::to_vec_pretty(&v).unwrap()); "unlocked"
}
