use std::fs;

use crate::object::commit::Commit;
use crate::object::store;

pub fn resolve_head() -> Result<String, String> {
    let head_content =
        fs::read_to_string("./.minigit/HEAD").map_err(|e| format!("failed to read HEAD: {}", e))?;
    let head_ref = head_content.trim();
    if let Some(ref_rel) = head_ref.strip_prefix("ref: ") {
        return Ok(format!("./.minigit/{}", ref_rel));
    }
    Err("invalid HEAD format".to_string())
}

pub fn current_commit() -> Option<String> {
    let ref_path = resolve_head().ok()?;
    let hash = fs::read_to_string(ref_path).ok()?;
    let hash = hash.trim().to_string();
    if hash.is_empty() { None } else { Some(hash) }
}

pub fn current_tree_hash() -> Option<String> {
    let commit_hash = current_commit()?;
    let (_, body) = store::load(&commit_hash).ok()?;
    Commit::parse_tree_hash(&body).ok()
}

pub fn update_head(hash: &str) -> Result<(), String> {
    let ref_path = resolve_head()?;
    fs::write(&ref_path, hash).map_err(|e| format!("failed to update HEAD ref: {}", e))
}
