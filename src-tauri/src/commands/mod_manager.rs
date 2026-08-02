use crate::ini_container::migoto_data_types::condition::Condition;
use crate::ini_container::migoto_data_types::domain::Key;
use crate::ini_container::migoto_ini_container::container::MigotoIniContainer;
use crate::utils::ssmt_compress_utils::{ArchivePreview, ExtractResult, SSMTCompressUtils};

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, State};

// Watcher State
pub struct ModWatcher(pub Mutex<Option<RecommendedWatcher>>);

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModInfo {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub path: String,
    pub relative_path: String,
    pub preview_images: Vec<String>,
    pub group: String, // Changed to 'group' to match JS interface
    pub is_dir: bool,
    pub last_modified: u64,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GroupInfo {
    pub id: String,
    pub name: String,
    pub icon_path: Option<String>,
    pub path: String,
    pub enabled: bool,
    pub mod_count: u64,
}

fn modified_unix_seconds(path: &Path) -> u64 {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn is_content_image_file(name_lower: &str) -> bool {
    matches!(name_lower, _ if name_lower.ends_with(".jpg")
        || name_lower.ends_with(".jpeg")
        || name_lower.ends_with(".png")
        || name_lower.ends_with(".gif")
        || name_lower.ends_with(".bmp")
        || name_lower.ends_with(".webp"))
}

fn is_standard_group_icon(name_lower: &str) -> bool {
    matches!(
        name_lower,
        "folder.jpg" | "folder.png" | "icon.jpg" | "icon.png" | "cover.jpg" | "cover.png"
    )
}

fn mods_root(install_dir: &str) -> PathBuf {
    let install_path = PathBuf::from(install_dir);
    if install_path
        .file_name()
        .and_then(|value| value.to_str())
        .map(|name| name.eq_ignore_ascii_case("Mods"))
        .unwrap_or(false)
    {
        install_path
    } else {
        install_path.join("Mods")
    }
}

fn is_leaf_mod_dir(path: &Path) -> bool {
    let mut has_ini = false;
    let mut has_subdirs = false;
    let mut has_content_images = false;

    if let Ok(subs) = fs::read_dir(path) {
        for sub in subs.flatten() {
            let sub_path = sub.path();
            if sub_path.is_dir() {
                let sub_name = sub_path.file_name().unwrap_or_default().to_string_lossy();
                if !sub_name.starts_with('.') && !sub_name.starts_with('$') {
                    has_subdirs = true;
                }
            } else if sub_path.is_file() {
                if let Some(name) = sub_path.file_name().and_then(|n| n.to_str()) {
                    let lower = name.to_lowercase();
                    if lower.ends_with(".ini") {
                        has_ini = true;
                    }
                    if is_content_image_file(&lower) && !is_standard_group_icon(&lower) {
                        has_content_images = true;
                    }
                }
            }
        }
    }

    has_ini || (!has_subdirs && has_content_images)
}

fn count_direct_leaf_mod_dirs(path: &Path) -> u64 {
    let mut count = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let sub_path = entry.path();
            if !sub_path.is_dir() {
                continue;
            }

            let sub_name = sub_path.file_name().unwrap_or_default().to_string_lossy();
            if sub_name.starts_with('.') || sub_name.starts_with('$') {
                continue;
            }

            if is_leaf_mod_dir(&sub_path) {
                count += 1;
            }
        }
    }
    count
}

