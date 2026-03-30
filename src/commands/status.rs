use crate::index::Index;
use crate::object::blob::Blob;
use crate::object::store;
use crate::object::tree;
use crate::refs;
use std::fs;
use std::path::Path;

pub fn execute() -> Result<(), String> {
    let index = Index::load();
    let committed = match refs::current_tree_hash() {
        Some(tree_hash) => tree::collect_entries(&tree_hash).unwrap_or_default(),
        None => Vec::new(),
    };
    let working_files = list_working_files(Path::new("."))?;

    // Section 1: Changes to be committed
    let mut staged: Vec<(&str, &str)> = Vec::new();
    for entry in &index.entries {
        if let Some((_, committed_hash)) = committed.iter().find(|(path, _)| path == &entry.path) {
            if committed_hash == &entry.hash {
                continue;
            }
        }
        let status = if committed.iter().any(|(path, _)| path == &entry.path) {
            "modified"
        } else {
            "new file"
        };
        staged.push((status, &entry.path));
    }
    if !staged.is_empty() {
        println!("Changes to be committed:");
        for (status, path) in &staged {
            println!("  {}: {}", status, path);
        }
    }

    // Section 2: Changes not staged for commit
    let mut unstaged: Vec<(&str, &str)> = Vec::new();
    for entry in &index.entries {
        match Blob::from_file(&entry.path) {
            Ok(blob) => {
                let working_hash = store::hash(&blob);
                if working_hash != entry.hash {
                    unstaged.push(("modified", &entry.path));
                }
            }
            Err(_) => {
                unstaged.push(("deleted", &entry.path));
            }
        }
    }
    if !unstaged.is_empty() {
        println!("\nChanges not staged for commit:");
        for (status, path) in &unstaged {
            println!("  {}: {}", status, path);
        }
    }

    // Section 3: Untracked files
    let untracked: Vec<&String> = working_files
        .iter()
        .filter(|file| {
            !index.entries.iter().any(|e| &e.path == *file)
                && !committed.iter().any(|(path, _)| path == *file)
        })
        .collect();
    if !untracked.is_empty() {
        println!("\nUntracked files:");
        for file in &untracked {
            println!("  {}", file);
        }
    }

    Ok(())
}

fn list_working_files(dir: &Path) -> Result<Vec<String>, String> {
    let mut file_path_list = Vec::new();
    let entries = fs::read_dir(dir).map_err(|e| format!("failed to read directory: {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("failed to read directory entry: {}", e))?;
        let path = entry.path();
        if path.is_file() {
            if let Some(rel_path) = path.strip_prefix(".").ok().and_then(|p| p.to_str()) {
                file_path_list.push(rel_path.to_string());
            }
        } else if path.is_dir() && path.file_name().and_then(|n| n.to_str()) != Some(".minigit") {
            let sub_files = list_working_files(&path)?;
            file_path_list.extend(sub_files);
        }
    }
    Ok(file_path_list)
}
