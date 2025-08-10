
use anyhow::Result;
use chrono::Utc;
use rune_core::{Author, Commit};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, io::Write, path::{Path, PathBuf}};
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct Index { pub entries: BTreeMap<String, i64> } // path -> mtime

pub struct Store { pub root: PathBuf, pub rune_dir: PathBuf }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuneConfig{
  #[serde(default)] pub core: CoreCfg,
  #[serde(default)] pub lfs: LfsCfg,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoreCfg{ #[serde(default="def_branch")] pub default_branch: String }
fn def_branch()->String{ "main".into() }
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LfsCfg{ #[serde(default="def_chunk")] pub chunk_size: usize, pub remote: Option<String>, #[serde(default)] pub track: Vec<TrackCfg> }
fn def_chunk()->usize{ 8*1024*1024 }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackCfg{ pub pattern: String }

impl Store {
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let rd = root.join(".rune");
        fs::create_dir_all(rd.join("objects"))?;
        Ok(Self { root, rune_dir: rd })
    }
    pub fn discover(start: impl AsRef<Path>) -> Result<Self> {
        let mut cur = Some(start.as_ref());
        while let Some(d) = cur {
            let rd = d.join(".rune");
            if rd.exists() { return Self::open(d); }
            cur = d.parent();
        }
        anyhow::bail!("not a rune repo (no .rune found)")
    }

    pub fn config_path(&self)->PathBuf{ self.rune_dir.join("config.toml") }
    pub fn config(&self)->RuneConfig{
        let p=self.config_path();
        if let Ok(s)=fs::read_to_string(p){ toml::from_str(&s).unwrap_or_else(|_| RuneConfig{ core: CoreCfg::default(), lfs: LfsCfg::default() }) }
        else { RuneConfig{ core: CoreCfg::default(), lfs: LfsCfg::default() } }
    }
    pub fn write_config(&self, cfg:&RuneConfig)->anyhow::Result<()> { fs::write(self.config_path(), toml::to_string_pretty(cfg)?)?; Ok(()) }

    pub fn head_ref(&self) -> String {
        fs::read_to_string(self.rune_dir.join("HEAD"))
            .ok()
            .and_then(|s| s.strip_prefix("ref: ").map(|x| x.trim().to_string()))
            .unwrap_or_else(|| "refs/heads/main".to_string())
    }
    pub fn set_head(&self, r: &str) -> Result<()> { fs::write(self.rune_dir.join("HEAD"), format!("ref: {}", r))?; Ok(()) }
    pub fn read_ref(&self, r: &str) -> Option<String> { fs::read_to_string(self.rune_dir.join(r)).ok().map(|s| s.trim().to_string()) }
    pub fn write_ref(&self, r: &str, id: &str) -> Result<()> { let p = self.rune_dir.join(r); if let Some(pp) = p.parent() { fs::create_dir_all(pp)?; } fs::write(p, id.as_bytes())?; Ok(()) }

    pub fn read_index(&self) -> Result<Index> {
        let p = self.rune_dir.join("index.json");
        if p.exists() { Ok(serde_json::from_str(&fs::read_to_string(p)?)?) } else { Ok(Index::default()) }
    }
    pub fn write_index(&self, idx: &Index) -> Result<()> { fs::write(self.rune_dir.join("index.json"), serde_json::to_vec_pretty(idx)?)?; Ok(()) }

    pub fn stage_file(&self, rel: &str) -> Result<()> {
        let mut idx = self.read_index()?;
        let meta = fs::metadata(self.root.join(rel))?;
        let mtime = meta.modified()?.elapsed().map(|e| -(e.as_secs() as i64)).unwrap_or(0);
        idx.entries.insert(rel.to_string(), mtime);
        self.write_index(&idx)
    }

    pub fn commit(&self, msg: &str, author: Author) -> Result<Commit> {
        let idx = self.read_index()?; if idx.entries.is_empty() { anyhow::bail!("nothing to commit"); }
        let branch = self.head_ref(); let branch_head = self.read_ref(&branch);
        let files = idx.entries.keys().cloned().collect::<Vec<_>>();
        let id = format!("{:x}", blake3::hash(format!("{}{}{:?}{}", msg, author.email, files, Utc::now().timestamp()).as_bytes()));
        let c = Commit { id: id.clone(), message: msg.to_string(), author, time: Utc::now().timestamp(), parent: branch_head, files, branch: branch.clone() };
        let mut f = fs::OpenOptions::new().create(true).append(true).open(self.rune_dir.join("log.jsonl"))?; writeln!(f, "{}", serde_json::to_string(&c)?)?;
        self.write_ref(&branch, &id)?; self.write_index(&Index::default())?; Ok(c)
    }
    pub fn log(&self) -> Vec<Commit> {
        let p = self.rune_dir.join("log.jsonl");
        if !p.exists() { return vec![]; }
        fs::read_to_string(p).unwrap_or_default().lines().filter_map(|l| serde_json::from_str::<Commit>(l).ok()).collect()
    }
    pub fn create(&self) -> Result<()> {
        if self.rune_dir.exists() { anyhow::bail!(".rune already exists"); }
        fs::create_dir_all(self.rune_dir.join("objects"))?; fs::create_dir_all(self.rune_dir.join("refs/heads"))?;
        fs::write(self.rune_dir.join("refs/heads/main"), b"")?; self.set_head("refs/heads/main")?; self.write_index(&Index::default())?; Ok(())
    }
}
