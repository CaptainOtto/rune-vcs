
use anyhow::Result;
use axum::{routing::{get, post}, Router, Json};
use serde::{Serialize, Deserialize};
use std::net::SocketAddr;

#[derive(Serialize,Deserialize)] struct CommitReq{ message:String, name:Option<String>, email:Option<String> }
#[derive(Serialize,Deserialize)] struct StageReq{ paths: Vec<String> }
#[derive(Serialize,Deserialize)] struct BranchCreate{ name:String }
#[derive(Serialize,Deserialize)] struct CheckoutReq{ name:String }
#[derive(Serialize,Deserialize)] struct LfsTrackReq{ patterns: Vec<String> }
#[derive(Serialize,Deserialize)] struct LfsCleanReq{ path:String }
#[derive(Serialize,Deserialize)] struct LfsSmudgeReq{ path:String }
#[derive(Serialize,Deserialize)] struct LfsPushReq{ path:String }
#[derive(Serialize,Deserialize)] struct LfsPullReq{ oid:String, out:String }
#[derive(Serialize,Deserialize)] struct LockReq{ path:String, owner:String }
#[derive(Serialize,Deserialize)] struct UnlockReq{ path:String, owner:String }

pub async fn serve_api(addr:SocketAddr)->Result<()>{
    let app = Router::new()
        // core
        .route("/v1/status", get(status))
        .route("/v1/log", get(log))
        .route("/v1/commit", post(commit))
        .route("/v1/stage", post(stage))
        .route("/v1/branches", get(branches))
        .route("/v1/branch", post(branch_create))
        .route("/v1/checkout", post(checkout))
        // lfs
        .route("/v1/lfs/track", post(lfs_track))
        .route("/v1/lfs/clean", post(lfs_clean))
        .route("/v1/lfs/smudge", post(lfs_smudge))
        .route("/v1/lfs/push", post(lfs_push))
        .route("/v1/lfs/pull", post(lfs_pull))
        // locks via shrine
        .route("/v1/locks", get(locks_list))
        .route("/v1/lock", post(lock))
        .route("/v1/unlock", post(unlock));
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;
    Ok(())
}

async fn status() -> Json<serde_json::Value> {
    let s = rune_store::Store::discover(std::env::current_dir().unwrap()).unwrap();
    let idx = s.read_index().unwrap();
    Json(serde_json::json!({ "branch": s.head_ref(), "staged": idx.entries.keys().collect::<Vec<_>>() }))
}
async fn log() -> Json<Vec<rune_core::Commit>> {
    let s = rune_store::Store::discover(std::env::current_dir().unwrap()).unwrap();
    Json(s.log())
}
async fn commit(Json(req):Json<CommitReq>) -> Json<serde_json::Value> {
    let s = rune_store::Store::discover(std::env::current_dir().unwrap()).unwrap();
    let author = rune_core::Author{
        name: req.name.unwrap_or(whoami::realname()),
        email: req.email.unwrap_or(format!("{}@localhost", whoami::username())),
    };
    let c = s.commit(&req.message, author).unwrap();
    Json(serde_json::json!({"id": c.id, "message": c.message}))
}
async fn stage(Json(req):Json<StageReq>) -> Json<serde_json::Value> {
    let s = rune_store::Store::discover(std::env::current_dir().unwrap()).unwrap();
    for p in req.paths { s.stage_file(&p).unwrap(); }
    Json(serde_json::json!({"ok": true}))
}

async fn branches() -> Json<Vec<serde_json::Value>> {
    let s = rune_store::Store::discover(std::env::current_dir().unwrap()).unwrap();
    let mut out = vec![];
    let dir = s.rune_dir.join("refs/heads");
    if dir.exists() {
        for e in walkdir::WalkDir::new(dir) {
            let e = e.unwrap();
            if e.file_type().is_file() {
                out.push(serde_json::json!({"name": e.path().file_name().unwrap().to_string_lossy()}));
            }
        }
    }
    Json(out)
}
async fn branch_create(Json(req):Json<BranchCreate>) -> Json<serde_json::Value> {
    let s = rune_store::Store::discover(std::env::current_dir().unwrap()).unwrap();
    std::fs::create_dir_all(s.rune_dir.join("refs/heads")).unwrap();
    std::fs::write(s.rune_dir.join("refs/heads").join(&req.name), b"").unwrap();
    Json(serde_json::json!({"created": req.name}))
}
async fn checkout(Json(req):Json<CheckoutReq>) -> Json<serde_json::Value> {
    let s = rune_store::Store::discover(std::env::current_dir().unwrap()).unwrap();
    let r = format!("refs/heads/{}", req.name);
    if s.read_ref(&r).is_none() && !s.rune_dir.join(&r).exists() {
        return Json(serde_json::json!({"error": "branch not found"}));
    }
    s.set_head(&r).unwrap();
    Json(serde_json::json!({"switched": req.name}))
}

