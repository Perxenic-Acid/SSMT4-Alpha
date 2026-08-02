#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(windows)]
#[link(name = "delayimp")]
unsafe extern "C" {}

const EXIT_OK: i32 = 0;
const EXIT_USAGE: i32 = 2;
const EXIT_REQUEST_READ_FAILED: i32 = 11;
const EXIT_REQUEST_PARSE_FAILED: i32 = 12;
const EXIT_NOT_ELEVATED: i32 = 13;
const EXIT_APPLY_FAILED: i32 = 20;
const EXIT_RESULT_WRITE_FAILED: i32 = 21;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WuwaSettingsRequest {
    target_exe_path: String,
    #[serde(default = "default_true")]
    configure_game: bool,
    #[serde(default)]
    apply_perf_tweaks: bool,
    #[serde(default)]
    unlock_fps: bool,
    #[serde(default)]
    force_max_lod_bias: bool,
    #[serde(default)]
    disable_wounded_fx: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WuwaSettingsResponse {
    success: bool,
    message: String,
    error: Option<String>,
}

fn default_true() -> bool {
    true
}

fn print_usage() {
    eprintln!("Usage:\n  wuwa_settings --request-json <path>");
}

fn response_path_from_request_path(request_path: &Path) -> PathBuf {
    let request_str = request_path.to_string_lossy();
    if request_str.ends_with(".json") {
        PathBuf::from(format!(
            "{}.result.json",
            request_str.trim_end_matches(".json")
        ))
    } else {
        PathBuf::from(format!("{}.result.json", request_str))
    }
}

fn write_response(response_path: &Path, response: &WuwaSettingsResponse) -> Result<(), String> {
    if let Some(parent) = response_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create {}: {}", parent.display(), error))?;
    }

    let raw = serde_json::to_string_pretty(response)
        .map_err(|error| format!("Failed to serialize response: {}", error))?;
    fs::write(response_path, raw)
        .map_err(|error| format!("Failed to write {}: {}", response_path.display(), error))
}