fn analyze_directory(path: &Path) -> (Vec<String>, bool, bool, bool, Option<String>) {
    let mut images = Vec::new();
    let mut has_ini = false;
    let mut has_subdirs = false;
    let mut has_content_images = false;
    let mut icon_path: Option<String> = None;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let sub_path = entry.path();
            if sub_path.is_dir() {
                let sub_name = sub_path.file_name().unwrap_or_default().to_string_lossy();
                if !sub_name.starts_with('.') && !sub_name.starts_with('$') {
                    has_subdirs = true;
                }
                continue;
            }

            if !sub_path.is_file() {
                continue;
            }

            if let Some(name) = sub_path.file_name().and_then(|n| n.to_str()) {
                let lower = name.to_lowercase();
                if lower.ends_with(".ini") {
                    has_ini = true;
                }
                if is_content_image_file(&lower) {
                    let image_path = sub_path.to_string_lossy().replace("\\", "/");
                    images.push(image_path.clone());
                    if is_standard_group_icon(&lower) {
                        icon_path = Some(image_path);
                    } else {
                        has_content_images = true;
                    }
                }
            }
        }
    }

    images.sort();
    (images, has_ini, has_subdirs, has_content_images, icon_path)
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub mods: Vec<ModInfo>,
    pub groups: Vec<GroupInfo>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModKeyInfo {
    pub source_ini: String,
    pub key_name: String,
    pub back_name: String,
    pub key_type: String,
    pub value_summary: String,
    pub condition_summary: String,
}

fn strip_disabled_prefix(name: &str) -> (String, bool) {
    let upper = name.to_uppercase();
    if upper.starts_with("DISABLED_") {
        (name[9..].to_string(), true)
    } else if upper.starts_with("DISABLED ") {
        (name[9..].to_string(), true)
    } else if upper.starts_with("DISABLED") {
        (name[8..].to_string(), true)
    } else {
        (name.to_string(), false)
    }
}

fn archive_file_name_is_valid(name: &str) -> bool {
    let trimmed = name.trim();
    !trimmed.is_empty()
        && !trimmed.ends_with('.')
        && !trimmed.ends_with(' ')
        && !trimmed
            .chars()
            .any(|c| matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
}

fn mod_path_segment_candidates(segment: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut seen = HashSet::new();
    let mut push = |value: String| {
        let key = value.to_lowercase();
        if seen.insert(key) {
            values.push(value);
        }
    };

    push(segment.to_string());
    let (clean, disabled) = strip_disabled_prefix(segment);
    if disabled {
        push(clean);
    } else {
        push(format!("DISABLED_{}", segment));
        push(format!("DISABLED {}", segment));
    }

    values
}

fn find_mod_dir_by_clean_tail(root: &Path, tail: &str) -> Option<PathBuf> {
    let wanted = strip_disabled_prefix(tail).0.to_lowercase();
    if wanted.is_empty() {
        return None;
    }

    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name.starts_with('$') {
                continue;
            }

            let clean = strip_disabled_prefix(&name).0;
            if clean.eq_ignore_ascii_case(&wanted) && is_leaf_mod_dir(&path) {
                return Some(path);
            }

            stack.push(path);
        }
    }

    None
}

fn resolve_existing_mod_dir(install_dir: &str, mod_relative_path: &str) -> Result<PathBuf, String> {
    let root = mods_root(install_dir);
    let normalized = mod_relative_path
        .replace('\\', "/")
        .trim_matches('/')
        .to_string();
    if normalized.is_empty() {
        return Err("Mod relative path is empty".to_string());
    }

    let direct = root.join(normalized.replace('/', std::path::MAIN_SEPARATOR_STR));
    if direct.is_dir() {
        return Ok(direct);
    }

    let segments: Vec<&str> = normalized
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();
    let mut candidates = vec![root.clone()];
    for segment in segments {
        let variants = mod_path_segment_candidates(segment);
        let mut next = Vec::new();
        for base in &candidates {
            for variant in &variants {
                next.push(base.join(variant));
            }
        }
        candidates = next;
    }

    for candidate in candidates {
        if candidate.is_dir() {
            return Ok(candidate);
        }
    }

    if let Some(tail) = normalized
        .split('/')
        .filter(|segment| !segment.is_empty())
        .last()
    {
        if let Some(found) = find_mod_dir_by_clean_tail(&root, tail) {
            return Ok(found);
        }
    }

    Err(format!(
        "Mod source directory not found. Tried relative path '{}' under '{}'. The Mod list may be stale; refresh Mods and try again.",
        mod_relative_path,
        root.to_string_lossy()
    ))
}

