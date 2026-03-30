use std::fs;

use crate::refs;

pub fn execute(branch_name: &[String]) -> Result<(), String> {
    if branch_name.len() > 1 {
        return Err("Only one branch can be created at a time.".to_string());
    }
    let branch_list = fs::read_dir("./.minigit/refs/heads/")
        .map_err(|e| format!("Failed to read branches: {}", e))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().into_string().unwrap())
        .collect::<Vec<String>>();
    let current_branch = fs::read_to_string("./.minigit/HEAD")
        .map_err(|e| format!("Failed to read current branch: {}", e))?
        .trim()
        .strip_prefix("ref: refs/heads/")
        .unwrap_or("")
        .to_string();

    if branch_name.is_empty() {
        println!("Branches:");
        for branch in branch_list {
            if branch == current_branch {
                println!("* {}", branch);
            } else {
                println!("  {}", branch);
            }
        }
        return Ok(());
    }

    for name in branch_name {
        if branch_list.contains(name) {
            return Err(format!("Branch '{}' already exists.", name));
        }
        let current_commit = refs::current_commit().unwrap();
        fs::write(format!("./.minigit/refs/heads/{}", name), current_commit)
            .map_err(|e| format!("Failed to create branch '{}': {}", name, e))?;
        println!("Branch '{}' created successfully.", name);
    }
    Ok(())
}
