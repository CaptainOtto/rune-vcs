
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

pub fn unpack_blob(pack_data: &[u8], entry: &PackEntry) -> Result<Vec<u8>> {
    let start = entry.offset as usize;
    let end = start + entry.size as usize;
    if end > pack_data.len() {
        anyhow::bail!("Pack entry extends beyond pack data");
    }
    let compressed_data = &pack_data[start..end];
    let decompressed = zstd::decode_all(compressed_data)?;
    Ok(decompressed)
}

impl PackIndex {
    pub fn find_entry(&self, path: &str) -> Option<&PackEntry> {
        self.entries.iter().find(|entry| entry.path == path)
    }

    pub fn total_size(&self) -> u64 {
        self.entries.iter().map(|e| e.size).sum()
    }

    pub fn verify_checksum(&self, pack_data: &[u8]) -> bool {
        let computed = format!("{}", blake3::hash(pack_data));
        computed == self.checksum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_empty_blobs() {
        let blobs = Vec::new();
        let result = pack_blobs(blobs).unwrap();
        
        let (pack_data, index) = result;
        assert!(pack_data.is_empty());
        assert!(index.entries.is_empty());
        assert!(!index.checksum.is_empty());
    }

    #[test]
    fn test_pack_single_blob() {
        let blobs = vec![
            ("test.txt".to_string(), b"Hello, World!".to_vec()),
        ];
        
        let (pack_data, index) = pack_blobs(blobs).unwrap();
        
        assert!(!pack_data.is_empty());
        assert_eq!(index.entries.len(), 1);
        assert_eq!(index.entries[0].path, "test.txt");
        assert!(index.entries[0].size > 0);
        assert_eq!(index.entries[0].offset, 0);
        assert!(!index.checksum.is_empty());
    }

    #[test]
    fn test_pack_multiple_blobs() {
        let blobs = vec![
            ("file1.txt".to_string(), b"Content of file 1".to_vec()),
            ("file2.txt".to_string(), b"Content of file 2".to_vec()),
            ("file3.txt".to_string(), b"Content of file 3".to_vec()),
        ];
        
        let (pack_data, index) = pack_blobs(blobs).unwrap();
        
        assert!(!pack_data.is_empty());
        assert_eq!(index.entries.len(), 3);
        
        // Check that offsets are properly calculated
        assert_eq!(index.entries[0].offset, 0);
        assert_eq!(index.entries[1].offset, index.entries[0].size);
        assert_eq!(index.entries[2].offset, index.entries[0].size + index.entries[1].size);
        
        // Verify paths
        assert_eq!(index.entries[0].path, "file1.txt");
        assert_eq!(index.entries[1].path, "file2.txt");
        assert_eq!(index.entries[2].path, "file3.txt");
    }

    #[test]
    fn test_pack_and_unpack_roundtrip() {
        let original_data = b"This is test content for compression and decompression.";
        let blobs = vec![
            ("test_file.txt".to_string(), original_data.to_vec()),
        ];
        
        let (pack_data, index) = pack_blobs(blobs).unwrap();
        let entry = &index.entries[0];
        let unpacked = unpack_blob(&pack_data, entry).unwrap();
        
        assert_eq!(unpacked, original_data);
    }

    #[test]
    fn test_pack_large_data() {
        let large_content = vec![b'A'; 10000]; // 10KB of 'A's
        let blobs = vec![
            ("large_file.bin".to_string(), large_content.clone()),
        ];
        
        let (pack_data, index) = pack_blobs(blobs).unwrap();
        
        // Verify compression actually happened
        assert!(pack_data.len() < large_content.len());
        
        let entry = &index.entries[0];
        let unpacked = unpack_blob(&pack_data, entry).unwrap();
        assert_eq!(unpacked, large_content);
    }

    #[test]
    fn test_pack_entry_creation() {
        let entry = PackEntry {
            path: "test.txt".to_string(),
            size: 1024,
            offset: 512,
        };
        
        assert_eq!(entry.path, "test.txt");
        assert_eq!(entry.size, 1024);
        assert_eq!(entry.offset, 512);
    }

    #[test]
    fn test_pack_index_find_entry() {
        let blobs = vec![
            ("file1.txt".to_string(), b"Content 1".to_vec()),
            ("file2.txt".to_string(), b"Content 2".to_vec()),
            ("file3.txt".to_string(), b"Content 3".to_vec()),
        ];
        
        let (_, index) = pack_blobs(blobs).unwrap();
        
        let found = index.find_entry("file2.txt");
        assert!(found.is_some());
        assert_eq!(found.unwrap().path, "file2.txt");
        
        let not_found = index.find_entry("nonexistent.txt");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_pack_index_total_size() {
        let blobs = vec![
            ("file1.txt".to_string(), b"Small".to_vec()),
            ("file2.txt".to_string(), b"Medium content".to_vec()),
            ("file3.txt".to_string(), b"Larger content for testing".to_vec()),
        ];
        
        let (_, index) = pack_blobs(blobs).unwrap();
        let total = index.total_size();
        let expected: u64 = index.entries.iter().map(|e| e.size).sum();
        
        assert_eq!(total, expected);
        assert!(total > 0);
    }

    #[test]
    fn test_pack_index_verify_checksum() {
        let blobs = vec![
            ("test.txt".to_string(), b"Test content".to_vec()),
        ];
        
        let (pack_data, index) = pack_blobs(blobs).unwrap();
        
        // Valid checksum should pass
        assert!(index.verify_checksum(&pack_data));
        
        // Modified data should fail checksum
        let mut modified_data = pack_data.clone();
        if !modified_data.is_empty() {
            modified_data[0] = modified_data[0].wrapping_add(1);
            assert!(!index.verify_checksum(&modified_data));
        }
    }

    #[test]
    fn test_unpack_invalid_entry() {
        let blobs = vec![
            ("test.txt".to_string(), b"Valid content".to_vec()),
        ];
        
        let (pack_data, _) = pack_blobs(blobs).unwrap();
        
        // Create invalid entry that extends beyond pack data
        let invalid_entry = PackEntry {
            path: "invalid.txt".to_string(),
            size: (pack_data.len() + 100) as u64,
            offset: 0,
        };
        
        let result = unpack_blob(&pack_data, &invalid_entry);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("extends beyond"));
    }

    #[test]
    fn test_pack_empty_file() {
        let blobs = vec![
            ("empty.txt".to_string(), Vec::new()),
            ("nonempty.txt".to_string(), b"content".to_vec()),
        ];
        
        let (pack_data, index) = pack_blobs(blobs).unwrap();
        
        assert_eq!(index.entries.len(), 2);
        
        // Find and unpack empty file
        let empty_entry = index.find_entry("empty.txt").unwrap();
        let unpacked_empty = unpack_blob(&pack_data, empty_entry).unwrap();
        assert!(unpacked_empty.is_empty());
        
        // Find and unpack non-empty file
        let content_entry = index.find_entry("nonempty.txt").unwrap();
        let unpacked_content = unpack_blob(&pack_data, content_entry).unwrap();
        assert_eq!(unpacked_content, b"content");
    }

    #[test]
    fn test_pack_binary_data() {
        let binary_data = vec![0, 1, 2, 3, 255, 254, 253, 128, 127];
        let blobs = vec![
            ("binary.bin".to_string(), binary_data.clone()),
        ];
        
        let (pack_data, index) = pack_blobs(blobs).unwrap();
        let entry = &index.entries[0];
        let unpacked = unpack_blob(&pack_data, entry).unwrap();
        
        assert_eq!(unpacked, binary_data);
    }

    #[test]
    fn test_pack_with_special_characters() {
        let content = "Hello 🦀 Rust! 中文 🌟".as_bytes().to_vec();
        let blobs = vec![
            ("unicode_file_🦀.txt".to_string(), content.clone()),
        ];
        
        let (pack_data, index) = pack_blobs(blobs).unwrap();
        let entry = index.find_entry("unicode_file_🦀.txt").unwrap();
        let unpacked = unpack_blob(&pack_data, entry).unwrap();
        
        assert_eq!(unpacked, content);
    }

    #[test]
    fn test_pack_index_serialization() {
        let blobs = vec![
            ("test.txt".to_string(), b"Test content".to_vec()),
        ];
        
        let (_, index) = pack_blobs(blobs).unwrap();
        
        // Test JSON serialization
        let json = serde_json::to_string(&index).unwrap();
        assert!(!json.is_empty());
        
        let deserialized: PackIndex = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.entries.len(), index.entries.len());
        assert_eq!(deserialized.checksum, index.checksum);
        assert_eq!(deserialized.entries[0].path, index.entries[0].path);
    }