fn collect_ini_files_recursive(path: &Path, output: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };

    for entry in entries.flatten() {
        let current_path = entry.path();
        if current_path.is_dir() {
            collect_ini_files_recursive(&current_path, output);
            continue;
        }

        let Some(name) = current_path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };

        if name.to_ascii_lowercase().ends_with(".ini") {
            output.push(current_path);
        }
    }
}

fn build_condition_summary(condition: &Condition) -> String {
    if condition.condition_expression_list.is_empty() {
        return String::new();
    }

    if condition.logic_list.is_empty() {
        let expr = &condition.condition_expression_list[0];
        return format!("{} == {}", expr.var_name, expr.var_value);
    }

    let mut segments = Vec::new();
    for (index, expr) in condition.condition_expression_list.iter().enumerate() {
        if index > 0 {
            let logic = condition
                .logic_list
                .get(index - 1)
                .cloned()
                .unwrap_or_else(|| "&&".to_string());
            segments.push(logic);
        }
        segments.push(format!("{} == {}", expr.var_name, expr.var_value));
    }

    segments.join(" ")
}

fn build_value_summary(key: &Key) -> String {
    let mut segments: Vec<String> = Vec::new();

    let mut active_entries: Vec<_> = key.active_variable_name_active_value_map.iter().collect();
    active_entries.sort_by(|left, right| left.0.cmp(right.0));
    for (name, value) in active_entries {
        segments.push(format!("{} = {}", name, value));
    }

    let mut cycle_entries: Vec<_> = key
        .cycle_variable_name_possible_value_list_map
        .iter()
        .collect();
    cycle_entries.sort_by(|left, right| left.0.cmp(right.0));
    for (name, values) in cycle_entries {
        segments.push(format!("{} = {}", name, values.join(", ")));
    }

    segments.join(" | ")
}

#[tauri::command]
pub async fn scan_directory(
    install_dir: String,
    relative_path: String,
) -> Result<ScanResult, String> {
    let mods_root = mods_root(&install_dir);

    // Construct target path: mods_root + relative_path (relative_path can be empty or "Root")
    // If relative_path is "Root", treat as empty
    let target_sub = if relative_path == "Root" {
        "".to_string()
    } else {
        relative_path.clone()
    };

    let target_path = mods_root.join(&target_sub);

    if !target_path.exists() {
        return Ok(ScanResult {
            mods: vec![],
            groups: vec![],
        });
    }

    let mut mods = Vec::new();
    let mut groups = Vec::new();

    let entries = fs::read_dir(&target_path).map_err(|e| e.to_string())?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if dir_name.starts_with('.') || dir_name.starts_with('$') {
            continue;
        }

        let (clean_name, is_disabled) = strip_disabled_prefix(&dir_name);
        let enabled = !is_disabled;
        let (images, has_ini, has_subdirs, has_content_images, icon_path) =
            analyze_directory(&path);
        let is_leaf_mod = has_ini || (!has_subdirs && has_content_images);
        let entry_relative_path = if target_sub.is_empty() {
            dir_name.clone()
        } else {
            format!("{}/{}", target_sub, dir_name)
        };

        if is_leaf_mod {
            mods.push(ModInfo {
                id: entry_relative_path.clone(),
                name: clean_name,
                enabled,
                path: path.to_string_lossy().replace("\\", "/"),
                relative_path: entry_relative_path.clone(),
                preview_images: images,
                group: if target_sub.is_empty() {
                    "Root".to_string()
                } else {
                    target_sub.replace("\\", "/")
                },
                is_dir: true,
                last_modified: modified_unix_seconds(&path),
            });
        } else {
            let mod_count = count_direct_leaf_mod_dirs(&path);
            groups.push(GroupInfo {
                id: entry_relative_path.clone(),
                name: clean_name,
                icon_path,
                path: entry_relative_path,
                enabled,
                mod_count,
            });
        }
    }

    mods.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    groups.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(ScanResult { mods, groups })
}

