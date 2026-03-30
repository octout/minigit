use super::git_object;
use std::fs;

// 種別を表示（-t）
pub fn cat_file_type(hash: &str) -> Result<String, String> {
    let (object_type, _) = git_object::read_object(&hash)?;
    return Ok(object_type);
}

// 中身を表示（-p）
pub fn cat_file_print(hash: &str) -> Result<String, String> {
    let (object_type, content) = git_object::read_object(hash)?;
    if object_type == "tree" {
        let entries = parse_tree_body(&content)?;
        let mut result = String::new();
        for entry in entries {
            let kind = if entry.mode == "40000" {
                "tree"
            } else {
                "blob"
            };
            result.push_str(&format!(
                "{:0>6} {} {}    {}\n",
                entry.mode, kind, entry.hash_hex, entry.name
            ));
        }
        return Ok(result);
    }
    return String::from_utf8(content).map_err(|e| format!("invalid object content: {}", e));
}

pub struct TreeEntryDisplay {
    pub mode: String,
    pub name: String,
    pub hash_hex: String,
}
fn parse_tree_body(body: &[u8]) -> Result<Vec<TreeEntryDisplay>, String> {
    let mut entries: Vec<TreeEntryDisplay> = Vec::new();
    let mut i = 0;
    while i < body.len() {
        // モードを読み取る
        let mode_end = body[i..]
            .iter()
            .position(|&b| b == b' ')
            .ok_or("invalid tree entry format")?
            + i;
        let mode = String::from_utf8(body[i..mode_end].to_vec())
            .map_err(|e| format!("invalid tree entry mode: {}", e))?;
        i = mode_end + 1;

        // ファイル名を読み取る
        let name_end = body[i..]
            .iter()
            .position(|&b| b == 0)
            .ok_or("invalid tree entry format")?
            + i;
        let name = String::from_utf8(body[i..name_end].to_vec())
            .map_err(|e| format!("invalid tree entry name: {}", e))?;
        i = name_end + 1;

        // ハッシュを読み取る
        if i + 20 > body.len() {
            return Err("invalid tree entry format".to_string());
        }
        let hash_hex = body[i..i + 20]
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        i += 20;

        entries.push(TreeEntryDisplay {
            mode,
            name,
            hash_hex,
        });
    }
    return Ok(entries);
}

pub fn collect_tree_entries(tree_hash: &str) -> Result<Vec<(String, String)>, String> {
    let (object_type, content) = git_object::read_object(tree_hash)?;
    if object_type != "tree" {
        return Err(format!("object {} is not a tree", tree_hash));
    }
    let entries = parse_tree_body(&content)?;
    let mut result: Vec<(String, String)> = Vec::new();
    for entry in entries {
        if entry.mode == "40000" {
            // サブツリーの場合は再帰的に収集
            let sub_entries = collect_tree_entries(&entry.hash_hex)?;
            for (sub_name, sub_hash) in sub_entries {
                result.push((format!("{}/{}", entry.name, sub_name), sub_hash));
            }
        } else {
            // ブロブの場合はそのまま追加
            result.push((entry.name, entry.hash_hex));
        }
    }
    return Ok(result);
}

pub fn resolve_head_ref() -> Result<String, String> {
    let head_path = "./.minigit/HEAD";
    let head_content =
        fs::read_to_string(head_path).map_err(|e| format!("failed to read HEAD file: {}", e))?;
    let head_ref = head_content.trim();
    if head_ref.starts_with("ref: ") {
        let ref_path = format!("./.minigit/{}", &head_ref[5..]);
        return Ok(ref_path);
    }
    return Err("invalid HEAD format".to_string());
}

pub fn resolve_head_commit_hash() -> Result<String, String> {
    let head_ref_path = resolve_head_ref()?;
    let commit_hash = fs::read_to_string(head_ref_path)
        .map_err(|e| format!("failed to read HEAD ref file: {}", e))?;
    return Ok(commit_hash.trim().to_string());
}

pub fn get_tree_hash_from_commit(commit_hash: &str) -> Result<String, String> {
    let (object_type, content) = git_object::read_object(commit_hash)?;
    if object_type != "commit" {
        return Err(format!("object {} is not a commit", commit_hash));
    }
    let content_str =
        String::from_utf8(content).map_err(|e| format!("invalid commit object content: {}", e))?;
    for line in content_str.lines() {
        if line.starts_with("tree ") {
            return Ok(line[5..].to_string());
        }
    }
    return Err("tree hash not found in commit object".to_string());
}
