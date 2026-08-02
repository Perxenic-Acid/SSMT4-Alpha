use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use tauri::AppHandle;

#[derive(Debug, Deserialize)]
struct DrawIBEntry {
    #[serde(rename = "DrawIB")]
    draw_ib: String,
}

#[derive(Debug, Deserialize)]
struct SkipIBEntry {
    #[serde(rename = "SkipIB")]
    skip_ib: String,

    #[serde(rename = "IndexCount", default)]
    _index_count: String,

    #[serde(rename = "FirstIndex", default)]
    _first_index: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct VSCheckEntry {
    #[serde(rename = "Enabled")]
    enabled: bool,
    #[serde(rename = "Hash")]
    hash: String,
}

fn normalize_hash(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn parse_vs_hash_from_filename(file_name: &str) -> Option<(String, String)> {
    if !file_name.ends_with(".txt") {
        return None;
    }

    let vs_index = file_name.find("-vs=")?;
    if vs_index < 8 || file_name.len() < vs_index + 4 + 16 {
        return None;
    }

    let buffer_hash = normalize_hash(&file_name[vs_index - 8..vs_index]);
    let vs_shader_hash = normalize_hash(&file_name[vs_index + 4..vs_index + 20]);

    if buffer_hash.is_empty() || vs_shader_hash.is_empty() {
        return None;
    }

    Some((buffer_hash, vs_shader_hash))
}

fn collect_vs_hashes_recursive(
    dir: &Path,
    dict: &mut HashMap<String, Vec<String>>,
) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }

