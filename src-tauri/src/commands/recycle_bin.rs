use std::path::Path;

fn ensure_existing_file(path: &str) -> Result<(), String> {
    let target = Path::new(path);

    if !target.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    if !target.is_file() {
        return Err(format!("Path is not a file: {}", path));
    }

    Ok(())
}

fn ensure_existing_dir(path: &str) -> Result<(), String> {
    let target = Path::new(path);

    if !target.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    if !target.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }

    Ok(())
}

#[tauri::command]
pub fn move_file_to_recycle_bin(path: String) -> Result<(), String> {
    ensure_existing_file(&path)?;
    trash::delete(&path).map_err(|err| format!("Failed to move file to recycle bin: {}", err))
}

#[tauri::command]
pub fn move_dir_to_recycle_bin(path: String) -> Result<(), String> {
    ensure_existing_dir(&path)?;
    trash::delete(&path).map_err(|err| format!("Failed to move directory to recycle bin: {}", err))
}
