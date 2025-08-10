
use anyhow::Result;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackEntry { pub path: String, pub size: u64, pub offset: u64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackIndex { pub entries: Vec<PackEntry>, pub checksum: String }

pub fn pack_blobs(blobs: Vec<(String, Vec<u8>)>) -> Result<(Vec<u8>, PackIndex)> {
    let mut out = Vec::new(); let mut entries = Vec::new(); let mut off = 0u64;
    for (path, data) in blobs {
        let compressed = zstd::encode_all(&data[..], 3)?; let sz = compressed.len() as u64;
        out.extend_from_slice(&compressed); entries.push(PackEntry { path, size: sz, offset: off }); off += sz;
    }
    let checksum = format!("{}", blake3::hash(&out)); Ok((out, PackIndex { entries, checksum }))
}