#[cfg(windows)]
fn is_elevated() -> Result<bool, String> {
    let output = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; if (([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { '1' } else { '0' }",
        ])
        .output()
        .map_err(|error| format!("Failed to run PowerShell elevation check: {}", error))?;

    if !output.status.success() {
        return Err(format!(
            "Elevation check failed with code {:?}. stdout: {} stderr: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim() == "1")
}

#[cfg(not(windows))]
fn is_elevated() -> Result<bool, String> {
    Err("Unsupported platform".to_string())
}

fn infer_wwmi_game_root(target_exe_path: &str) -> Result<PathBuf, String> {
    let target_path = PathBuf::from(target_exe_path);
    let mut current = target_path
        .parent()
        .ok_or_else(|| format!("Failed to resolve parent directory for {}", target_exe_path))?
        .to_path_buf();

    for _ in 0..8 {
        if current.join("Client").is_dir() && current.join("Engine").is_dir() {
            return Ok(current);
        }

        let Some(parent) = current.parent() else {
            break;
        };
        current = parent.to_path_buf();
    }

    Err(format!(
        "Failed to infer WWMI game root from target process path: {}",
        target_exe_path
    ))
}

fn normalize_ini_lines(content: &str) -> Vec<String> {
    let normalized = content.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines: Vec<String> = if normalized.is_empty() {
        Vec::new()
    } else {
        normalized
            .split('\n')
            .map(|line| line.to_string())
            .collect()
    };
    if normalized.ends_with('\n') && lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
    lines
}

fn parse_ini_key(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
        return None;
    }
    let separator_index = trimmed.find('=')?;
    Some(trimmed[..separator_index].trim().to_ascii_lowercase())
}

fn find_ini_section(lines: &[String], section_name: &str) -> Option<(usize, usize)> {
    let header = format!("[{}]", section_name).to_ascii_lowercase();
    let start = lines
        .iter()
        .position(|line| line.trim().to_ascii_lowercase() == header)?;
    let end = lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find(|(_, line)| {
            let trimmed = line.trim();
            trimmed.starts_with('[') && trimmed.ends_with(']')
        })
        .map(|(index, _)| index)
        .unwrap_or(lines.len());
    Some((start, end))
}

fn set_ini_section_values(content: &str, section_name: &str, values: &[(&str, String)]) -> String {
    let mut lines = normalize_ini_lines(content);
    let keys: HashSet<String> = values
        .iter()
        .map(|(key, _)| key.to_ascii_lowercase())
        .collect();
    let value_lines: Vec<String> = values
        .iter()
        .map(|(key, value)| format!("{}={}", key, value))
        .collect();

    let Some((start, end)) = find_ini_section(&lines, section_name) else {
        if !lines.is_empty() && lines.last().is_some_and(|line| !line.trim().is_empty()) {
            lines.push(String::new());
        }
        lines.push(format!("[{}]", section_name));
        lines.extend(value_lines);
        return format!("{}\r\n", lines.join("\r\n"));
    };

    let mut replacement = Vec::new();
    replacement.extend_from_slice(&lines[..=start]);
    replacement.extend(
        lines[start + 1..end]
            .iter()
            .filter(|line| parse_ini_key(line).is_none_or(|key| !keys.contains(&key)))
            .cloned(),
    );
    replacement.extend(value_lines);
    replacement.extend_from_slice(&lines[end..]);

    format!("{}\r\n", replacement.join("\r\n"))
}

fn remove_ini_section_keys(content: &str, section_name: &str, keys: &[&str]) -> String {
    let lines = normalize_ini_lines(content);
    let key_set: HashSet<String> = keys.iter().map(|key| key.to_ascii_lowercase()).collect();

    let Some((start, end)) = find_ini_section(&lines, section_name) else {
        return format!("{}\r\n", lines.join("\r\n"));
    };

    let mut replacement = Vec::new();
    replacement.extend_from_slice(&lines[..=start]);
    replacement.extend(
        lines[start + 1..end]
            .iter()
            .filter(|line| parse_ini_key(line).is_none_or(|key| !key_set.contains(&key)))
            .cloned(),
    );
    replacement.extend_from_slice(&lines[end..]);

    format!("{}\r\n", replacement.join("\r\n"))
}

fn read_text_if_exists(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Ok(String::new());
    }
    fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))
}

fn clear_readonly_if_needed(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    let metadata = fs::metadata(path)
        .map_err(|e| format!("Failed to read metadata for {}: {}", path.display(), e))?;
    let permissions = metadata.permissions();
    if !permissions.readonly() {
        return Ok(());
    }

    let mut writable_permissions = permissions;
    writable_permissions.set_readonly(false);
    fs::set_permissions(path, writable_permissions).map_err(|e| {
        format!(
            "Failed to clear readonly attribute for {}: {}. Try running SSMT as administrator or move the game out of a protected directory.",
            path.display(),
            e
        )
    })
}

fn write_text_if_changed(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
    }
    let previous = read_text_if_exists(path)?;
    if previous != content {
        fs::write(path, content)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    }
    Ok(())
}

