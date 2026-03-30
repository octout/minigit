use std::fs;
use std::path::Path;

pub fn load_ignore_patterns() -> Vec<String> {
    let mut patterns = vec![".git".to_string(), ".minigit".to_string()];
    if let Ok(content) = fs::read_to_string(".gitignore") {
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                let line = line.trim_start_matches('/');
                patterns.push(line.to_string());
            }
        }
    }
    patterns
}

pub fn is_ignored(name: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|p| name == p)
}

pub fn list_files_in_dir(dir: &Path, patterns: &[String]) -> Result<Vec<String>, String> {
    let mut file_path_list = Vec::new();
    let entries = fs::read_dir(dir).map_err(|e| format!("failed to read directory: {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("failed to read directory entry: {}", e))?;
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if is_ignored(name, patterns) {
            continue;
        }
        if path.is_file() {
            let rel = normalize_path(&path);
            file_path_list.push(rel);
        } else if path.is_dir() {
            let sub_files = list_files_in_dir(&path, patterns)?;
            file_path_list.extend(sub_files);
        }
    }
    Ok(file_path_list)
}

fn normalize_path(path: &Path) -> String {
    let mut components = Vec::new();
    for c in path.components() {
        match c {
            std::path::Component::CurDir => {}
            std::path::Component::Normal(s) => {
                components.push(s.to_str().unwrap_or(""));
            }
            _ => {}
        }
    }
    components.join("/")
}