#[tauri::command]
pub async fn get_mod_key_list(
    install_dir: String,
    mod_relative_path: String,
) -> Result<Vec<ModKeyInfo>, String> {
    let mod_path = mods_root(&install_dir).join(&mod_relative_path);
    if !mod_path.exists() {
        return Err(format!(
            "Mod directory not found at: {}",
            mod_path.display()
        ));
    }

    let mut ini_files = Vec::new();
    collect_ini_files_recursive(&mod_path, &mut ini_files);
    ini_files.sort_by(|left, right| left.to_string_lossy().cmp(&right.to_string_lossy()));

    let mut result = Vec::new();
    for ini_file in ini_files {
        let container = MigotoIniContainer::from_ini_file(&ini_file)?;
        let source_ini = ini_file
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_string();

        for key in container.global_m_key_list {
            let value_summary = build_value_summary(&key);
            if value_summary.is_empty() {
                continue;
            }

            result.push(ModKeyInfo {
                source_ini: source_ini.clone(),
                key_name: key.key_name,
                back_name: key.back_name,
                key_type: key.key_type,
                value_summary,
                condition_summary: build_condition_summary(&key.condition),
            });
        }
    }

    result.sort_by(|left, right| {
        left.source_ini
            .cmp(&right.source_ini)
            .then_with(|| left.key_name.cmp(&right.key_name))
            .then_with(|| left.back_name.cmp(&right.back_name))
    });

    Ok(result)
}

#[tauri::command]
pub async fn watch_mods(
    app: AppHandle,
    state: State<'_, ModWatcher>,
    install_dir: String,
) -> Result<(), String> {
    let mods_dir = mods_root(&install_dir);

    if !mods_dir.exists() {
        return Err(format!("Mods directory not found at: {:?}", mods_dir));
    }

    // Stop existing watcher
    let mut watcher_guard = state.0.lock().unwrap();
    if let Some(_) = *watcher_guard {
        // Drop old watcher
        *watcher_guard = None;
    }

    let app_handle = app.clone();

    // Config: Poll every 2 seconds if native events fail, but usage of default() usually implies native.
    // For Windows, default is ReadDirectoryChangesW which is instant.
    // We can add a small delay to debounce at the source if needed, but notify v6 handles this differently.
    // We will just emit raw events and let frontend debounce.

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            match res {
                Ok(_event) => {
                    // Filter for relevant events if needed, but 'refresh' is safe for all
                    // println!("File event: {:?}", event);
                    // Send event to all windows
                    let _ = app_handle.emit("mod-filesystem-changed", ());
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        },
        Config::default(),
    )
    .map_err(|e| format!("Failed to create watcher: {}", e))?;

    // Watch recursively
    watcher
        .watch(&mods_dir, RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to start watch: {}", e))?;

    // Store watcher
    *watcher_guard = Some(watcher);

    println!("[ModWatcher] Started watching: {:?}", mods_dir);

    Ok(())
}

#[tauri::command]
pub fn unwatch_mods(state: State<'_, ModWatcher>) -> Result<(), String> {
    let mut watcher_guard = state.0.lock().unwrap();
    *watcher_guard = None;
    println!("[ModWatcher] Stopped watching");
    Ok(())
}

#[derive(Debug, Serialize, Clone)]
pub struct InstallProgressPayload {
    pub game_name: String,
    pub mod_name: String,
    pub stage: String,
    pub current: u64,
    pub total: u64,
}

fn emit_install_progress(app: &AppHandle, payload: InstallProgressPayload) {
    let _ = app.emit("mod-install-progress", payload);
}

fn count_files_recursive(dir: &Path) -> Result<u64, String> {
    let mut total = 0u64;
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        let entries = fs::read_dir(&current).map_err(|e| {
            format!(
                "Failed to read directory {}: {}",
                current.to_string_lossy(),
                e
            )
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                total += 1;
            }
        }
    }

    Ok(total)
}