fn update_wwmi_engine_ini(game_root: &Path, apply_perf_tweaks: bool) -> Result<(), String> {
    let path = game_root
        .join("Client")
        .join("Saved")
        .join("Config")
        .join("WindowsNoEditor")
        .join("Engine.ini");
    let mut content = read_text_if_exists(&path)?;

    content = remove_ini_section_keys(
        &content,
        "ConsoleVariables",
        &["r.Kuro.SkeletalMesh.DistanceLODBaseFOV"],
    );

    let perf_tweaks = [
        ("r.Streaming.HLODStrategy", "2".to_string()),
        ("r.Streaming.PoolSizeForMeshes", "-1".to_string()),
        ("r.XGEShaderCompile", "0".to_string()),
        ("FX.BatchAsync", "1".to_string()),
        ("FX.EarlyScheduleAsync", "1".to_string()),
        ("fx.Niagara.ForceAutoPooling", "1".to_string()),
        (
            "wp.Runtime.KuroRuntimeStreamingRangeOverallScale",
            "0.5".to_string(),
        ),
        ("tick.AllowAsyncTickCleanup", "1".to_string()),
        ("tick.AllowAsyncTickDispatch", "1".to_string()),
    ];

    content = if apply_perf_tweaks {
        set_ini_section_values(&content, "SystemSettings", &perf_tweaks)
    } else {
        let keys: Vec<&str> = perf_tweaks.iter().map(|(key, _)| *key).collect();
        remove_ini_section_keys(&content, "SystemSettings", &keys)
    };

    write_text_if_changed(&path, &content)
}

fn update_wwmi_user_engine_ini(game_root: &Path) -> Result<(), String> {
    let path = game_root
        .join("Client")
        .join("Config")
        .join("UserEngine.ini");
    let content = read_text_if_exists(&path)?;
    let content = set_ini_section_values(
        &content,
        "ConsoleVariables",
        &[
            ("r.Kuro.SkeletalMesh.DistanceLODBaseFOV", "165".to_string()),
            (
                "r.Kuro.SkeletalMesh.LODDistanceScaleDeviceOffset",
                "-10".to_string(),
            ),
            ("r.Streaming.Boost", "20.0".to_string()),
            ("r.Streaming.MinBoost", "0.0".to_string()),
            ("r.Streaming.UseAllMips", "1".to_string()),
            ("r.Streaming.PoolSize", "0".to_string()),
            ("r.Streaming.LimitPoolSizeToVRAM", "1".to_string()),
            ("r.Streaming.UseFixedPoolSize", "1".to_string()),
        ],
    );
    write_text_if_changed(&path, &content)
}

fn update_wwmi_game_user_settings_ini(game_root: &Path) -> Result<(), String> {
    let path = game_root
        .join("Client")
        .join("Saved")
        .join("Config")
        .join("WindowsNoEditor")
        .join("GameUserSettings.ini");
    let content = read_text_if_exists(&path)?;
    let content = set_ini_section_values(
        &content,
        "/Script/Engine.GameUserSettings",
        &[("FrameRateLimit", "120.000000".to_string())],
    );
    write_text_if_changed(&path, &content)
}

