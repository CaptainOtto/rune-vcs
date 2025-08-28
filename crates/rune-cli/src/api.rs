use anyhow::Result;
use axum::{
    extract::Path,
    response::Html,
    routing::{get, post},
    Json, Router,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tower_http::services::ServeDir;
use std::net::SocketAddr;

#[derive(Serialize, Deserialize)]
struct CommitReq {
    message: String,
    name: Option<String>,
    email: Option<String>,
}
#[derive(Serialize, Deserialize)]
struct StageReq {
    paths: Vec<String>,
}
#[derive(Serialize, Deserialize)]
struct BranchCreate {
    name: String,
}
#[derive(Serialize, Deserialize)]
struct CheckoutReq {
    name: String,
}
#[derive(Serialize, Deserialize)]
struct LfsTrackReq {
    patterns: Vec<String>,
}
#[derive(Serialize, Deserialize)]
struct LfsCleanReq {
    path: String,
}
#[derive(Serialize, Deserialize)]
struct LfsSmudgeReq {
    path: String,
}
#[derive(Serialize, Deserialize)]
struct LfsPushReq {
    path: String,
}
#[derive(Serialize, Deserialize)]
struct LfsPullReq {
    oid: String,
    out: String,
}
#[derive(Serialize, Deserialize)]
struct LockReq {
    path: String,
    owner: String,
}
#[derive(Serialize, Deserialize)]
struct UnlockReq {
    path: String,
    owner: String,
}

pub async fn serve_api(addr: SocketAddr) -> Result<()> {
    let app = Router::new()
        // core
        .route("/v1/status", get(status))
        .route("/v1/log", get(log))
        .route("/v1/commit", post(commit))
        .route("/v1/stage", post(stage))
        .route("/v1/branches", get(branches))
        .route("/v1/branch", post(branch_create))
        .route("/v1/checkout", post(checkout))
        // visual/exploration
        .route("/v1/tree", get(tree))
        .route("/v1/files", get(files))
        .route("/v1/show/:commit", get(show_commit))
        // React app endpoints
        .route("/v1/repository", get(repository_info))
        .route("/v1/changes", get(changes))
        .route("/v1/history", get(history))
        .route("/v1/file-tree", get(file_tree))
        // lfs
        .route("/v1/lfs/track", post(lfs_track))
        .route("/v1/lfs/clean", post(lfs_clean))
        .route("/v1/lfs/smudge", post(lfs_smudge))
        .route("/v1/lfs/push", post(lfs_push))
        .route("/v1/lfs/pull", post(lfs_pull))
        // locks via shrine
        .route("/v1/locks", get(locks_list))
        .route("/v1/lock", post(lock))
        .route("/v1/unlock", post(unlock))
        // web ui - serve React app
        .route("/", get(serve_index))
        .nest_service("/assets", ServeDir::new("web/assets"));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

async fn status() -> Json<serde_json::Value> {
    let s = rune_store::Store::discover(std::env::current_dir().unwrap()).unwrap();
    let idx = s.read_index().unwrap();
    Json(
        serde_json::json!({ "branch": s.head_ref(), "staged": idx.entries.keys().collect::<Vec<_>>() }),
    )
}
async fn log() -> Json<Vec<rune_core::Commit>> {
    let s = rune_store::Store::discover(std::env::current_dir().unwrap()).unwrap();
    Json(s.log())
}
async fn commit(Json(req): Json<CommitReq>) -> Json<serde_json::Value> {
    let s = rune_store::Store::discover(std::env::current_dir().unwrap()).unwrap();
    let author = rune_core::Author {
        name: req.name.unwrap_or(whoami::realname()),
        email: req
            .email
            .unwrap_or(format!("{}@localhost", whoami::username())),
    };
    let c = s.commit(&req.message, author).unwrap();
    Json(serde_json::json!({"id": c.id, "message": c.message}))
}
async fn stage(Json(req): Json<StageReq>) -> Json<serde_json::Value> {
    let s = rune_store::Store::discover(std::env::current_dir().unwrap()).unwrap();
    for p in req.paths {
        s.stage_file(&p).unwrap();
    }
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
                out.push(
                    serde_json::json!({"name": e.path().file_name().unwrap().to_string_lossy()}),
                );
            }
        }
    }
    Json(out)
}
async fn branch_create(Json(req): Json<BranchCreate>) -> Json<serde_json::Value> {
    let s = rune_store::Store::discover(std::env::current_dir().unwrap()).unwrap();
    std::fs::create_dir_all(s.rune_dir.join("refs/heads")).unwrap();
    std::fs::write(s.rune_dir.join("refs/heads").join(&req.name), b"").unwrap();
    Json(serde_json::json!({"created": req.name}))
}
async fn checkout(Json(req): Json<CheckoutReq>) -> Json<serde_json::Value> {
    let s = rune_store::Store::discover(std::env::current_dir().unwrap()).unwrap();
    let r = format!("refs/heads/{}", req.name);
    if s.read_ref(&r).is_none() && !s.rune_dir.join(&r).exists() {
        return Json(serde_json::json!({"error": "branch not found"}));
    }
    s.set_head(&r).unwrap();
    Json(serde_json::json!({"switched": req.name}))
}

