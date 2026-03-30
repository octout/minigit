use crate::index::Index;
use crate::object::commit::Commit;
use crate::object::store;
use crate::object::tree;
use crate::refs;

pub fn execute(message: &str) -> Result<(), String> {
    let index = Index::load();
    if index.is_empty() {
        return Err("nothing to commit".to_string());
    }

    let entries: Vec<(String, String)> = index
        .entries
        .iter()
        .map(|e| (e.path.clone(), e.hash.clone()))
        .collect();

    let tree_data = tree::build_tree(entries);
    let tree_hash = tree::save_tree(&tree_data)?;

    let parent_hash = refs::current_commit();
    let commit = Commit::new(tree_hash, parent_hash, message.to_string());
    let commit_hash = store::save(&commit)?;

    refs::update_head(&commit_hash)
}
