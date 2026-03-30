use std::path::Path;

use crate::index::{Index, IndexEntry};
use crate::object::blob::Blob;
use crate::object::store;

use super::util;

pub fn execute(paths: &[String]) -> Result<(), String> {
    let mut index = Index::load();
    let patterns = util::load_ignore_patterns();

    for path in paths {
        let p = Path::new(path);
        if p.is_file() {
            add_file(&mut index, path)?;
        } else if p.is_dir() {
            let files = util::list_files_in_dir(p, &patterns)?;
            for file in files {
                add_file(&mut index, &file)?;
            }
        } else {
            return Err(format!("pathspec '{}' did not match any files", path));
        }
    }

    index.save()
}

fn add_file(index: &mut Index, path: &str) -> Result<(), String> {
    let blob = Blob::from_file(path)?;
    let hash = store::save(&blob)?;

    match index.find_mut(path) {
        Some(entry) => {
            entry.hash = hash;
        }
        None => {
            index.add(IndexEntry::new(path, &hash));
        }
    }
    Ok(())
}