fn preview_mod_folder(path: &Path) -> Result<ArchivePreview, String> {
    let mut root_dirs = Vec::new();
    let mut has_ini = false;

    let direct_entries = fs::read_dir(path)
        .map_err(|e| format!("Failed to read folder {}: {}", path.to_string_lossy(), e))?;
    for entry in direct_entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            if let Some(name) = entry_path.file_name().and_then(|s| s.to_str()) {
                root_dirs.push(name.to_string());
            }
        }
    }
    root_dirs.sort();

    let file_count = count_files_recursive(path)? as usize;

    let mut stack = vec![path.to_path_buf()];
    'outer: while let Some(current) = stack.pop() {
        let entries = fs::read_dir(&current).map_err(|e| {
            format!(
                "Failed to read directory {}: {}",
                current.to_string_lossy(),
                e
            )
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else if p
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("ini"))
                .unwrap_or(false)
            {
                has_ini = true;
                break 'outer;
            }
        }
    }

    Ok(ArchivePreview {
        root_dirs,
        file_count,
        has_ini,
        format: "folder".to_string(),
    })
}

fn is_ignorable_install_entry(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };

    name == "__MACOSX" || name == ".DS_Store"
}

fn resolve_install_source_dir(source_dir: &Path) -> Result<PathBuf, String> {
    let entries = fs::read_dir(source_dir).map_err(|e| {
        format!(
            "Failed to read folder {}: {}",
            source_dir.to_string_lossy(),
            e
        )
    })?;

    let mut effective_entries = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if is_ignorable_install_entry(&path) {
            continue;
        }
        effective_entries.push(path);
    }

    if effective_entries.len() == 1 && effective_entries[0].is_dir() {
        return Ok(effective_entries[0].clone());
    }

    Ok(source_dir.to_path_buf())
}

fn copy_folder_contents_with_progress<F>(
    source_dir: &Path,
    dest_dir: &Path,
    mut on_progress: F,
) -> Result<ExtractResult, String>
where
    F: FnMut(u64, u64),
{
    let total_files = count_files_recursive(source_dir)?;
    let mut processed = 0u64;

    fn copy_recursive<F>(
        src: &Path,
        dst: &Path,
        processed: &mut u64,
        total: u64,
        on_progress: &mut F,
    ) -> Result<(), String>
    where
        F: FnMut(u64, u64),
    {
        fs::create_dir_all(dst).map_err(|e| {
            format!(
                "Failed to create directory {}: {}",
                dst.to_string_lossy(),
                e
            )
        })?;

        let entries = fs::read_dir(src)
            .map_err(|e| format!("Failed to read directory {}: {}", src.to_string_lossy(), e))?;

        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                copy_recursive(&src_path, &dst_path, processed, total, on_progress)?;
            } else if src_path.is_file() {
                fs::copy(&src_path, &dst_path).map_err(|e| {
                    format!(
                        "Failed to copy file {} to {}: {}",
                        src_path.to_string_lossy(),
                        dst_path.to_string_lossy(),
                        e
                    )
                })?;

                *processed += 1;
                on_progress(*processed, total.max(1));
            }
        }

        Ok(())
    }

    copy_recursive(
        source_dir,
        dest_dir,
        &mut processed,
        total_files,
        &mut on_progress,
    )?;

    Ok(ExtractResult {
        processed,
        total: total_files,
    })
}

fn normalize_install_relative_path(
    value: &str,
    field_name: &str,
    allow_nested: bool,
    allow_empty: bool,
) -> Result<PathBuf, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        if allow_empty {
            return Ok(PathBuf::new());
        }
        return Err(format!("{} cannot be empty", field_name));
    }

    let normalized = trimmed.replace('\\', "/");
    if normalized.starts_with('/') || normalized.chars().nth(1) == Some(':') {
        return Err(format!("{} must be a relative path", field_name));
    }

    let mut output = PathBuf::new();
    let mut segment_count = 0usize;
    for segment in normalized.split('/') {
        let segment = segment.trim();
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." || segment.contains(':') {
            return Err(format!("{} contains an unsafe path segment", field_name));
        }
        segment_count += 1;
        if !allow_nested && segment_count > 1 {
            return Err(format!("{} cannot contain path separators", field_name));
        }
        output.push(segment);
    }

    if segment_count == 0 {
        if allow_empty {
            return Ok(PathBuf::new());
        }
        return Err(format!("{} cannot be empty", field_name));
    }

    Ok(output)
}

