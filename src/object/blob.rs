use std::fs;

use super::GitObject;

pub struct Blob {
    pub content: Vec<u8>,
}

impl Blob {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read(path).map_err(|e| format!("failed to read file: {}", e))?;
        Ok(Self { content })
    }
}

impl GitObject for Blob {
    fn object_type(&self) -> &str {
        "blob"
    }

    fn serialize_body(&self) -> Vec<u8> {
        self.content.clone()
    }
}
