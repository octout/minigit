use std::fs;

pub fn execute() -> Result<(), String> {
    fs::create_dir("./.minigit").map_err(|e| format!("failed to create .minigit: {}", e))?;

    fs::write("./.minigit/HEAD", "ref: refs/heads/main\n")
        .map_err(|e| format!("failed to create HEAD: {}", e))?;

    fs::create_dir("./.minigit/refs").map_err(|e| format!("failed to create refs: {}", e))?;

    fs::create_dir("./.minigit/refs/heads")
        .map_err(|e| format!("failed to create heads: {}", e))?;

    fs::create_dir("./.minigit/objects").map_err(|e| format!("failed to create objects: {}", e))?;

    Ok(())
}