fn create_install_staging_dir(staging_parent: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(staging_parent).map_err(|e| {
        format!(
            "Failed to create install parent {}: {}",
            staging_parent.to_string_lossy(),
            e
        )
    })?;

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);

    for attempt in 0..20 {
        let staging_dir = staging_parent.join(format!(
            ".ssmt-installing-{}-{}-{}",
            std::process::id(),
            millis,
            attempt
        ));
        if staging_dir.exists() {
            continue;
        }
        fs::create_dir(&staging_dir).map_err(|e| {
            format!(
                "Failed to create install staging directory {}: {}",
                staging_dir.to_string_lossy(),
                e
            )
        })?;
        return Ok(staging_dir);
    }

    Err("Failed to allocate install staging directory".to_string())
}

fn remove_install_staging_dir(staging_dir: &Path) {
    if staging_dir.exists() {
        let _ = fs::remove_dir_all(staging_dir);
    }
}

#[cfg(test)]
mod install_path_tests {
    use super::normalize_install_relative_path;
    use std::path::PathBuf;

    #[test]
    fn install_group_path_allows_nested_relative_groups() {
        let path = normalize_install_relative_path("Characters/Ayaka", "Target group", true, false)
            .expect("nested group should be accepted");

        assert_eq!(path, PathBuf::from("Characters").join("Ayaka"));
    }

    #[test]
    fn install_mod_name_rejects_nested_or_escape_paths() {
        assert!(normalize_install_relative_path("Ayaka/Alt", "Mod name", false, false).is_err());
        assert!(normalize_install_relative_path("../Alt", "Mod name", false, false).is_err());
        assert!(normalize_install_relative_path("C:/Alt", "Mod name", false, false).is_err());
    }
}

#[tauri::command]
pub async fn preview_mod_archive(path: String) -> Result<ArchivePreview, String> {
    let path_buf = PathBuf::from(&path);

    if path_buf.is_dir() {
        return preview_mod_folder(&path_buf);
    }

    SSMTCompressUtils::preview_archive(&path_buf)
}