fn newest_local_storage_db(storage_dir: &Path) -> Result<Option<PathBuf>, String> {
    if !storage_dir.exists() {
        return Ok(None);
    }

    let mut newest: Option<(PathBuf, std::time::SystemTime)> = None;
    for entry in fs::read_dir(storage_dir)
        .map_err(|e| format!("Failed to read {}: {}", storage_dir.display(), e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read LocalStorage entry: {}", e))?;
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if !path.is_file()
            || !file_name.starts_with("LocalStorage")
            || path.extension().and_then(|ext| ext.to_str()) != Some("db")
        {
            continue;
        }
        let modified = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .map_err(|e| format!("Failed to read metadata for {}: {}", path.display(), e))?;
        if newest
            .as_ref()
            .is_none_or(|(_, current)| modified > *current)
        {
            newest = Some((path, modified));
        }
    }

    Ok(newest.map(|(path, _)| path))
}

fn prepare_local_storage_db(game_root: &Path) -> Result<PathBuf, String> {
    let storage_dir = game_root.join("Client").join("Saved").join("LocalStorage");
    fs::create_dir_all(&storage_dir)
        .map_err(|e| format!("Failed to create {}: {}", storage_dir.display(), e))?;

    let default_path = storage_dir.join("LocalStorage.db");
    let active_path = newest_local_storage_db(&storage_dir)?;

    for entry in fs::read_dir(&storage_dir)
        .map_err(|e| format!("Failed to read {}: {}", storage_dir.display(), e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read LocalStorage entry: {}", e))?;
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if !path.is_file()
            || !file_name.starts_with("LocalStorage")
            || path.extension().and_then(|ext| ext.to_str()) != Some("db")
        {
            continue;
        }
        if active_path.as_ref().is_none_or(|active| *active != path) {
            let _ = fs::remove_file(path.with_extension("db-journal"));
            fs::remove_file(&path)
                .map_err(|e| format!("Failed to remove {}: {}", path.display(), e))?;
        }
    }

    if let Some(active_path) = active_path {
        if active_path != default_path {
            let active_journal = active_path.with_extension("db-journal");
            if active_journal.exists() {
                fs::rename(&active_journal, default_path.with_extension("db-journal"))
                    .map_err(|e| format!("Failed to rename {}: {}", active_journal.display(), e))?;
            }
            fs::rename(&active_path, &default_path)
                .map_err(|e| format!("Failed to rename {}: {}", active_path.display(), e))?;
        }
    }

    Ok(default_path)
}

fn ensure_local_storage_schema(connection: &Connection) -> Result<(), String> {
    connection
        .execute(
            "CREATE TABLE IF NOT EXISTS LocalStorage(key text primary key not null, value text not null)",
            [],
        )
        .map_err(|e| format!("Failed to create LocalStorage table: {}", e))?;

    for (key, value) in [
        ("NotFirstTimeOpenPush", "\"___1B___\""),
        ("HasLocalGameSettings", "\"___1B___\""),
        ("IsCustomImageQuality", "\"___1B___\""),
    ] {
        connection
            .execute(
                "INSERT OR IGNORE INTO LocalStorage(key, value) VALUES (?1, ?2)",
                params![key, value],
            )
            .map_err(|e| format!("Failed to seed LocalStorage {}: {}", key, e))?;
    }
    Ok(())
}

fn set_local_storage_value(connection: &Connection, key: &str, value: &str) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO LocalStorage(key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, value],
        )
        .map_err(|e| format!("Failed to set LocalStorage {}: {}", key, e))?;
    Ok(())
}

fn reset_fps_setting(connection: &Connection) -> Result<(), String> {
    let mut stmt = connection
        .prepare("SELECT name, sql FROM sqlite_master WHERE type='trigger'")
        .map_err(|e| format!("Failed to read LocalStorage triggers: {}", e))?;
    let trigger_names: Vec<String> = stmt
        .query_map([], |row| {
            let name: String = row.get(0)?;
            let body: String = row.get(1)?;
            Ok((name, body))
        })
        .map_err(|e| format!("Failed to query LocalStorage triggers: {}", e))?
        .filter_map(|result| result.ok())
        .filter(|(_, body)| body.contains("CustomFrameRate"))
        .map(|(name, _)| name)
        .collect();

    for name in trigger_names {
        connection
            .execute(
                &format!("DROP TRIGGER IF EXISTS \"{}\"", name.replace('"', "\"\"")),
                [],
            )
            .map_err(|e| format!("Failed to drop trigger {}: {}", name, e))?;
    }
    Ok(())
}

