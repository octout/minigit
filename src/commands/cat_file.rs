use crate::object::store;
use crate::object::tree;

pub fn show_type(hash: &str) -> Result<String, String> {
    let (object_type, _) = store::load(hash)?;
    Ok(object_type)
}

pub fn show_content(hash: &str) -> Result<String, String> {
    let (object_type, body) = store::load(hash)?;
    if object_type == "tree" {
        let entries = tree::parse_body(&body)?;
        let mut result = String::new();
        for entry in entries {
            let kind = if entry.mode == "40000" {
                "tree"
            } else {
                "blob"
            };
            result.push_str(&format!(
                "{:0>6} {} {}    {}\n",
                entry.mode, kind, entry.hash, entry.name
            ));
        }
        Ok(result)
    } else {
        String::from_utf8(body).map_err(|e| format!("invalid object content: {}", e))
    }
}