#[tauri::command]
pub async fn install_mod_archive(
    app: AppHandle,
    game_name: String,
    install_dir: String,
    archive_path: String,
    target_name: String,  // User defined name for the folder
    target_group: String, // E.g. "Ayaka", or "Root"
    password: Option<String>,
) -> Result<(), String> {
    let mods_dir = mods_root(&install_dir);
    let target_name_path = normalize_install_relative_path(&target_name, "Mod name", false, false)?;
    let target_group_trimmed = target_group.trim();
    let target_group_path =
        if target_group_trimmed.eq_ignore_ascii_case("Root") || target_group_trimmed.is_empty() {
            PathBuf::new()
        } else {
            normalize_install_relative_path(target_group_trimmed, "Target group", true, false)?
        };
    let target_parent = mods_dir.join(target_group_path);
    let dest_dir = target_parent.join(&target_name_path);

    if dest_dir.exists() {
        return Err("该分类下已存在同名 Mod，请更改名称后重试".to_string());
    }

    let path_buf = PathBuf::from(&archive_path);
    if !path_buf.exists() {
        return Err(format!(
            "Source path not found: {}",
            path_buf.to_string_lossy()
        ));
    }

    let ext = path_buf
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();

    if !path_buf.is_dir() && ext != "zip" && ext != "7z" && ext != "rar" {
        return Err("Unsupported format".to_string());
    }

    emit_install_progress(
        &app,
        InstallProgressPayload {
            game_name: game_name.clone(),
            mod_name: target_name.clone(),
            stage: "analyzing".to_string(),
            current: 0,
            total: 0,
        },
    );

    let staging_parent = mods_dir.join(".ssmt").join("install-staging");
    let staging_dir = create_install_staging_dir(&staging_parent)?;
    let extract_result = (|| -> Result<ExtractResult, String> {
        if path_buf.is_dir() {
            let effective_source_dir = resolve_install_source_dir(&path_buf)?;
            copy_folder_contents_with_progress(
                &effective_source_dir,
                &staging_dir,
                |current, total| {
                    emit_install_progress(
                        &app,
                        InstallProgressPayload {
                            game_name: game_name.clone(),
                            mod_name: target_name.clone(),
                            stage: "copying".to_string(),
                            current,
                            total,
                        },
                    );
                },
            )
        } else if ext == "zip" {
            SSMTCompressUtils::extract_zip_archive(&path_buf, &staging_dir, |current, total| {
                emit_install_progress(
                    &app,
                    InstallProgressPayload {
                        game_name: game_name.clone(),
                        mod_name: target_name.clone(),
                        stage: "extracting".to_string(),
                        current,
                        total,
                    },
                );
            })
        } else if ext == "7z" {
            SSMTCompressUtils::extract_7z_archive(&path_buf, &staging_dir, |current, total| {
                emit_install_progress(
                    &app,
                    InstallProgressPayload {
                        game_name: game_name.clone(),
                        mod_name: target_name.clone(),
                        stage: "extracting".to_string(),
                        current,
                        total,
                    },
                );
            })
        } else if ext == "rar" {
            SSMTCompressUtils::extract_rar_archive_with_password(
                &path_buf,
                &staging_dir,
                password.as_deref(),
                |current, total| {
                    emit_install_progress(
                        &app,
                        InstallProgressPayload {
                            game_name: game_name.clone(),
                            mod_name: target_name.clone(),
                            stage: "extracting".to_string(),
                            current,
                            total,
                        },
                    );
                },
            )
        } else {
            Err("Unsupported format".to_string())
        }
    })();

    let extract_result = match extract_result {
        Ok(value) => value,
        Err(error) => {
            remove_install_staging_dir(&staging_dir);
            return Err(error);
        }
    };

    if dest_dir.exists() {
        remove_install_staging_dir(&staging_dir);
        return Err("该分类下已存在同名 Mod，请更改名称后重试".to_string());
    }

    fs::create_dir_all(&target_parent).map_err(|e| {
        remove_install_staging_dir(&staging_dir);
        format!(
            "Failed to create install target parent {}: {}",
            target_parent.to_string_lossy(),
            e
        )
    })?;

    fs::rename(&staging_dir, &dest_dir).map_err(|e| {
        remove_install_staging_dir(&staging_dir);
        format!(
            "Failed to finalize install {} -> {}: {}",
            staging_dir.to_string_lossy(),
            dest_dir.to_string_lossy(),
            e
        )
    })?;

    let ExtractResult {
        processed: processed_entries,
        total: total_entries,
    } = extract_result;

    // Final done event
    emit_install_progress(
        &app,
        InstallProgressPayload {
            game_name,
            mod_name: target_name,
            stage: "done".to_string(),
            current: processed_entries.max(total_entries),
            total: total_entries.max(1),
        },
    );

    Ok(())
}

#[tauri::command]
pub async fn export_mod_archive(
    install_dir: String,
    mod_relative_path: String,
    output_dir: String,
    archive_name: String,
    format: String,
    password: Option<String>,
) -> Result<String, String> {
    let archive_name = archive_name.trim();
    if !archive_file_name_is_valid(archive_name) {
        return Err("Archive name contains invalid characters".to_string());
    }

    let format = format.trim().to_lowercase();
    if !matches!(format.as_str(), "zip" | "7z" | "rar") {
        return Err(format!("Unsupported archive format: {}", format));
    }

    let source_dir = resolve_existing_mod_dir(&install_dir, &mod_relative_path)?;
    let output_dir = PathBuf::from(output_dir);
    fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;
    let output_path = output_dir.join(format!("{}.{}", archive_name, format));

    SSMTCompressUtils::create_mod_archive(&source_dir, &output_path, &format, password.as_deref())?;

    Ok(output_path.to_string_lossy().replace('\\', "/"))
}
