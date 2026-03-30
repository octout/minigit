use crate::index::{Index, IndexEntry};
use crate::object::blob::Blob;
use crate::object::store;

pub fn execute(path: &str) -> Result<(), String> {
    let blob = Blob::from_file(path)?;
    let hash = store::save(&blob)?;

    let mut index = Index::load();

    match index.find_mut(path) {
        Some(entry) => {
            entry.hash = hash;
        }
        None => {
            index.add(IndexEntry::new(path, &hash));
        }
    }

    index.save()
}
