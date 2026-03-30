use std::fs;

use super::tree;
use crate::common::git_object;
use crate::common::helper;
use crate::common::index_readed;

pub fn commit(message: &String) -> Result<String, String> {
    let tree_hash = create_tree_object()?;
    let commit_hash = create_commit_object(&tree_hash, message)?;

    let head_ref_path = helper::resolve_head_ref()?;
    fs::write(&head_ref_path, &commit_hash)
        .map_err(|e| format!("failed to update HEAD ref: {}", e))?;
    return Ok("".to_string());
}

fn create_tree_object() -> Result<String, String> {
    let index_vec =
        index_readed::read_index().map_err(|e| format!("failed to read index: {}", e))?;

    let entries: Vec<(String, String)> = index_vec
        .iter()
        .map(|index| (index.path.clone(), index.hex.clone()))
        .collect();

    let tree_data = tree::build_tree(entries);
    tree::save_tree(&tree_data)
}

fn create_commit_object(tree_hash: &String, message: &String) -> Result<String, String> {
    let mut body: Vec<u8> = Vec::new();
    body.extend(format!("tree {}\n", tree_hash).as_bytes());

    let parent_commit = get_parent_commit_hash(&helper::resolve_head_ref().unwrap());
    if let Some(parent_commit) = parent_commit {
        body.extend(format!("parent {}\n", parent_commit).as_bytes());
    }
    body.extend("author foo<bar@example.com>\n".as_bytes());
    body.extend("committer foo<bar@example.com>\n".as_bytes());
    body.extend("\n".as_bytes());
    body.extend(format!("{}\n", message).as_bytes());

    let header = format!("commit {}\0", body.len());
    let mut commit_content = header.into_bytes();
    commit_content.extend(body);
    let hash_hex = git_object::write_object(&commit_content)?;

    return Ok(hash_hex);
}

fn get_parent_commit_hash(ref_path: &String) -> Option<String> {
    match fs::read_to_string(ref_path) {
        Ok(content) => {
            let hash = content.trim().to_string();
            if hash.is_empty() { None } else { Some(hash) }
        }
        Err(_) => None,
    }
}
