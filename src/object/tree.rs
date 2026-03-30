use std::collections::HashMap;

use super::store;

#[derive(Debug, PartialEq)]
pub enum TreeEntry {
    File { name: String, hash: String },
    Directory { name: String, children: Tree },
}

#[derive(Debug, PartialEq)]
pub struct Tree {
    pub entries: Vec<TreeEntry>,
}

pub fn build_tree(entries: Vec<(String, String)>) -> Tree {
    let mut file_entries: Vec<(String, String)> = Vec::new();
    let mut dir_entries: HashMap<String, Vec<(String, String)>> = HashMap::new();

    for (path, hash) in entries {
        if let Some(pos) = path.find('/') {
            let dir_name = path[..pos].to_string();
            let sub_path = path[pos + 1..].to_string();
            dir_entries
                .entry(dir_name)
                .or_insert_with(Vec::new)
                .push((sub_path, hash));
        } else {
            file_entries.push((path, hash));
        }
    }

    let mut tree_entries: Vec<TreeEntry> = Vec::new();

    for (dir_name, sub_entries) in dir_entries {
        let children = build_tree(sub_entries);
        tree_entries.push(TreeEntry::Directory {
            name: dir_name,
            children,
        });
    }

    for (name, hash) in file_entries {
        tree_entries.push(TreeEntry::File { name, hash });
    }

    Tree {
        entries: tree_entries,
    }
}

pub fn save_tree(tree: &Tree) -> Result<String, String> {
    let mut body: Vec<u8> = Vec::new();

    for entry in &tree.entries {
        match entry {
            TreeEntry::File { name, hash } => {
                body.extend(format!("100644 {}\0", name).as_bytes());
                body.extend(store::hex_to_bytes(hash));
            }
            TreeEntry::Directory { name, children } => {
                let sub_tree_hash = save_tree(children)?;
                body.extend(format!("40000 {}\0", name).as_bytes());
                body.extend(store::hex_to_bytes(&sub_tree_hash));
            }
        }
    }

    let header = format!("tree {}\0", body.len());
    let mut tree_content = header.into_bytes();
    tree_content.extend(body);
    store::save_bytes(&tree_content)
}

#[derive(Debug)]
pub struct ParsedEntry {
    pub mode: String,
    pub name: String,
    pub hash: String,
}

pub fn parse_body(body: &[u8]) -> Result<Vec<ParsedEntry>, String> {
    let mut entries = Vec::new();
    let mut i = 0;
    while i < body.len() {
        let mode_end = body[i..]
            .iter()
            .position(|&b| b == b' ')
            .ok_or("invalid tree entry format")?
            + i;
        let mode = String::from_utf8(body[i..mode_end].to_vec())
            .map_err(|e| format!("invalid tree entry mode: {}", e))?;
        i = mode_end + 1;

        let name_end = body[i..]
            .iter()
            .position(|&b| b == 0)
            .ok_or("invalid tree entry format")?
            + i;
        let name = String::from_utf8(body[i..name_end].to_vec())
            .map_err(|e| format!("invalid tree entry name: {}", e))?;
        i = name_end + 1;

        if i + 20 > body.len() {
            return Err("invalid tree entry format".to_string());
        }
        let hash = store::bytes_to_hex(&body[i..i + 20]);
        i += 20;

        entries.push(ParsedEntry { mode, name, hash });
    }
    Ok(entries)
}

