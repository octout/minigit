use std::fs;

use crate::common::git_object;
use crate::common::helper;
use crate::common::index_readed;

pub fn add_file(path: &str) -> Result<String, String> {
    let content = fs::read(path).map_err(|e| format!("failed to read file: {}", e))?;
    let header = format!("blob {}\0", content.len());
    let mut blob = header.into_bytes();
    blob.extend(&content);

    let hash_hex = git_object::write_object(&blob)?;
    return Ok(hash_hex);
}

pub fn add_index(path: &str, hash_hex: &String) -> Result<(), String> {
    let mut index_vec = match index_readed::read_index() {
        Ok(vec) => vec,
        Err(_) => Vec::new(),
    };
    let tree_entries = match helper::resolve_head_commit_hash() {
        Ok(commit_hash) => {
            let tree_hash = helper::get_tree_hash_from_commit(&commit_hash)?;
            helper::collect_tree_entries(&tree_hash)?
        }
        Err(_) => Vec::new(),
    };

    match index_vec.iter_mut().find(|idx| idx.path == path) {
        Some(index) => {
            index.hex = hash_hex.clone();
            index.status = if tree_entries
                .iter()
                .any(|(entry_path, _)| entry_path == path)
            {
                "change"
            } else {
                "create"
            }
            .to_string();
        }
        None => {
            let status = if tree_entries
                .iter()
                .any(|(entry_path, _)| entry_path == path)
            {
                "change"
            } else {
                "create"
            };
            index_vec.push(index_readed::IndexReaded::new(path, hash_hex, status));
        }
    }
    index_readed::write_index(&index_vec).map_err(|e| format!("failed to write index: {}", e))?;

    return Ok(());
}
