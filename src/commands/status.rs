use crate::index::Index;
use crate::object::tree;
use crate::refs;

pub fn execute() -> Result<(), String> {
    let index = Index::load();
    if index.is_empty() {
        println!("nothing staged");
        return Ok(());
    }

    let committed = match refs::current_tree_hash() {
        Some(tree_hash) => tree::collect_entries(&tree_hash).unwrap_or_default(),
        None => Vec::new(),
    };

    for entry in &index.entries {
        let status = match committed.iter().find(|(path, _)| path == &entry.path) {
            Some((_, hash)) if hash == &entry.hash => continue,
            Some(_) => "modified",
            None => "new file",
        };
        println!("{}: {}", status, entry.path);
    }

    Ok(())
}
