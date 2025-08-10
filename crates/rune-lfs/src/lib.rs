
use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::{fs, path::{Path,PathBuf}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LfsConfig { pub patterns: Vec<String>, pub chunk_size: usize, pub remote: Option<String> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pointer { pub oid: String, pub size: u64, pub chunks: Vec<String> }

pub struct Lfs { pub root: PathBuf, pub dir: PathBuf }
impl Lfs{
  pub fn open(root:impl AsRef<Path>)->Result<Self>{ let root=root.as_ref().to_path_buf(); let d=root.join(".rune").join("lfs"); fs::create_dir_all(d.join("objects"))?; Ok(Self{ root, dir:d }) }
  pub fn config_path(&self)->PathBuf{ self.dir.join("config.json") }
  pub fn config(&self)->Result<LfsConfig>{ if self.config_path().exists(){ Ok(serde_json::from_str(&fs::read_to_string(self.config_path())?)?) } else { Ok(LfsConfig{patterns:vec![], chunk_size:8*1024*1024, remote:None}) } }
  pub fn write_config(&self, cfg:&LfsConfig)->Result<()> { fs::write(self.config_path(), serde_json::to_vec_pretty(cfg)?)?; Ok(()) }
  pub fn is_tracked(&self, path:&str)->Result<bool>{ let cfg=self.config()?; for pat in cfg.patterns { if glob::Pattern::new(&pat).map(|g|g.matches(path)).unwrap_or(false){ return Ok(true); } } Ok(false) }
  fn chunk_dir(&self, oid:&str)->PathBuf{ self.dir.join("objects").join(&oid[0..2]).join(&oid[2..4]).join(oid) }

  pub fn clean_to_pointer(&self, rel:&str)->Result<Option<Pointer>>{
    if !self.is_tracked(rel)? { return Ok(None); }
    let data = fs::read(self.root.join(rel))?; let oid = format!("{}", blake3::hash(&data));
    let chunk_size = self.config()?.chunk_size; let dir = self.chunk_dir(&oid); fs::create_dir_all(&dir)?;
    let mut chunks = Vec::new();
    for (i, part) in data.chunks(chunk_size).enumerate(){ let cid = format!("{}.{:06}", oid, i); fs::write(dir.join(&cid), part)?; chunks.push(cid); }
    let ptr = Pointer{ oid: oid.clone(), size: data.len() as u64, chunks };
    fs::write(self.root.join(rel), format!("version https://rune-lfs/v1
oid {}
size {}", oid, data.len()))?;
    fs::write(dir.join("pointer.json"), serde_json::to_vec_pretty(&ptr)?)?; Ok(Some(ptr))
  }
  pub fn smudge_from_pointer(&self, rel:&str)->Result<bool>{
    let s = fs::read_to_string(self.root.join(rel)).unwrap_or_default(); if !s.starts_with("version https://rune-lfs/v1"){ return Ok(false); }
    let oid = s.lines().find(|l| l.starts_with("oid ")).unwrap().trim_start_matches("oid ").trim().to_string();
    let dir = self.chunk_dir(&oid); let ppath = dir.join("pointer.json"); if !ppath.exists(){ anyhow::bail!("pointer data missing for {}", rel); }
    let ptr: Pointer = serde_json::from_slice(&fs::read(ppath)?)?; let mut out = Vec::with_capacity(ptr.size as usize);
    for cid in ptr.chunks { let part = fs::read(dir.join(cid))?; out.extend_from_slice(&part); } fs::write(self.root.join(rel), out)?; Ok(true)
  }
}
