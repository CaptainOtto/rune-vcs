
use serde::{Deserialize, Serialize};

// Core data structures
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Author { pub name: String, pub email: String }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Commit {
    pub id: String,
    pub message: String,
    pub author: Author,
    pub time: i64,
    pub parent: Option<String>,
    pub files: Vec<String>,
    pub branch: String,
}

// Intelligence module moved from rune-cli
pub mod intelligence;

// Advanced ignore system
pub mod ignore;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_author_creation() {
        let author = Author {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        assert_eq!(author.name, "Test User");
        assert_eq!(author.email, "test@example.com");
    }

    #[test]
    fn test_author_serialization() {
        let author = Author {
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        };
        
        let serialized = serde_json::to_string(&author).unwrap();
        let deserialized: Author = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(author.name, deserialized.name);
        assert_eq!(author.email, deserialized.email);
    }

    #[test]
    fn test_commit_creation() {
        let author = Author {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        let commit = Commit {
            id: "abc123".to_string(),
            message: "Initial commit".to_string(),
            author,
            time: 1234567890,
            parent: None,
            files: vec!["README.md".to_string()],
            branch: "main".to_string(),
        };
        
        assert_eq!(commit.id, "abc123");
        assert_eq!(commit.message, "Initial commit");
        assert_eq!(commit.files.len(), 1);
        assert!(commit.parent.is_none());
    }

    #[test]
    fn test_commit_with_parent() {
        let author = Author {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        let commit = Commit {
            id: "def456".to_string(),
            message: "Second commit".to_string(),
            author,
            time: 1234567891,
            parent: Some("abc123".to_string()),
            files: vec!["src/main.rs".to_string(), "Cargo.toml".to_string()],
            branch: "main".to_string(),
        };
        
        assert_eq!(commit.parent, Some("abc123".to_string()));
        assert_eq!(commit.files.len(), 2);
    }

    #[test]
    fn test_commit_serialization() {
        let author = Author {
            name: "Jane Doe".to_string(),
            email: "jane@example.com".to_string(),
        };
        
        let commit = Commit {
            id: "xyz789".to_string(),
            message: "Test commit".to_string(),
            author,
            time: 1234567892,
            parent: Some("def456".to_string()),
            files: vec!["test.rs".to_string()],
            branch: "feature".to_string(),
        };
        
        let serialized = serde_json::to_string(&commit).unwrap();
        let deserialized: Commit = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(commit.id, deserialized.id);
        assert_eq!(commit.message, deserialized.message);
        assert_eq!(commit.author.name, deserialized.author.name);
        assert_eq!(commit.author.email, deserialized.author.email);
        assert_eq!(commit.time, deserialized.time);
        assert_eq!(commit.parent, deserialized.parent);
        assert_eq!(commit.files, deserialized.files);
        assert_eq!(commit.branch, deserialized.branch);
    }
}