    #[test]
    fn test_compression_efficiency() {
        // Test with highly compressible data
        let repetitive_data = "AAAAAAAAAA".repeat(1000).into_bytes();
        let blobs = vec![
            ("repetitive.txt".to_string(), repetitive_data.clone()),
        ];
        
        let (pack_data, index) = pack_blobs(blobs).unwrap();
        
        // Compressed size should be much smaller than original
        assert!(pack_data.len() < repetitive_data.len() / 10);
        
        // But unpacking should restore original
        let entry = &index.entries[0];
        let unpacked = unpack_blob(&pack_data, entry).unwrap();
        assert_eq!(unpacked, repetitive_data);
    }

    #[test]
    fn test_debug_formatting() {
        let entry = PackEntry {
            path: "debug_test.txt".to_string(),
            size: 42,
            offset: 100,
        };
        
        let debug_str = format!("{:?}", entry);
        assert!(debug_str.contains("PackEntry"));
        assert!(debug_str.contains("debug_test.txt"));
        assert!(debug_str.contains("42"));
        assert!(debug_str.contains("100"));
        
        let index = PackIndex {
            entries: vec![entry],
            checksum: "test_checksum".to_string(),
        };
        
        let index_debug = format!("{:?}", index);
        assert!(index_debug.contains("PackIndex"));
        assert!(index_debug.contains("test_checksum"));
    }
}
