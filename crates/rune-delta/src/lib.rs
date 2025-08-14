
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
    
    // Only build the map if base is large enough for the chunk size
    if base.len() >= w {
        for i in 0..=base.len().saturating_sub(w){
            map.entry(&base[i..i+w]).or_default().push(i);
        }
    }
    
    let mut i = 0usize;
    let mut ops: Vec<Op> = Vec::new();
    while i < new.len(){
        let end = (i+w).min(new.len());
        if end-i == w && !map.is_empty() {
            let win = &new[i..end];
            if let Some(pos_list) = map.get(win){
                // choose first match for simplicity
                let best_off = pos_list[0];
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_data() {
        let data = b"Hello, World! This is a longer string to test.";
        let patch = make(data, data, 8).unwrap();
        
        assert_eq!(patch.base_hash, patch.new_hash);
        assert_eq!(patch.chunk, 8);
        
        // Should have copy operations for identical data
        assert!(!patch.ops.is_empty());
        
        let result = apply(data, &patch).unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_completely_different_data() {
        let base = b"Hello, World! This is the original text.";
        let new = b"Goodbye, Universe! This is completely different.";
        let patch = make(base, new, 8).unwrap();
        
        assert_ne!(patch.base_hash, patch.new_hash);
        
        let result = apply(base, &patch).unwrap();
        assert_eq!(result, new);
    }

    #[test]
    fn test_append_data() {
        let base = b"Hello, World!";
        let new = b"Hello, World! And some additional text here.";
        let patch = make(base, new, 8).unwrap();
        
        let result = apply(base, &patch).unwrap();
        assert_eq!(result, new);
        
        // Should have at least one copy operation for the common part
        let has_copy = patch.ops.iter().any(|op| matches!(op, Op::Copy { .. }));
        assert!(has_copy, "Should have copy operations for common text");
    }

    #[test]
    fn test_prepend_data() {
        let base = b"World! This is a test.";
        let new = b"Hello, World! This is a test.";
        let patch = make(base, new, 8).unwrap();
        
        let result = apply(base, &patch).unwrap();
        assert_eq!(result, new);
    }

    #[test]
    fn test_empty_base() {
        let base = b"";
        let new = b"Hello, World!";
        let patch = make(base, new, 8).unwrap();
        
        // Should be all insert operations since base is empty
        for op in &patch.ops {
            assert!(matches!(op, Op::Insert { .. }), "All operations should be inserts for empty base");
        }
        
        let result = apply(base, &patch).unwrap();
        assert_eq!(result, new);
    }

    #[test]
    fn test_empty_new() {
        let base = b"Hello, World!";
        let new = b"";
        let patch = make(base, new, 8).unwrap();
        
        let result = apply(base, &patch).unwrap();
        assert_eq!(result, new);
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_byte_change() {
        let base = b"Hello, World!";
        let new = b"Hello, world!"; // Changed 'W' to 'w'
        let patch = make(base, new, 8).unwrap();
        
        let result = apply(base, &patch).unwrap();
        assert_eq!(result, new);
    }

    #[test]
    fn test_chunk_size_handling() {
        let base = b"Hello, World! This is a longer string for testing.";
        let new = b"Hello, World! This is a longer string for testing. Extra text.";
        
        // Test with different chunk sizes
        for chunk_size in [8, 16, 32] {
            let patch = make(base, new, chunk_size).unwrap();
            assert_eq!(patch.chunk, chunk_size);
            
            let result = apply(base, &patch).unwrap();
            assert_eq!(result, new);
        }
    }

    #[test]
    fn test_minimum_chunk_size() {
        let base = b"Hello, World! This is long enough.";
        let new = b"Hello, World! This is long enough. And more.";
        
        // Chunk size less than 8 should be adjusted to 8
        let patch = make(base, new, 4).unwrap();
        assert_eq!(patch.chunk, 8);
        
        let result = apply(base, &patch).unwrap();
        assert_eq!(result, new);
    }

    #[test]
    fn test_hash_validation() {
        let base = b"Hello, World!";
        let new = b"Hello, Universe!";
        let mut patch = make(base, new, 8).unwrap();
        
        // Corrupt the base hash
        patch.base_hash = "invalid_hash".to_string();
        let result = apply(base, &patch);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("base does not match"));
    }

    #[test]
    fn test_result_hash_validation() {
        let base = b"Hello, World!";
        let new = b"Hello, Universe!";
        let mut patch = make(base, new, 8).unwrap();
        
        // Corrupt the new hash
        patch.new_hash = "invalid_hash".to_string();
        let result = apply(base, &patch);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("result hash mismatch"));
    }

    #[test]
    fn test_copy_operation() {
        let copy_op = Op::Copy { offset: 10, len: 5 };
        match copy_op {
            Op::Copy { offset, len } => {
                assert_eq!(offset, 10);
                assert_eq!(len, 5);
            }
            _ => panic!("Expected Copy operation"),
        }
    }

    #[test]
    fn test_insert_operation() {
        let data = vec![1, 2, 3, 4, 5];
        let insert_op = Op::Insert { data: data.clone() };
        match insert_op {
            Op::Insert { data: op_data } => {
                assert_eq!(op_data, data);
            }
            _ => panic!("Expected Insert operation"),
        }
    }

    #[test]
    fn test_patch_serialization() {
        let base = b"Hello, World!";
        let new = b"Hello, Universe!";
        let patch = make(base, new, 8).unwrap();
        
        // Test serialization to JSON
        let serialized = serde_json::to_string(&patch).unwrap();
        assert!(!serialized.is_empty());
        
        // Test deserialization from JSON
        let deserialized: Patch = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.base_hash, patch.base_hash);
        assert_eq!(deserialized.new_hash, patch.new_hash);
        assert_eq!(deserialized.chunk, patch.chunk);
        assert_eq!(deserialized.ops.len(), patch.ops.len());
    }

    #[test]
    fn test_repeated_patterns() {
        let base = b"abcabcabc";
        let new = b"abcabcabcabc";
        let patch = make(base, new, 8).unwrap();
        
        let result = apply(base, &patch).unwrap();
        assert_eq!(result, new);
        
        // Should efficiently handle repeated patterns
        assert!(!patch.ops.is_empty());
    }

    #[test]
    fn test_large_data() {
        let base = vec![b'A'; 1000];
        let mut new = base.clone();
        new.extend_from_slice(b"ADDITIONAL_DATA");
        
        let patch = make(&base, &new, 16).unwrap();
        let result = apply(&base, &patch).unwrap();
        assert_eq!(result, new);
        
        // Should have efficient copy operations for large identical sections
        let has_large_copy = patch.ops.iter().any(|op| {
            matches!(op, Op::Copy { len, .. } if *len > 100)
        });
        assert!(has_large_copy, "Should have large copy operations for big identical sections");
    }

    #[test]
    fn test_debug_formatting() {
        let patch = Patch {
            base_hash: "base123".to_string(),
            new_hash: "new456".to_string(),
            chunk: 8,
            ops: vec![
                Op::Copy { offset: 0, len: 5 },
                Op::Insert { data: vec![1, 2, 3] },
            ],
        };
        
        let debug_str = format!("{:?}", patch);
        assert!(debug_str.contains("Patch"));
        assert!(debug_str.contains("base123"));
        assert!(debug_str.contains("new456"));
    }
}
