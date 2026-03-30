use super::GitObject;

pub struct Commit {
    pub tree_hash: String,
    pub parent_hash: Option<String>,
    pub author: String,
    pub committer: String,
    pub message: String,
}

impl Commit {
    pub fn new(tree_hash: String, parent_hash: Option<String>, message: String) -> Self {
        Self {
            tree_hash,
            parent_hash,
            author: "foo <bar@example.com>".to_string(),
            committer: "foo <bar@example.com>".to_string(),
            message,
        }
    }

    pub fn parse_tree_hash(body: &[u8]) -> Result<String, String> {
        let content =
            std::str::from_utf8(body).map_err(|e| format!("invalid commit content: {}", e))?;
        for line in content.lines() {
            if let Some(hash) = line.strip_prefix("tree ") {
                return Ok(hash.to_string());
            }
        }
        Err("tree hash not found in commit object".to_string())
    }
}

impl GitObject for Commit {
    fn object_type(&self) -> &str {
        "commit"
    }

    fn serialize_body(&self) -> Vec<u8> {
        let mut body = format!("tree {}\n", self.tree_hash);
        if let Some(parent) = &self.parent_hash {
            body.push_str(&format!("parent {}\n", parent));
        }
        body.push_str(&format!("author {}\n", self.author));
        body.push_str(&format!("committer {}\n", self.committer));
        body.push('\n');
        body.push_str(&self.message);
        body.push('\n');
        body.into_bytes()
    }
}
