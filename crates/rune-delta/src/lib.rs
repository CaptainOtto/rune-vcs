
use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Op { Copy{ offset: usize, len: usize }, Insert{ data: Vec<u8> } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch { pub base_hash: String, pub new_hash: String, pub chunk: usize, pub ops: Vec<Op> }

pub fn make(base:&[u8], new:&[u8], chunk:usize)->Result<Patch>{
    let base_hash = format!("{}", blake3::hash(base));
    let new_hash = format!("{}", blake3::hash(new));
    let mut map: HashMap<&[u8], Vec<usize>> = HashMap::new();
    let w = if chunk<8 {8} else {chunk};
    for i in 0..=base.len().saturating_sub(w){
        map.entry(&base[i..i+w]).or_default().push(i);
    }
    let mut i = 0usize;
    let mut ops: Vec<Op> = Vec::new();
    while i < new.len(){
        let end = (i+w).min(new.len());
        if end-i == w {
            let win = &new[i..end];
            if let Some(pos_list) = map.get(win){
                // choose first match for simplicity
                let mut best_off = pos_list[0];
                let mut match_len = w;
                // extend match forward
                while best_off+match_len < base.len() && i+match_len < new.len() && base[best_off+match_len]==new[i+match_len] { match_len+=1; }
                ops.push(Op::Copy{ offset: best_off, len: match_len });
                i += match_len;
                continue;
            }
        }
        // no match: emit one byte insert and continue (could batch more, but keep simple)
        let b = new[i];
        if let Some(Op::Insert{ data }) = ops.last_mut() {
            data.push(b);
        } else {
            ops.push(Op::Insert{ data: vec![b] });
        }
        i += 1;
    }
    Ok(Patch{ base_hash, new_hash, chunk:w, ops })
}

pub fn apply(base:&[u8], patch:&Patch)->Result<Vec<u8>>{
    let mut out = Vec::new();
    if format!("{}", blake3::hash(base)) != patch.base_hash { anyhow::bail!("base does not match patch base_hash"); }
    for op in &patch.ops {
        match op {
            Op::Copy{offset,len} => { out.extend_from_slice(&base[*offset..*offset+*len]); },
            Op::Insert{data} => out.extend_from_slice(data),
        }
    }
    if format!("{}", blake3::hash(&out)) != patch.new_hash { anyhow::bail!("result hash mismatch"); }
    Ok(out)
}