fn set_fps_setting(connection: &Connection, fps: i32) -> Result<(), String> {
    reset_fps_setting(connection)?;
    set_local_storage_value(
        connection,
        "MenuData",
        r#"{"___MetaType___":"___Map___","Content":[[1,100],[2,100],[3,100],[4,100],[5,0],[6,0],[7,-0.4658685302734375],[10,3],[11,3],[20,0],[21,0],[22,0],[23,0],[24,0],[25,0],[26,0],[27,0],[28,0],[29,0],[30,0],[31,0],[32,0],[33,0],[34,0],[35,0],[36,0],[37,0],[38,0],[39,0],[40,0],[41,0],[42,0],[43,0],[44,0],[45,0],[46,0],[47,0],[48,0],[49,0],[50,0],[51,1],[52,1],[53,0],[54,3],[55,1],[56,2],[57,1],[58,1],[59,1],[61,0],[62,0],[63,1],[64,1],[65,0],[66,0],[67,3],[68,2],[69,100],[70,100],[79,1],[81,0],[82,1],[83,1],[84,0],[85,0],[87,0],[88,0],[89,50],[90,50],[91,50],[92,50],[93,1],[99,0],[100,30],[101,0],[102,1],[103,0],[104,50],[105,0],[106,0.3],[107,0],[112,0],[113,0],[114,0],[115,0],[116,0],[117,0],[118,0],[119,0],[120,0],[121,1],[122,1],[123,0],[130,0],[131,0],[132,1],[135,1],[133,0]]}"#,
    )?;
    set_local_storage_value(
        connection,
        "PlayMenuInfo",
        r#"{"1":100,"2":100,"3":100,"4":100,"5":0,"6":0,"7":-0.4658685302734375,"10":3,"11":3,"20":0,"21":0,"22":0,"23":0,"24":0,"25":0,"26":0,"27":0,"28":0,"29":0,"30":0,"31":0,"32":0,"33":0,"34":0,"35":0,"36":0,"37":0,"38":0,"39":0,"40":0,"41":0,"42":0,"43":0,"44":0,"45":0,"46":0,"47":0,"48":0,"49":0,"50":0,"51":1,"52":1,"53":0,"54":3,"55":1,"56":2,"57":1,"58":1,"59":1,"61":0,"62":0,"63":1,"64":1,"65":0,"66":0,"67":3,"68":2,"69":100,"70":100,"79":1,"81":0,"82":1,"83":1,"84":0,"85":0,"87":0,"88":0,"89":50,"90":50,"91":50,"92":50,"93":1,"99":0,"100":30,"101":0,"102":1,"103":0,"104":50,"105":0,"106":0.3,"107":0,"112":0,"113":0,"114":0,"115":0,"116":0,"117":0,"118":0,"119":0,"120":0,"121":1,"122":1,"123":0,"130":0,"131":0,"132":1}"#,
    )?;
    set_local_storage_value(connection, "CustomFrameRate", &fps.to_string())?;
    connection
        .execute("DROP TRIGGER IF EXISTS CustomFrameRateLock", [])
        .map_err(|e| format!("Failed to drop CustomFrameRateLock: {}", e))?;
    connection
        .execute(
            &format!(
                "CREATE TRIGGER CustomFrameRateLock
                 AFTER UPDATE OF value ON LocalStorage
                 WHEN NEW.key = 'CustomFrameRate'
                 BEGIN
                     UPDATE LocalStorage
                     SET value = {}
                     WHERE key = 'CustomFrameRate';
                 END;",
                fps
            ),
            [],
        )
        .map_err(|e| format!("Failed to create CustomFrameRateLock: {}", e))?;
    Ok(())
}

fn configure_wwmi_local_storage(
    game_root: &Path,
    options: &WuwaSettingsRequest,
) -> Result<(), String> {
    if !options.configure_game && !options.unlock_fps {
        return Ok(());
    }

    let db_path = prepare_local_storage_db(game_root)?;
    let storage_dir = db_path.parent().ok_or_else(|| {
        format!(
            "Failed to resolve LocalStorage directory for {}",
            db_path.display()
        )
    })?;
    clear_readonly_if_needed(storage_dir)?;
    clear_readonly_if_needed(&db_path)?;
    clear_readonly_if_needed(&db_path.with_extension("db-journal"))?;
    clear_readonly_if_needed(&db_path.with_extension("db-wal"))?;
    clear_readonly_if_needed(&db_path.with_extension("db-shm"))?;
    let connection = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open {}: {}", db_path.display(), e))?;
    ensure_local_storage_schema(&connection)?;

    if options.unlock_fps {
        set_fps_setting(&connection, 120)?;
    } else {
        reset_fps_setting(&connection)?;
    }

    if options.configure_game {
        if options.force_max_lod_bias {
            set_local_storage_value(&connection, "ImageDetail", "3")?;
        }
        set_local_storage_value(&connection, "RayTracing", "0")?;
        set_local_storage_value(&connection, "RayTracedReflection", "0")?;
        set_local_storage_value(&connection, "RayTracedGI", "0")?;
        set_local_storage_value(
            &connection,
            "SkinDamageMode",
            if options.disable_wounded_fx { "0" } else { "1" },
        )?;
    }

    Ok(())
}