pub fn collect_entries(tree_hash: &str) -> Result<Vec<(String, String)>, String> {
    let (object_type, content) = store::load(tree_hash)?;
    if object_type != "tree" {
        return Err(format!("object {} is not a tree", tree_hash));
    }
    let entries = parse_body(&content)?;
    let mut result: Vec<(String, String)> = Vec::new();
    for entry in entries {
        if entry.mode == "40000" {
            let sub_entries = collect_entries(&entry.hash)?;
            for (sub_name, sub_hash) in sub_entries {
                result.push((format!("{}/{}", entry.name, sub_name), sub_hash));
            }
        } else {
            result.push((entry.name, entry.hash));
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_file() {
        let entries = vec![("foo.txt".to_string(), "abc123".to_string())];
        let tree = build_tree(entries);

        assert_eq!(tree.entries.len(), 1);
        assert_eq!(
            tree.entries[0],
            TreeEntry::File {
                name: "foo.txt".to_string(),
                hash: "abc123".to_string(),
            }
        );
    }

    #[test]
    fn test_multiple_files_flat() {
        let entries = vec![
            ("foo.txt".to_string(), "hash_a".to_string()),
            ("bar.txt".to_string(), "hash_b".to_string()),
        ];
        let tree = build_tree(entries);

        assert_eq!(tree.entries.len(), 2);
        assert!(
            tree.entries
                .iter()
                .all(|e| matches!(e, TreeEntry::File { .. }))
        );
    }

    #[test]
    fn test_single_directory() {
        let entries = vec![
            ("src/main.rs".to_string(), "hash_a".to_string()),
            ("src/lib.rs".to_string(), "hash_b".to_string()),
        ];
        let tree = build_tree(entries);

        assert_eq!(tree.entries.len(), 1);
        if let TreeEntry::Directory { name, children } = &tree.entries[0] {
            assert_eq!(name, "src");
            assert_eq!(children.entries.len(), 2);
            assert!(
                children
                    .entries
                    .iter()
                    .all(|e| matches!(e, TreeEntry::File { .. }))
            );
        } else {
            panic!("Expected directory 'src'");
        }
    }

    #[test]
    fn test_mixed_files_and_directories() {
        let entries = vec![
            ("foo.txt".to_string(), "hash_a".to_string()),
            ("src/main.rs".to_string(), "hash_b".to_string()),
            ("src/lib.rs".to_string(), "hash_c".to_string()),
        ];
        let tree = build_tree(entries);

        assert_eq!(tree.entries.len(), 2);

        let dir_count = tree
            .entries
            .iter()
            .filter(|e| matches!(e, TreeEntry::Directory { .. }))
            .count();
        let file_count = tree
            .entries
            .iter()
            .filter(|e| matches!(e, TreeEntry::File { .. }))
            .count();
        assert_eq!(dir_count, 1);
        assert_eq!(file_count, 1);
    }

    #[test]
    fn test_deeply_nested() {
        let entries = vec![("src/utils/helper.rs".to_string(), "hash_d".to_string())];
        let tree = build_tree(entries);

        assert_eq!(tree.entries.len(), 1);
        if let TreeEntry::Directory { name, children } = &tree.entries[0] {
            assert_eq!(name, "src");
            assert_eq!(children.entries.len(), 1);
            if let TreeEntry::Directory { name, children } = &children.entries[0] {
                assert_eq!(name, "utils");
                assert_eq!(children.entries.len(), 1);
                assert_eq!(
                    children.entries[0],
                    TreeEntry::File {
                        name: "helper.rs".to_string(),
                        hash: "hash_d".to_string(),
                    }
                );
            } else {
                panic!("Expected directory 'utils'");
            }
        } else {
            panic!("Expected directory 'src'");
        }
    }

    #[test]
    fn test_complex_structure() {
        let entries = vec![
            ("README.md".to_string(), "hash_a".to_string()),
            ("src/main.rs".to_string(), "hash_b".to_string()),
            ("src/lib.rs".to_string(), "hash_c".to_string()),
            ("src/utils/helper.rs".to_string(), "hash_d".to_string()),
            ("tests/test_main.rs".to_string(), "hash_e".to_string()),
        ];
        let tree = build_tree(entries);

        let file_count = tree
            .entries
            .iter()
            .filter(|e| matches!(e, TreeEntry::File { .. }))
            .count();
        let dir_count = tree
            .entries
            .iter()
            .filter(|e| matches!(e, TreeEntry::Directory { .. }))
            .count();
        assert_eq!(file_count, 1);
        assert_eq!(dir_count, 2);

        let src = tree
            .entries
            .iter()
            .find(|e| matches!(e, TreeEntry::Directory { name, .. } if name == "src"))
            .unwrap();
        if let TreeEntry::Directory { children, .. } = src {
            let src_files = children
                .entries
                .iter()
                .filter(|e| matches!(e, TreeEntry::File { .. }))
                .count();
            let src_dirs = children
                .entries
                .iter()
                .filter(|e| matches!(e, TreeEntry::Directory { .. }))
                .count();
            assert_eq!(src_files, 2);
            assert_eq!(src_dirs, 1);
        }
    }

    #[test]
    fn test_empty_entries() {
        let entries: Vec<(String, String)> = vec![];
        let tree = build_tree(entries);
        assert_eq!(tree.entries.len(), 0);
    }
}