async fn lfs_track(Json(req):Json<LfsTrackReq>) -> Json<serde_json::Value> {
    let l = rune_lfs::Lfs::open(std::env::current_dir().unwrap()).unwrap();
    let mut cfg = l.config().unwrap();
    for p in req.patterns { if !cfg.patterns.contains(&p) { cfg.patterns.push(p); } }
    l.write_config(&cfg).unwrap();
    Json(serde_json::json!({"tracked": cfg.patterns}))
}
async fn lfs_clean(Json(req):Json<LfsCleanReq>) -> Json<serde_json::Value> {
    let l = rune_lfs::Lfs::open(std::env::current_dir().unwrap()).unwrap();
    match l.clean_to_pointer(&req.path).unwrap() {
        Some(ptr) => Json(serde_json::json!({"oid": ptr.oid, "size": ptr.size, "chunks": ptr.chunks.len()})),
        None => Json(serde_json::json!({"error":"not tracked"}))
    }
}
async fn lfs_smudge(Json(req):Json<LfsSmudgeReq>) -> Json<serde_json::Value> {
    let l = rune_lfs::Lfs::open(std::env::current_dir().unwrap()).unwrap();
    match l.smudge_from_pointer(&req.path).unwrap() {
        true => Json(serde_json::json!({"ok": true})),
        false => Json(serde_json::json!({"error":"not a pointer"}))
    }
}
async fn lfs_push(Json(req):Json<LfsPushReq>) -> Json<serde_json::Value> {
    use serde_json::json;
    use rune_lfs::Pointer;
    let l = rune_lfs::Lfs::open(std::env::current_dir().unwrap()).unwrap();
    let cfg = l.config().unwrap();
    let remote = cfg.remote.clone().unwrap_or_else(|| "http://127.0.0.1:7420".into());
    let s = std::fs::read_to_string(&req.path).unwrap_or_default();
    if !s.starts_with("version https://rune-lfs/v1"){ return Json(json!({"error":"not a pointer"})); }
    let oid = s.lines().find(|l| l.starts_with("oid ")).unwrap().trim_start_matches("oid ").to_string();
    let dir = l.root.join(".rune/lfs/objects").join(&oid[0..2]).join(&oid[2..4]).join(&oid);
    let pj = std::fs::read(dir.join("pointer.json")).unwrap();
    let ptr: Pointer = serde_json::from_slice(&pj).unwrap();
    let client = reqwest::Client::new();
    let missing: Vec<String> = client.post(format!("{}/lfs/has", remote)).json(&json!({"oid": &oid, "chunks": ptr.chunks})).send().await.unwrap().json().await.unwrap();
    client.post(format!("{}/lfs/upload", remote)).json(&json!({"oid": &oid, "chunk": "pointer.json", "data": pj})).send().await.unwrap();
    let mut uploaded = 0usize;
    for cid in missing { let data = std::fs::read(dir.join(&cid)).unwrap(); client.post(format!("{}/lfs/upload", cfg.remote.as_ref().unwrap())).json(&json!({"oid": &oid, "chunk": cid, "data": data})).send().await.unwrap(); uploaded+=1; }
    Json(json!({"uploaded": uploaded}))
}
async fn lfs_pull(Json(req):Json<LfsPullReq>) -> Json<serde_json::Value> {
    use serde_json::json;
    use rune_lfs::Pointer;
    let l = rune_lfs::Lfs::open(std::env::current_dir().unwrap()).unwrap();
    let cfg = l.config().unwrap();
    let remote = cfg.remote.clone().unwrap_or_else(|| "http://127.0.0.1:7420".into());
    let client = reqwest::Client::new();
    let pj: Vec<u8> = client.post(format!("{}/lfs/download", remote)).json(&json!({"oid": &req.oid, "chunk": "pointer.json"})).send().await.unwrap().json().await.unwrap();
    let ptr: Pointer = serde_json::from_slice(&pj).unwrap();
    let mut outbuf = Vec::with_capacity(ptr.size as usize);
    for cid in &ptr.chunks { let part: Vec<u8> = client.post(format!("{}/lfs/download", cfg.remote.as_ref().unwrap())).json(&json!({"oid": &req.oid, "chunk": cid})).send().await.unwrap().json().await.unwrap(); outbuf.extend_from_slice(&part); }
    let outp = std::path::PathBuf::from(&req.out); if let Some(pp)=outp.parent(){ std::fs::create_dir_all(pp).unwrap(); } std::fs::write(outp, outbuf).unwrap();
    Json(json!({"ok": true}))
}

async fn locks_list() -> Json<serde_json::Value> {
    let url = std::env::var("RUNE_SHRINE").unwrap_or_else(|_| "http://127.0.0.1:7420".into());
    let v: serde_json::Value = reqwest::get(format!("{}/locks/list", url)).await.unwrap().json().await.unwrap();
    Json(v)
}
async fn lock(Json(req):Json<LockReq>) -> Json<serde_json::Value> {
    let url = std::env::var("RUNE_SHRINE").unwrap_or_else(|_| "http://127.0.0.1:7420".into());
    let c = reqwest::Client::new();
    c.post(format!("{}/locks/lock", url)).json(&serde_json::json!({"path": req.path, "owner": req.owner})).send().await.unwrap();
    Json(serde_json::json!({"ok":true}))
}
async fn unlock(Json(req):Json<UnlockReq>) -> Json<serde_json::Value> {
    let url = std::env::var("RUNE_SHRINE").unwrap_or_else(|_| "http://127.0.0.1:7420".into());
    let c = reqwest::Client::new();
    c.post(format!("{}/locks/unlock", url)).json(&serde_json::json!({"path": req.path, "owner": req.owner})).send().await.unwrap();
    Json(serde_json::json!({"ok":true}))
}

pub async fn run_api(addr:String)->Result<()>{
    let addr: SocketAddr = addr.parse()?;
    println!("🔮 Rune API at http://{}", addr);
    axum_main(addr).await
}
