use crate::index::Index;

pub fn execute() -> Result<(), String> {
    let index = Index::load();
    if index.is_empty() {
        println!("nothing staged");
        return Ok(());
    }
    for entry in &index.entries {
        println!("{} {}", entry.status, entry.path);
    }
    Ok(())
}
