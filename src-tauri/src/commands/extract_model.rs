use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::Instant;
use tauri::AppHandle;

use crate::common::frame_analysis::frameanalysis::FrameAnalysis;
use crate::common::index_buffer_txt_file::IndexBufferTxtFile;
use crate::constants::gametype::ExtractTechnique;
use crate::extract_new::extract_services::{ExtractNewService, FullExtractDataTypeFilter};
use crate::helper::mark_texture_helper::MarkTextureHelper;

fn resolve_lod_workspace_path(workspace_root_path: &str, lod_name: &str) -> Result<String, String> {
    let trimmed_workspace_root_path = workspace_root_path.trim();
    if trimmed_workspace_root_path.is_empty() {
        return Err("workspace_root_path is empty".to_string());
    }

    let trimmed_lod_name = lod_name.trim();
    if trimmed_lod_name.is_empty() {
        return Err("lod_name is empty".to_string());
    }

    Ok(PathBuf::from(trimmed_workspace_root_path)
        .join(trimmed_lod_name)
        .to_string_lossy()
        .to_string())
}

/// Rebuild the derived component map from the extracted folders already on disk.
/// This deliberately does not consult Import.json: that file is Blender's
/// selected-data-type state, not the set of extracted/importable components.
#[tauri::command]
pub fn regenerate_draw_ib_component_json(lod_workspace_path: String) -> Result<(), String> {
    let lod_workspace_path = lod_workspace_path.trim();
    if lod_workspace_path.is_empty() {
        return Err("lod_workspace_path is empty".to_string());
    }

    let lod_path = PathBuf::from(lod_workspace_path);
    if !lod_path.is_dir() {
        return Err(format!(
            "LOD workspace path does not exist: {}",
            lod_workspace_path
        ));
    }

    let repaired_count = MarkTextureHelper::reset_gimi_vertex_ranges_for_import(lod_workspace_path)?;
    if repaired_count > 0 {
        println!(
            "Restored {} GIMI submesh JSON files to full-buffer import semantics",
            repaired_count
        );
    }
    MarkTextureHelper::generate_draw_ib_component_json(lod_workspace_path);
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DrawIBSubmeshRange {
    #[serde(rename = "firstIndex")]
    pub first_index: String,
    #[serde(rename = "indexCount")]
    pub index_count: String,
}

#[tauri::command]
pub async fn extract_models_new(
    _app: AppHandle,
    frame_analysis_folder: String,
    game_preset: String,
    workspace_root_path: String,
    lod_name: String,
) -> Result<(), String> {
    let lod_workspace_path = resolve_lod_workspace_path(&workspace_root_path, &lod_name)?;

    if game_preset == "ZZMIDX12" {
        return ExtractNewService::run_extract_dx12(
            &frame_analysis_folder,
            &lod_workspace_path,
            FullExtractDataTypeFilter::All,
            false,
        );
    }

    let fa = FrameAnalysis::new(&frame_analysis_folder)
        .map_err(|e| format!("Failed to read FrameAnalysis data: {}", e))?;

    ExtractNewService::run_extract(
        &fa,
        &game_preset,
        &workspace_root_path,
        &lod_workspace_path,
        FullExtractDataTypeFilter::All,
        false,
    )?;

    Ok(())
}

#[tauri::command]
pub async fn full_extract(
    _app: AppHandle,
    frame_analysis_folder: String,
    game_preset: String,
    workspace_root_path: String,
    lod_name: String,
    data_type_filter: String,
) -> Result<(), String> {
    let start_time = Instant::now();

    let data_type_filter = FullExtractDataTypeFilter::from_input(&data_type_filter)?;

    let lod_workspace_path = resolve_lod_workspace_path(&workspace_root_path, &lod_name)?;

    if game_preset == "ZZMIDX12" {
        let result = ExtractNewService::run_extract_dx12(
            &frame_analysis_folder,
            &lod_workspace_path,
            data_type_filter,
            true,
        );
        let elapsed = start_time.elapsed();
        println!(
            "[full_extract][ZZMIDX12] Total execution time: {:.3}s ({} ms)",
            elapsed.as_secs_f64(),
            elapsed.as_millis()
        );
        return result;
    }

    let fa = FrameAnalysis::new(&frame_analysis_folder)
        .map_err(|e| format!("Failed to read FrameAnalysis data: {}", e))?;

    ExtractNewService::run_extract(
        &fa,
        &game_preset,
        &workspace_root_path,
        &lod_workspace_path,
        data_type_filter,
        true,
    )?;

    let elapsed = start_time.elapsed();
    println!(
        "[full_extract] Total execution time: {:.3}s ({} ms)",
        elapsed.as_secs_f64(),
        elapsed.as_millis()
    );

    Ok(())
}

#[tauri::command]
pub async fn analyze_draw_ib_submeshes(
    frame_analysis_folder: String,
    draw_ib: String,
) -> Result<Vec<DrawIBSubmeshRange>, String> {
    let draw_ib = draw_ib.trim();
    if draw_ib.is_empty() {
        return Ok(Vec::new());
    }

    let fa = FrameAnalysis::new(&frame_analysis_folder)
        .map_err(|e| format!("Failed to read FrameAnalysis data: {}", e))?;

    let ib_txt_files = fa.data.filter_filelist(&format!("-ib={}", draw_ib), ".txt");

    let mut submesh_set: BTreeSet<(u64, u64)> = BTreeSet::new();

    for ib_txt_file_name in ib_txt_files {
        let ib_txt_path = fa.data.dir.join(&ib_txt_file_name);
        let ib_txt_file = IndexBufferTxtFile::new(&ib_txt_path, false)?;

        if ib_txt_file.topology != ExtractTechnique::TRIANGLELIST {
            continue;
        }

        let first_index = ib_txt_file.first_index.trim().parse::<u64>().unwrap_or(0);
        let raw_index_count = ib_txt_file.index_count.trim().parse::<u64>().unwrap_or(0);
        let index_count = if raw_index_count == 0 {
            ib_txt_file.index_number_count
        } else {
            raw_index_count
        };

        if index_count == 0 {
            continue;
        }

        submesh_set.insert((first_index, index_count));
    }

    Ok(submesh_set
        .into_iter()
        .map(|(first_index, index_count)| DrawIBSubmeshRange {
            first_index: first_index.to_string(),
            index_count: index_count.to_string(),
        })
        .collect())
}