    let entries = fs::read_dir(dir).map_err(|e| {
        format!(
            "Failed to read frame analysis folder {}: {}",
            dir.display(),
            e
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.is_dir() {
            collect_vs_hashes_recursive(&path, dict)?;
            continue;
        }

        if !path.is_file() {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        let Some((buffer_hash, vs_shader_hash)) = parse_vs_hash_from_filename(file_name) else {
            continue;
        };

        let hash_list = dict.entry(buffer_hash).or_default();
        if !hash_list.contains(&vs_shader_hash) {
            hash_list.push(vs_shader_hash);
        }
    }

    Ok(())
}

fn get_buff_hash_vs_shader_hash_values_dict(
    frame_analysis_folder_path: &str,
) -> Result<HashMap<String, Vec<String>>, String> {
    let mut dict: HashMap<String, Vec<String>> = HashMap::new();
    collect_vs_hashes_recursive(Path::new(frame_analysis_folder_path), &mut dict)?;
    Ok(dict)
}

fn read_draw_ib_list_from_workspace(workspace_path: &str) -> Result<Vec<String>, String> {
    let path = Path::new(workspace_path).join("Config.json");
    if !path.exists() {
        return Ok(Vec::new());
    }

    let s = fs::read_to_string(&path).map_err(|e| format!("Failed to read Config.json: {}", e))?;
    let list: Vec<DrawIBEntry> =
        serde_json::from_str(&s).map_err(|e| format!("Failed to parse Config.json: {}", e))?;

    Ok(list
        .into_iter()
        .map(|x| normalize_hash(&x.draw_ib))
        .filter(|x| !x.is_empty())
        .collect())
}

fn read_skip_ib_list_from_workspace(workspace_path: &str) -> Result<Vec<String>, String> {
    let path = Path::new(workspace_path).join("SkipIBConfig.json");
    if !path.exists() {
        return Ok(Vec::new());
    }

    let s = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read SkipIBConfig.json: {}", e))?;
    let list: Vec<SkipIBEntry> = serde_json::from_str(&s)
        .map_err(|e| format!("Failed to parse SkipIBConfig.json: {}", e))?;

    Ok(list
        .into_iter()
        .map(|x| normalize_hash(&x.skip_ib))
        .filter(|x| !x.is_empty())
        .collect())
}

fn read_vscheck_list_from_workspace(workspace_path: &str) -> Result<Vec<VSCheckEntry>, String> {
    let path = Path::new(workspace_path).join("VSCheckConfig.json");
    if !path.exists() {
        return Ok(Vec::new());
    }

    let s = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read VSCheckConfig.json: {}", e))?;
    let list: Vec<VSCheckEntry> = serde_json::from_str(&s)
        .map_err(|e| format!("Failed to parse VSCheckConfig.json: {}", e))?;
    Ok(list)
}

fn save_vscheck_list_to_workspace(
    workspace_path: &str,
    list: &[VSCheckEntry],
) -> Result<(), String> {
    let ws_dir = Path::new(workspace_path);
    fs::create_dir_all(ws_dir).map_err(|e| format!("Failed to create workspace dir: {}", e))?;

    let config_path = ws_dir.join("VSCheckConfig.json");
    if list.is_empty() {
        if config_path.exists() {
            fs::remove_file(&config_path)
                .map_err(|e| format!("Failed to remove VSCheckConfig.json: {}", e))?;
        }
        return Ok(());
    }

    let json = serde_json::to_string_pretty(list)
        .map_err(|e| format!("Failed to serialize VSCheckConfig.json: {}", e))?;
    fs::write(&config_path, json).map_err(|e| format!("Failed to write VSCheckConfig.json: {}", e))
}

#[tauri::command]
pub async fn update_vscheck(
    _app: AppHandle,
    frame_analysis_folder: String,
    game_preset: String,
    workspace_path: String,
) -> Result<(), String> {
    println!("开始更新 VSCheck:");
    println!("FrameAnalysis 文件夹路径: {}", frame_analysis_folder);
    println!("当前游戏预设: {}", game_preset);
    println!("当前工作空间路径: {}", workspace_path);

    let mut total_ib_list = read_skip_ib_list_from_workspace(&workspace_path)?;
    total_ib_list.extend(read_draw_ib_list_from_workspace(&workspace_path)?);

    let total_ib_hash_set: HashSet<String> = total_ib_list
        .into_iter()
        .filter(|hash| !hash.is_empty())
        .collect();

    let buff_hash_vs_shader_hash_values_dict =
        get_buff_hash_vs_shader_hash_values_dict(&frame_analysis_folder)?;

    let mut total_vs_hash_list: Vec<String> = Vec::new();
    for draw_ib in total_ib_hash_set.iter() {
        if let Some(vs_hash_list) = buff_hash_vs_shader_hash_values_dict.get(draw_ib) {
            for hash in vs_hash_list {
                if !total_vs_hash_list.contains(hash) {
                    total_vs_hash_list.push(hash.clone());
                }
            }
        }
    }

    let current_vscheck_items = read_vscheck_list_from_workspace(&workspace_path)?;
    let mut current_vscheck_map: HashMap<String, VSCheckEntry> = HashMap::new();
    let mut current_vs_hash_order: Vec<String> = Vec::new();
    for item in current_vscheck_items {
        let normalized_hash = normalize_hash(&item.hash);
        if normalized_hash.is_empty() {
            continue;
        }

        if !current_vscheck_map.contains_key(&normalized_hash) {
            current_vs_hash_order.push(normalized_hash.clone());
        }

        current_vscheck_map.insert(
            normalized_hash.clone(),
            VSCheckEntry {
                enabled: item.enabled,
                hash: item.hash.trim().to_string(),
            },
        );
    }

    let mut new_vscheck_item_list: Vec<VSCheckEntry> = Vec::new();
    for normalized_hash in current_vs_hash_order {
        if let Some(existing_item) = current_vscheck_map.get(&normalized_hash) {
            new_vscheck_item_list.push(existing_item.clone());
        }
    }

    for hash in total_vs_hash_list {
        if !current_vscheck_map.contains_key(&hash) {
            new_vscheck_item_list.push(VSCheckEntry {
                enabled: true,
                hash,
            });
        }
    }

    save_vscheck_list_to_workspace(&workspace_path, &new_vscheck_item_list)?;

    Ok(())
}

#[tauri::command]
pub async fn generate_vscheck(
    _app: AppHandle,
    frame_analysis_folder: String,
    game_preset: String,
    workspace_path: String,
    generated_mod_folder_path: String,
) -> Result<(), String> {
    println!("开始生成 VSCheck:");
    println!("FrameAnalysis 文件夹路径: {}", frame_analysis_folder);
    println!("当前游戏预设: {}", game_preset);
    println!("当前工作空间路径: {}", workspace_path);

    let vscheck_items = read_vscheck_list_from_workspace(&workspace_path)?;

    let mut total_vs_hash_list: Vec<String> = Vec::new();
    for item in vscheck_items {
        let hash = item.hash.trim();
        if item.enabled && !hash.is_empty() {
            total_vs_hash_list.push(hash.to_string());
        }
    }

    let mut output_content = String::new();
    let mut writed_hash_list: Vec<String> = Vec::new();

    for hash in total_vs_hash_list {
        if writed_hash_list.contains(&hash) {
            continue;
        }
        writed_hash_list.push(hash.clone());

        output_content.push_str(&format!("[ShaderOverride_{}]\r\n", hash));
        output_content.push_str("allow_duplicate_hash = overrule\r\n");
        output_content.push_str(&format!("hash = {}\r\n", hash));
        output_content.push_str("if $costume_mods\r\n");
        output_content.push_str("  checktextureoverride = ib\r\n");
        output_content.push_str("endif\r\n\r\n");
    }

    let generated_mod_folder = Path::new(&generated_mod_folder_path);

    fs::create_dir_all(generated_mod_folder)
        .map_err(|e| format!("Failed to create generated mod folder: {}", e))?;

    let vscheck_ini_path = generated_mod_folder.join("VSCheck.ini");
    fs::write(&vscheck_ini_path, output_content)
        .map_err(|e| format!("Failed to write VSCheck.ini: {}", e))?;

    Ok(())
}
