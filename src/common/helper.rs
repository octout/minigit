use super::git_object;

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
// TreeEntryDisplay = { mode, name, hash_hex }
