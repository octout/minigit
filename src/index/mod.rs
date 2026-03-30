use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct IndexEntry {
    pub path: String,
    pub hash: String,
    pub status: String,
}

impl IndexEntry {
    pub fn new(path: &str, hash: &str, status: &str) -> Self {
        Self {
            path: path.to_string(),
            hash: hash.to_string(),
            status: status.to_string(),
        }
    }
}

pub struct Index {
    pub entries: Vec<IndexEntry>,
}

impl Index {
    pub fn load() -> Self {
        let index_path = Path::new("./.minigit/index");
        let mut entries = Vec::new();
        if let Ok(file) = File::open(index_path) {
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line.unwrap();
                let parts: Vec<&str> = line.split(' ').collect();
                if parts.len() >= 3 {
                    let mut path = parts[1].to_string();
                    if path.starts_with("./") {
                        path = path[2..].to_string();
                    }
                    entries.push(IndexEntry::new(&path, parts[2], parts[0]));
                }
            }
        }
        Self { entries }
    }

    pub fn save(&self) -> Result<(), String> {
        let mut content = String::new();
        for entry in &self.entries {
            content.push_str(&format!(
                "{} ./{} {}\n",
                entry.status, entry.path, entry.hash
            ));
        }
        std::fs::write("./.minigit/index", content)
            .map_err(|e| format!("failed to write index: {}", e))
    }

    pub fn find_mut(&mut self, path: &str) -> Option<&mut IndexEntry> {
        self.entries.iter_mut().find(|e| e.path == path)
    }

    pub fn add(&mut self, entry: IndexEntry) {
        self.entries.push(entry);
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