fn run_wuwa_settings(request: &WuwaSettingsRequest) -> Result<String, String> {
    let game_root = infer_wwmi_game_root(&request.target_exe_path)?;
    println!(
        "[wuwa_settings] Configuring WWMI launch settings. game_root='{}', request={:?}",
        game_root.display(),
        request
    );

    configure_wwmi_local_storage(&game_root, request)?;
    update_wwmi_engine_ini(&game_root, request.apply_perf_tweaks)?;
    update_wwmi_user_engine_ini(&game_root)?;
    if request.unlock_fps {
        update_wwmi_game_user_settings_ini(&game_root)?;
    }

    Ok(format!(
        "Configured WWMI launch settings successfully for {}",
        game_root.display()
    ))
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(flag) = args.first().map(String::as_str) else {
        print_usage();
        std::process::exit(EXIT_USAGE);
    };

    if flag != "--request-json" || args.len() < 2 {
        print_usage();
        std::process::exit(EXIT_USAGE);
    }

    let request_path = PathBuf::from(&args[1]);
    let response_path = response_path_from_request_path(&request_path);

    let request_raw = match fs::read_to_string(&request_path) {
        Ok(raw) => raw,
        Err(error) => {
            let response = WuwaSettingsResponse {
                success: false,
                message: "Failed to read wuwa settings request".to_string(),
                error: Some(format!(
                    "Failed to read {}: {}",
                    request_path.display(),
                    error
                )),
            };
            let _ = write_response(&response_path, &response);
            std::process::exit(EXIT_REQUEST_READ_FAILED);
        }
    };

    let request: WuwaSettingsRequest = match serde_json::from_str(&request_raw) {
        Ok(request) => request,
        Err(error) => {
            let response = WuwaSettingsResponse {
                success: false,
                message: "Failed to parse wuwa settings request".to_string(),
                error: Some(format!("Invalid request JSON: {}", error)),
            };
            let _ = write_response(&response_path, &response);
            std::process::exit(EXIT_REQUEST_PARSE_FAILED);
        }
    };

    match is_elevated() {
        Ok(true) => {}
        Ok(false) => {
            let response = WuwaSettingsResponse {
                success: false,
                message: "wuwa_settings must run as administrator".to_string(),
                error: Some(
                    "Administrator privileges are required to modify WWMI ini/db settings"
                        .to_string(),
                ),
            };
            let _ = write_response(&response_path, &response);
            std::process::exit(EXIT_NOT_ELEVATED);
        }
        Err(error) => {
            let response = WuwaSettingsResponse {
                success: false,
                message: "Failed to verify administrator privileges".to_string(),
                error: Some(error),
            };
            let _ = write_response(&response_path, &response);
            std::process::exit(EXIT_NOT_ELEVATED);
        }
    }

    let response = match run_wuwa_settings(&request) {
        Ok(message) => WuwaSettingsResponse {
            success: true,
            message,
            error: None,
        },
        Err(error) => WuwaSettingsResponse {
            success: false,
            message: "Failed to apply WWMI launch settings".to_string(),
            error: Some(error),
        },
    };

    let exit_code = if response.success {
        EXIT_OK
    } else {
        EXIT_APPLY_FAILED
    };

    if let Err(error) = write_response(&response_path, &response) {
        eprintln!("[wuwa_settings] {}", error);
        std::process::exit(EXIT_RESULT_WRITE_FAILED);
    }

    std::process::exit(exit_code);
}
