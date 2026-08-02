use std::path::PathBuf;

use crate::utils::ssmt_compress_utils::SSMTCompressUtils;

#[tauri::command]
pub async fn extract_zip_archive(zip_path: String, dest_dir: String) -> Result<(), String> {
    let zip_path = PathBuf::from(&zip_path);
    let dest_dir = PathBuf::from(&dest_dir);

    if !zip_path.exists() {
        return Err(format!(
            "Zip file not found: {}",
            zip_path.to_string_lossy()
        ));
    }

    if !dest_dir.exists() {
        std::fs::create_dir_all(&dest_dir)
            .map_err(|e| format!("Failed to create destination: {}", e))?;
    }

    SSMTCompressUtils::extract_zip_archive(&zip_path, &dest_dir, |_current, _total| {}).map(|_| ())
}

#[tauri::command]
pub async fn create_rar_archive(source_dir: String, output_path: String) -> Result<(), String> {
    let source_dir = PathBuf::from(&source_dir);
    let output_path = PathBuf::from(&output_path);

    SSMTCompressUtils::create_rar_archive(&source_dir, &output_path)
}

#[tauri::command]
pub async fn create_mod_archive(
    source_dir: String,
    output_path: String,
    format: String,
    password: Option<String>,
) -> Result<(), String> {
    let source_dir = PathBuf::from(&source_dir);
    let output_path = PathBuf::from(&output_path);

    SSMTCompressUtils::create_mod_archive(&source_dir, &output_path, &format, password.as_deref())
}
