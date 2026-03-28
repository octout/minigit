use std::fs;
use std::fs::File;

pub fn create_init_file() -> Result<(), String> {
    fs::create_dir("./.minigit")
        .map_err(|e| format!("failed to create .minigit: {}", e))?;
    
    File::create("./.minigit/index")
        .map_err(|e| format!("failed to create index: {}", e))?;
    
    fs::create_dir("./.minigit/refs")
        .map_err(|e| format!("failed to create refs: {}", e))?;
    
    File::create("./.minigit/refs/main")
        .map_err(|e| format!("failed to create main: {}", e))?;
    
    fs::create_dir("./.minigit/objects")
        .map_err(|e| format!("failed to create objects: {}", e))?;
    
    Ok(())
}