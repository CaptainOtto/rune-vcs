
use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Op { Copy{ offset: usize, len: usize }, Insert{ data: Vec<u8> } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRename {
    pub old_path: String,
    pub new_path: String,
    pub similarity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCopy {
    pub source_path: String,
    pub dest_path: String,
    pub similarity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffMode {
    Character,
    Word,
    Line,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffOptions {
    pub mode: DiffMode,
    pub detect_renames: bool,
    pub detect_copies: bool,
    pub similarity_threshold: f64,
    pub context_lines: usize,
}

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

// Calculate similarity between two byte arrays using Jaccard similarity
pub fn calculate_similarity(a: &[u8], b: &[u8]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    
    let mut set_a = std::collections::HashSet::new();
    let mut set_b = std::collections::HashSet::new();
    
    // Use 3-grams for better similarity detection
    let window_size = 3;
    
    for window in a.windows(window_size) {
        set_a.insert(window);
    }
    
    for window in b.windows(window_size) {
        set_b.insert(window);
    }
    
    let intersection_size = set_a.intersection(&set_b).count();
    let union_size = set_a.union(&set_b).count();
    
    if union_size == 0 {
        0.0
    } else {
        intersection_size as f64 / union_size as f64
    }
}

// Detect file renames by comparing content similarity
pub fn detect_renames(
    deleted_files: &HashMap<String, Vec<u8>>,
    added_files: &HashMap<String, Vec<u8>>,
    threshold: f64,
) -> Vec<FileRename> {
    let mut renames = Vec::new();
    let mut used_targets = std::collections::HashSet::new();
    
    for (deleted_path, deleted_content) in deleted_files {
        let mut best_match = None;
        let mut best_similarity = threshold;
        
        for (added_path, added_content) in added_files {
            if used_targets.contains(added_path) {
                continue;
            }
            
            let similarity = calculate_similarity(deleted_content, added_content);
            if similarity > best_similarity {
                best_similarity = similarity;
                best_match = Some(added_path.clone());
            }
        }
        
        if let Some(new_path) = best_match {
            used_targets.insert(new_path.clone());
            renames.push(FileRename {
                old_path: deleted_path.clone(),
                new_path,
                similarity: best_similarity,
            });
        }
    }
    
    renames
}

// Detect file copies
pub fn detect_copies(
    existing_files: &HashMap<String, Vec<u8>>,
    added_files: &HashMap<String, Vec<u8>>,
    threshold: f64,
) -> Vec<FileCopy> {
    let mut copies = Vec::new();
    
    for (added_path, added_content) in added_files {
        for (existing_path, existing_content) in existing_files {
            let similarity = calculate_similarity(existing_content, added_content);
            if similarity >= threshold {
                copies.push(FileCopy {
                    source_path: existing_path.clone(),
                    dest_path: added_path.clone(),
                    similarity,
                });
            }
        }
    }
    
    copies
}

// Word-level diff for better text comparison
pub fn word_diff(old_text: &str, new_text: &str) -> Vec<(String, String)> {
    let old_words: Vec<&str> = old_text.split_whitespace().collect();
    let new_words: Vec<&str> = new_text.split_whitespace().collect();
    
    let mut result = Vec::new();
    let mut i = 0;
    let mut j = 0;
    
    while i < old_words.len() || j < new_words.len() {
        if i < old_words.len() && j < new_words.len() && old_words[i] == new_words[j] {
            // Words match
            result.push(("equal".to_string(), old_words[i].to_string()));
            i += 1;
            j += 1;
        } else if i < old_words.len() && (j >= new_words.len() || old_words[i] != new_words[j]) {
            // Word deleted
            result.push(("delete".to_string(), old_words[i].to_string()));
            i += 1;
        } else if j < new_words.len() {
            // Word added
            result.push(("insert".to_string(), new_words[j].to_string()));
            j += 1;
        }
    }
    
    result
}

// Enhanced diff with configurable options
pub fn enhanced_diff(
    old_content: &[u8],
    new_content: &[u8],
    options: &DiffOptions,
) -> Result<String> {
    let old_text = String::from_utf8_lossy(old_content);
    let new_text = String::from_utf8_lossy(new_content);
    
    match options.mode {
        DiffMode::Word => {
            let word_changes = word_diff(&old_text, &new_text);
            let mut output = String::new();
            
            for (change_type, word) in word_changes {
                match change_type.as_str() {
                    "equal" => output.push_str(&format!(" {}", word)),
                    "delete" => output.push_str(&format!(" [-{}]", word)),
                    "insert" => output.push_str(&format!(" [+{}]", word)),
                    _ => {}
                }
            }
            
            Ok(output)
        }
        DiffMode::Line => {
            let old_lines: Vec<&str> = old_text.lines().collect();
            let new_lines: Vec<&str> = new_text.lines().collect();
            
            let mut output = String::new();
            let mut i = 0;
            let mut j = 0;
            
            while i < old_lines.len() || j < new_lines.len() {
                if i < old_lines.len() && j < new_lines.len() && old_lines[i] == new_lines[j] {
                    if options.context_lines > 0 {
                        output.push_str(&format!("  {}\n", old_lines[i]));
                    }
                    i += 1;
                    j += 1;
                } else if i < old_lines.len() && (j >= new_lines.len() || old_lines[i] != new_lines[j]) {
                    output.push_str(&format!("- {}\n", old_lines[i]));
                    i += 1;
                } else if j < new_lines.len() {
                    output.push_str(&format!("+ {}\n", new_lines[j]));
                    j += 1;
                }
            }
            
            Ok(output)
        }
        DiffMode::Character => {
            // Simple character-by-character diff
            let old_chars: Vec<char> = old_text.chars().collect();
            let new_chars: Vec<char> = new_text.chars().collect();
            
            let mut output = String::new();
            let mut i = 0;
            let mut j = 0;
            
            while i < old_chars.len() || j < new_chars.len() {
                if i < old_chars.len() && j < new_chars.len() && old_chars[i] == new_chars[j] {
                    output.push(old_chars[i]);
                    i += 1;
                    j += 1;
                } else if i < old_chars.len() && (j >= new_chars.len() || old_chars[i] != new_chars[j]) {
                    output.push_str(&format!("[-{}]", old_chars[i]));
                    i += 1;
                } else if j < new_chars.len() {
                    output.push_str(&format!("[+{}]", new_chars[j]));
                    j += 1;
                }
            }
            
            Ok(output)
        }
    }
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self {
            mode: DiffMode::Line,
            detect_renames: true,
            detect_copies: false,
            similarity_threshold: 0.7,
            context_lines: 3,
        }
    }
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

    #[test]
    fn test_similarity_calculation() {
        // Identical content
        let a = b"Hello, World!";
        let b = b"Hello, World!";
        assert_eq!(calculate_similarity(a, b), 1.0);
        
        // Completely different content
        let a = b"Hello, World!";
        let b = b"Goodbye, Universe!";
        let similarity = calculate_similarity(a, b);
        assert!(similarity < 0.5);
        
        // Similar content
        let a = b"Hello, World! This is a test.";
        let b = b"Hello, World! This is a test with more text.";
        let similarity = calculate_similarity(a, b);
        assert!(similarity > 0.5);
        assert!(similarity < 1.0);
    }

    #[test]
    fn test_rename_detection() {
        let mut deleted_files = HashMap::new();
        let mut added_files = HashMap::new();
        
        deleted_files.insert("old_file.txt".to_string(), b"Hello, World! This is test content.".to_vec());
        added_files.insert("new_file.txt".to_string(), b"Hello, World! This is test content.".to_vec());
        
        let renames = detect_renames(&deleted_files, &added_files, 0.8);
        
        assert_eq!(renames.len(), 1);
        assert_eq!(renames[0].old_path, "old_file.txt");
        assert_eq!(renames[0].new_path, "new_file.txt");
        assert!(renames[0].similarity > 0.8);
    }

    #[test]
    fn test_copy_detection() {
        let mut existing_files = HashMap::new();
        let mut added_files = HashMap::new();
        
        existing_files.insert("source.txt".to_string(), b"Hello, World! This is test content.".to_vec());
        added_files.insert("copy.txt".to_string(), b"Hello, World! This is test content.".to_vec());
        
        let copies = detect_copies(&existing_files, &added_files, 0.8);
        
        assert_eq!(copies.len(), 1);
        assert_eq!(copies[0].source_path, "source.txt");
        assert_eq!(copies[0].dest_path, "copy.txt");
        assert!(copies[0].similarity > 0.8);
    }

    #[test]
    fn test_word_diff() {
        let old_text = "Hello world this is a test";
        let new_text = "Hello beautiful world this is a great test";
        
        let changes = word_diff(old_text, new_text);
        
        // Should detect the insertion of "beautiful" and "great"
        assert!(changes.iter().any(|(change_type, word)| 
            change_type == "insert" && word == "beautiful"));
        assert!(changes.iter().any(|(change_type, word)| 
            change_type == "insert" && word == "great"));
        
        // Should preserve common words
        assert!(changes.iter().any(|(change_type, word)| 
            change_type == "equal" && word == "Hello"));
    }

    #[test]
    fn test_enhanced_diff_word_mode() {
        let old_content = b"Hello world";
        let new_content = b"Hello beautiful world";
        
        let options = DiffOptions {
            mode: DiffMode::Word,
            ..Default::default()
        };
        
        let diff = enhanced_diff(old_content, new_content, &options).unwrap();
        assert!(diff.contains("[+beautiful]"));
        assert!(diff.contains("Hello"));
    }

    #[test]
    fn test_enhanced_diff_line_mode() {
        let old_content = b"line1\nline2\nline3";
        let new_content = b"line1\nnew_line\nline3";
        
        let options = DiffOptions {
            mode: DiffMode::Line,
            context_lines: 1,
            ..Default::default()
        };
        
        let diff = enhanced_diff(old_content, new_content, &options).unwrap();
        assert!(diff.contains("- line2"));
        assert!(diff.contains("+ new_line"));
    }

    #[test]
    fn test_diff_options_default() {
        let options = DiffOptions::default();
        assert!(matches!(options.mode, DiffMode::Line));
        assert!(options.detect_renames);
        assert!(!options.detect_copies);
        assert_eq!(options.similarity_threshold, 0.7);
        assert_eq!(options.context_lines, 3);
    }

    #[test]
    fn test_file_rename_serialization() {
        let rename = FileRename {
            old_path: "old.txt".to_string(),
            new_path: "new.txt".to_string(),
            similarity: 0.95,
        };
        
        let serialized = serde_json::to_string(&rename).unwrap();
        let deserialized: FileRename = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized.old_path, rename.old_path);
        assert_eq!(deserialized.new_path, rename.new_path);
        assert_eq!(deserialized.similarity, rename.similarity);
    }

    #[test]
    fn test_file_copy_serialization() {
        let copy = FileCopy {
            source_path: "source.txt".to_string(),
            dest_path: "dest.txt".to_string(),
            similarity: 0.85,
        };
        
        let serialized = serde_json::to_string(&copy).unwrap();
        let deserialized: FileCopy = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized.source_path, copy.source_path);
        assert_eq!(deserialized.dest_path, copy.dest_path);
        assert_eq!(deserialized.similarity, copy.similarity);
    }
}
