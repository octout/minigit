use crate::index::{Index, IndexEntry};
use crate::object::blob::Blob;
use crate::object::store;
use crate::object::tree;
use crate::refs;

pub fn execute(path: &str) -> Result<(), String> {
    let blob = Blob::from_file(path)?;
    let hash = store::save(&blob)?;

    let mut index = Index::load();

    let committed_entries = match refs::current_tree_hash() {
        Some(tree_hash) => tree::collect_entries(&tree_hash).unwrap_or_default(),
        None => Vec::new(),
    };

    let status = if committed_entries.iter().any(|(p, _)| p == path) {
        "change"
    } else {
        "create"
    };

    match index.find_mut(path) {
        Some(entry) => {
            entry.hash = hash;
            entry.status = status.to_string();
        }
        None => {
            index.add(IndexEntry::new(path, &hash, status));
        }
    }

    index.save()
}