async fn lfs_track(Json(req): Json<LfsTrackReq>) -> Json<serde_json::Value> {
    let l = rune_lfs::Lfs::open(std::env::current_dir().unwrap()).unwrap();
    let mut cfg = l.config().unwrap();
    for p in req.patterns {
        if !cfg.patterns.contains(&p) {
            cfg.patterns.push(p);
        }
    }
    l.write_config(&cfg).unwrap();
    Json(serde_json::json!({"tracked": cfg.patterns}))
}
async fn lfs_clean(Json(req): Json<LfsCleanReq>) -> Json<serde_json::Value> {
    let l = rune_lfs::Lfs::open(std::env::current_dir().unwrap()).unwrap();
    match l.clean_to_pointer(&req.path).unwrap() {
        Some(ptr) => {
            Json(serde_json::json!({"oid": ptr.oid, "size": ptr.size, "chunks": ptr.chunks.len()}))
        }
        None => Json(serde_json::json!({"error":"not tracked"})),
    }
}
async fn lfs_smudge(Json(req): Json<LfsSmudgeReq>) -> Json<serde_json::Value> {
    let l = rune_lfs::Lfs::open(std::env::current_dir().unwrap()).unwrap();
    match l.smudge_from_pointer(&req.path).unwrap() {
        true => Json(serde_json::json!({"ok": true})),
        false => Json(serde_json::json!({"error":"not a pointer"})),
    }
}
async fn lfs_push(Json(req): Json<LfsPushReq>) -> Json<serde_json::Value> {
    use rune_lfs::Pointer;
    use serde_json::json;
    let l = rune_lfs::Lfs::open(std::env::current_dir().unwrap()).unwrap();
    let cfg = l.config().unwrap();
    let remote = cfg
        .remote
        .clone()
        .unwrap_or_else(|| "http://127.0.0.1:7420".into());
    let s = std::fs::read_to_string(&req.path).unwrap_or_default();
    if !s.starts_with("version https://rune-lfs/v1") {
        return Json(json!({"error":"not a pointer"}));
    }
    let oid = s
        .lines()
        .find(|l| l.starts_with("oid "))
        .unwrap()
        .trim_start_matches("oid ")
        .to_string();
    let dir = l
        .root
        .join(".rune/lfs/objects")
        .join(&oid[0..2])
        .join(&oid[2..4])
        .join(&oid);
    let pj = std::fs::read(dir.join("pointer.json")).unwrap();
    let ptr: Pointer = serde_json::from_slice(&pj).unwrap();
    let client = reqwest::Client::new();
    let missing: Vec<String> = client
        .post(format!("{}/lfs/has", remote))
        .json(&json!({"oid": &oid, "chunks": ptr.chunks}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    client
        .post(format!("{}/lfs/upload", remote))
        .json(&json!({"oid": &oid, "chunk": "pointer.json", "data": pj}))
        .send()
        .await
        .unwrap();
    let mut uploaded = 0usize;
    for cid in missing {
        let data = std::fs::read(dir.join(&cid)).unwrap();
        client
            .post(format!("{}/lfs/upload", cfg.remote.as_ref().unwrap()))
            .json(&json!({"oid": &oid, "chunk": cid, "data": data}))
            .send()
            .await
            .unwrap();
        uploaded += 1;
    }
    Json(json!({"uploaded": uploaded}))
}
async fn lfs_pull(Json(req): Json<LfsPullReq>) -> Json<serde_json::Value> {
    use rune_lfs::Pointer;
    use serde_json::json;
    let l = rune_lfs::Lfs::open(std::env::current_dir().unwrap()).unwrap();
    let cfg = l.config().unwrap();
    let remote = cfg
        .remote
        .clone()
        .unwrap_or_else(|| "http://127.0.0.1:7420".into());
    let client = reqwest::Client::new();
    let pj: Vec<u8> = client
        .post(format!("{}/lfs/download", remote))
        .json(&json!({"oid": &req.oid, "chunk": "pointer.json"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let ptr: Pointer = serde_json::from_slice(&pj).unwrap();
    let mut outbuf = Vec::with_capacity(ptr.size as usize);
    for cid in &ptr.chunks {
        let part: Vec<u8> = client
            .post(format!("{}/lfs/download", cfg.remote.as_ref().unwrap()))
            .json(&json!({"oid": &req.oid, "chunk": cid}))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        outbuf.extend_from_slice(&part);
    }
    let outp = std::path::PathBuf::from(&req.out);
    if let Some(pp) = outp.parent() {
        std::fs::create_dir_all(pp).unwrap();
    }
    std::fs::write(outp, outbuf).unwrap();
    Json(json!({"ok": true}))
}

async fn locks_list() -> Json<serde_json::Value> {
    let url = std::env::var("RUNE_SHRINE").unwrap_or_else(|_| "http://127.0.0.1:7420".into());
    let v: serde_json::Value = reqwest::get(format!("{}/locks/list", url))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    Json(v)
}
async fn lock(Json(req): Json<LockReq>) -> Json<serde_json::Value> {
    let url = std::env::var("RUNE_SHRINE").unwrap_or_else(|_| "http://127.0.0.1:7420".into());
    let c = reqwest::Client::new();
    c.post(format!("{}/locks/lock", url))
        .json(&serde_json::json!({"path": req.path, "owner": req.owner}))
        .send()
        .await
        .unwrap();
    Json(serde_json::json!({"ok":true}))
}
async fn unlock(Json(req): Json<UnlockReq>) -> Json<serde_json::Value> {
    let url = std::env::var("RUNE_SHRINE").unwrap_or_else(|_| "http://127.0.0.1:7420".into());
    let c = reqwest::Client::new();
    c.post(format!("{}/locks/unlock", url))
        .json(&serde_json::json!({"path": req.path, "owner": req.owner}))
        .send()
        .await
        .unwrap();
    Json(serde_json::json!({"ok":true}))
}

// New visual/exploration endpoints
async fn tree() -> Json<serde_json::Value> {
    let s = rune_store::Store::discover(std::env::current_dir().unwrap()).unwrap();
    let mut files = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(&s.root) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            
            if name.starts_with('.') && name != ".gitignore" {
                continue;
            }
            
            files.push(serde_json::json!({
                "name": name,
                "path": path.strip_prefix(&s.root).unwrap_or(&path).to_string_lossy(),
                "type": if path.is_dir() { "directory" } else { "file" },
                "size": if path.is_file() { 
                    std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0) 
                } else { 0 }
            }));
        }
    }
    
    Json(serde_json::json!({
        "files": files,
        "root": s.root.to_string_lossy()
    }))
}

async fn files() -> Json<serde_json::Value> {
    let s = rune_store::Store::discover(std::env::current_dir().unwrap()).unwrap();
    let index = s.read_index().unwrap();
    let log = s.log();
    
    let mut all_files = std::collections::HashSet::new();
    for commit in &log {
        for file in &commit.files {
            all_files.insert(file.clone());
        }
    }
    
    let files: Vec<_> = all_files.into_iter().map(|file| {
        serde_json::json!({
            "path": file,
            "staged": index.entries.contains_key(&file),
            "status": if index.entries.contains_key(&file) { "modified" } else { "committed" }
        })
    }).collect();
    
    Json(serde_json::json!({"files": files}))
}

async fn show_commit(Path(commit_id): Path<String>) -> Json<serde_json::Value> {
    let s = rune_store::Store::discover(std::env::current_dir().unwrap()).unwrap();
    let log = s.log();
    
    if let Some(commit) = log.iter().find(|c| c.id.starts_with(&commit_id)) {
        Json(serde_json::json!({
            "commit": commit,
            "diff": s.diff(Some(&commit.id)).unwrap_or_else(|_| "Unable to generate diff".to_string())
        }))
    } else {
        Json(serde_json::json!({
            "error": "Commit not found"
        }))
    }
}

async fn serve_index() -> Result<Html<String>, StatusCode> {
    match std::fs::read_to_string("web/index.html") {
        Ok(content) => Ok(Html(content)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn repository_info() -> Json<serde_json::Value> {
    let s = rune_store::Store::discover(std::env::current_dir().unwrap()).unwrap();
    Json(serde_json::json!({
        "name": std::env::current_dir().unwrap().file_name().unwrap().to_string_lossy(),
        "branch": s.head_ref(),
        "url": "local",
        "lastSync": serde_json::Value::Null
    }))
}

async fn changes() -> Json<serde_json::Value> {
    let s = rune_store::Store::discover(std::env::current_dir().unwrap()).unwrap();
    let idx = s.read_index().unwrap();
    
    // Get staged files
    let staged_files: Vec<serde_json::Value> = idx.entries.keys()
        .map(|path| serde_json::json!({
            "id": path,
            "path": path,
            "status": "modified",
            "staged": true
        }))
        .collect();
    
    // Mock changelist for now
    let changelists = vec![
        serde_json::json!({
            "id": "default",
            "name": "Default Changelist",
            "description": "Default changelist",
            "files": staged_files
        })
    ];
    
    Json(serde_json::json!({
        "changelists": changelists,
        "unstagedFiles": []
    }))
}

async fn history() -> Json<serde_json::Value> {
    let s = rune_store::Store::discover(std::env::current_dir().unwrap()).unwrap();
    let commits = s.log();
    
    let formatted_commits: Vec<serde_json::Value> = commits.into_iter()
        .map(|commit| serde_json::json!({
            "id": commit.id,
            "message": commit.message,
            "author": {
                "name": commit.author.name,
                "email": commit.author.email
            },
            "date": commit.time,
            "branch": "main",
            "parents": []
        }))
        .collect();
    
    Json(serde_json::json!({ "commits": formatted_commits }))
}

async fn file_tree() -> Json<serde_json::Value> {
    use std::fs;
    use std::path::Path;
    
    fn build_tree_recursive(path: &Path, name: String) -> serde_json::Value {
        if path.is_dir() {
            let mut children = Vec::new();
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    let entry_name = entry.file_name().to_string_lossy().to_string();
                    if !entry_name.starts_with('.') && entry_name != "target" {
                        children.push(build_tree_recursive(&entry.path(), entry_name));
                    }
                }
            }
            serde_json::json!({
                "id": path.to_string_lossy(),
                "name": name,
                "type": "folder",
                "children": children,
                "status": serde_json::Value::Null
            })
        } else {
            serde_json::json!({
                "id": path.to_string_lossy(),
                "name": name,
                "type": "file",
                "children": serde_json::Value::Null,
                "status": serde_json::Value::Null
            })
        }
    }
    
    let current_dir = std::env::current_dir().unwrap();
    let tree = build_tree_recursive(&current_dir, "root".to_string());
    
    Json(tree)
}

async fn web_ui() -> Html<String> {
    Html(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rune VCS Repository</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; background: #1a1a1a; color: #e0e0e0; }
        .container { max-width: 1200px; margin: 0 auto; padding: 20px; }
        .header { background: #2d2d2d; padding: 20px; border-radius: 8px; margin-bottom: 20px; }
        .header h1 { color: #4fc3f7; margin-bottom: 10px; }
        .tabs { display: flex; gap: 10px; margin-bottom: 20px; }
        .tab { padding: 10px 20px; background: #3d3d3d; border: none; color: #e0e0e0; cursor: pointer; border-radius: 4px; }
        .tab.active { background: #4fc3f7; color: #1a1a1a; }
        .content { background: #2d2d2d; padding: 20px; border-radius: 8px; min-height: 400px; }
        .commit { border-bottom: 1px solid #4d4d4d; padding: 15px 0; }
        .commit:last-child { border-bottom: none; }
        .commit-hash { color: #ffb74d; font-family: monospace; }
        .commit-message { color: #e0e0e0; margin: 5px 0; }
        .commit-meta { color: #9e9e9e; font-size: 0.9em; }
        .file-item { padding: 8px; margin: 2px 0; background: #3d3d3d; border-radius: 4px; }
        .file-name { color: #81c784; }
        .file-path { color: #9e9e9e; font-size: 0.9em; }
        .loading { text-align: center; padding: 40px; color: #9e9e9e; }
        .status-info { display: flex; gap: 20px; margin-bottom: 15px; }
        .status-item { background: #3d3d3d; padding: 10px; border-radius: 4px; flex: 1; }
        .status-value { color: #4fc3f7; font-size: 1.2em; font-weight: bold; }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🪄 Rune VCS Repository</h1>
            <div id="status-info" class="status-info">
                <div class="status-item">
                    <div>Current Branch</div>
                    <div id="current-branch" class="status-value">-</div>
                </div>
                <div class="status-item">
                    <div>Staged Files</div>
                    <div id="staged-count" class="status-value">-</div>
                </div>
                <div class="status-item">
                    <div>Total Files</div>
                    <div id="total-files" class="status-value">-</div>
                </div>
            </div>
        </div>
        
        <div class="tabs">
            <button class="tab active" onclick="showTab('commits')">📋 Commits</button>
            <button class="tab" onclick="showTab('files')">📄 Files</button>
            <button class="tab" onclick="showTab('tree')">📁 Tree</button>
        </div>
        
        <div class="content">
            <div id="commits-content">
                <div class="loading">Loading commits...</div>
            </div>
            <div id="files-content" style="display: none;">
                <div class="loading">Loading files...</div>
            </div>
            <div id="tree-content" style="display: none;">
                <div class="loading">Loading file tree...</div>
            </div>
        </div>
    </div>

    <script>
        let currentTab = 'commits';
        
        function showTab(tab) {
            document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
            document.querySelector(`[onclick="showTab('${tab}')"]`).classList.add('active');
            
            document.querySelectorAll('[id$="-content"]').forEach(c => c.style.display = 'none');
            document.getElementById(`${tab}-content`).style.display = 'block';
            
            currentTab = tab;
            loadData();
        }
        
        async function loadStatus() {
            try {
                const response = await fetch('/v1/status');
                const data = await response.json();
                
                document.getElementById('current-branch').textContent = data.branch || 'main';
                document.getElementById('staged-count').textContent = data.staged ? data.staged.length : 0;
            } catch (e) {
                console.error('Failed to load status:', e);
            }
        }
        
        async function loadCommits() {
            try {
                const response = await fetch('/v1/log');
                const commits = await response.json();
                
                const content = document.getElementById('commits-content');
                if (commits.length === 0) {
                    content.innerHTML = '<div class="loading">No commits yet</div>';
                    return;
                }
                
                content.innerHTML = commits.slice().reverse().map(commit => `
                    <div class="commit">
                        <div class="commit-hash">${commit.id.substring(0, 8)}</div>
                        <div class="commit-message">${commit.message}</div>
                        <div class="commit-meta">
                            ${commit.author.name} • ${new Date(commit.time * 1000).toLocaleDateString()}
                            ${commit.files.length ? ` • ${commit.files.length} files` : ''}
                        </div>
                    </div>
                `).join('');
            } catch (e) {
                document.getElementById('commits-content').innerHTML = 
                    '<div class="loading">Failed to load commits</div>';
            }
        }
        
        async function loadFiles() {
            try {
                const response = await fetch('/v1/files');
                const data = await response.json();
                
                document.getElementById('total-files').textContent = data.files.length;
                
                const content = document.getElementById('files-content');
                if (data.files.length === 0) {
                    content.innerHTML = '<div class="loading">No tracked files</div>';
                    return;
                }
                
                content.innerHTML = data.files.map(file => `
                    <div class="file-item">
                        <div class="file-name">📄 ${file.path}</div>
                        <div class="file-path">Status: ${file.status}${file.staged ? ' (staged)' : ''}</div>
                    </div>
                `).join('');
            } catch (e) {
                document.getElementById('files-content').innerHTML = 
                    '<div class="loading">Failed to load files</div>';
            }
        }
        
        async function loadTree() {
            try {
                const response = await fetch('/v1/tree');
                const data = await response.json();
                
                const content = document.getElementById('tree-content');
                if (data.files.length === 0) {
                    content.innerHTML = '<div class="loading">No files found</div>';
                    return;
                }
                
                content.innerHTML = data.files.map(file => `
                    <div class="file-item">
                        <div class="file-name">${file.type === 'directory' ? '📁' : '📄'} ${file.name}</div>
                        <div class="file-path">
                            ${file.path}${file.type === 'file' ? ` • ${file.size} bytes` : ''}
                        </div>
                    </div>
                `).join('');
            } catch (e) {
                document.getElementById('tree-content').innerHTML = 
                    '<div class="loading">Failed to load file tree</div>';
            }
        }
        
        function loadData() {
            switch (currentTab) {
                case 'commits': loadCommits(); break;
                case 'files': loadFiles(); break;
                case 'tree': loadTree(); break;
            }
        }
        
        // Load initial data
        loadStatus();
        loadData();
        
        // Refresh every 30 seconds
        setInterval(() => {
            loadStatus();
            loadData();
        }, 30000);
    </script>
</body>
</html>
    "#.to_string())
}

pub async fn run_api(addr: String) -> Result<()> {
    let addr: SocketAddr = addr.parse()?;
    println!("🔮 Rune API at http://{}", addr);
    serve_api(addr).await
}
