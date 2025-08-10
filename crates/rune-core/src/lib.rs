
use serde::{Deserialize, Serialize};

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
