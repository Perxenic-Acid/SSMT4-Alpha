use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use crate::common::index_buffer_txt_file::IndexBufferTxtFile;
use crate::constants::gametype::ExtractTechnique;
use crate::utils::ssmt_string_utils::SSMTStringUtils;

#[derive(Debug, Clone, Default)]
pub struct FrameAnalysisData {
    pub dir: PathBuf,
    pub files: Vec<String>,
}

impl FrameAnalysisData {
    fn normalize_dump_filename(file_name: &str) -> String {
        file_name
            .strip_suffix(".lnk")
            .unwrap_or(file_name)
            .to_string()
    }

    /// Create a new `FrameAnalysisData` and read files (non-recursive) from `dir`.
    pub fn new(frame_analysis_folder: &str) -> Result<Self, String> {
        let dir = PathBuf::from(frame_analysis_folder);
        let files = Self::read_files_non_recursive(&dir)?;
        Ok(Self { dir, files })
    }

    fn read_files_non_recursive(dir: &Path) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        if !dir.exists() {
            return Ok(out);
        }
        for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                    out.push(Self::normalize_dump_filename(file_name));
                }
            }
        }
        out.sort();
        out.dedup();
        Ok(out)
    }

    pub fn get_trianglelist_index_list(&self, draw_ib: &str) -> Vec<String> {
        let mut trianglelist_index_set: BTreeSet<String> = BTreeSet::new();
        let search_str = format!("-ib={}", draw_ib);

        for ib_txt_file_name in self.filter_filelist(&search_str, ".txt") {
            let index = ib_txt_file_name
                .get(0..6)
                .unwrap_or_default()
                .trim()
                .to_string();
            if !index.is_empty() {
                trianglelist_index_set.insert(index);
            }
        }

        if trianglelist_index_set.is_empty() {
            for ib_file_path in self.filter_filelist(&search_str, ".buf") {
                let ib_filename = Path::new(&ib_file_path)
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
                    .to_string();
                let index = SSMTStringUtils::substring(&ib_filename, 0, 6).unwrap_or_default();
                if !index.is_empty() {
                    trianglelist_index_set.insert(index);
                }
            }
        }

        trianglelist_index_set.into_iter().collect()
    }

    pub fn get_all_drawib_list(&self) -> Vec<String> {
        let mut draw_ib_set: BTreeSet<String> = BTreeSet::new();

        for ib_txt_file_name in self.filter_filelist("-ib=", ".txt") {
            let draw_ib = ib_txt_file_name.get(10..18).unwrap_or_default().to_string();
            if draw_ib.len() == 8 {
                draw_ib_set.insert(draw_ib);
            }
        }

        draw_ib_set.into_iter().collect()
    }

    pub fn filter_filelist(&self, content: &str, suffix: &str) -> Vec<String> {
        self.files
            .iter()
            .filter(|f| f.contains(content) && f.ends_with(suffix))
            .cloned()
            .collect()
    }

    pub fn filter_filelist_by_content_and_suffix(
        &self,
        content: &str,
        suffix: &str,
    ) -> Vec<String> {
        self.filter_filelist(content, suffix)
    }

    pub fn filter_first_file(&self, content: &str, suffix: &str) -> Result<String, String> {
        Ok(self
            .files
            .iter()
            .find(|f| f.contains(content) && f.ends_with(suffix))
            .cloned()
            .unwrap_or_default())
    }

    pub fn filter_first_file_by_content_and_suffix(
        &self,
        content: &str,
        suffix: &str,
    ) -> Result<String, String> {
        self.filter_first_file(content, suffix)
    }

    /// Filter texture filenames by `content`, keep `.dds`/`.jpg`, and sort by `ps-t` slot number.
    pub fn filter_texture_filename_list(&self, content: &str) -> Vec<String> {
        let mut filtered: Vec<(i32, String)> = self
            .files
            .iter()
            .filter(|file_name| file_name.contains(content))
            .filter(|file_name| file_name.ends_with(".dds") || file_name.ends_with(".jpg"))
            .map(|file_name| {
                let slot_number =
                    SSMTStringUtils::get_pixel_slot_number_from_texture_file_name(file_name)
                        .unwrap_or(-1);
                (slot_number, file_name.clone())
            })
            .collect();

        filtered.sort_by_key(|(slot_number, _)| *slot_number);
        filtered
            .into_iter()
            .map(|(_, file_name)| file_name)
            .collect()
    }

    /// Read `Component N -> MatchFirstIndex` mapping from IB txt files for the given DrawIB.
    ///
    /// Equivalent to C# `Read_ComponentName_MatchFirstIndex_Dict`:
    /// - find `*-ib={draw_ib}*.txt`
    /// - keep only `topology == trianglelist`
    /// - deduplicate and sort `first index`
    /// - name as `Component 1..n`
    pub fn read_component_name_match_first_index_dict(
        &self,
        draw_ib: &str,
    ) -> Result<HashMap<String, u64>, String> {
        let mut component_map: HashMap<String, u64> = HashMap::new();

        let ib_txt_file_name_list = self.filter_filelist(&format!("-ib={}", draw_ib), ".txt");

        if ib_txt_file_name_list.is_empty() {
            return Ok(component_map);
        }

        let mut match_first_index_list: Vec<u64> = Vec::new();

        for ib_txt_file_name in ib_txt_file_name_list {
            let ib_txt_path = self.dir.join(&ib_txt_file_name);
            let ib_txt_file = IndexBufferTxtFile::new(&ib_txt_path, false)?;

            if ib_txt_file.topology != ExtractTechnique::TRIANGLELIST {
                continue;
            }

            let match_first_index = ib_txt_file.first_index.parse::<u64>().map_err(|e| {
                format!(
                    "Failed to parse first index '{}' from {}: {}",
                    ib_txt_file.first_index,
                    ib_txt_path.to_string_lossy(),
                    e
                )
            })?;

            if !match_first_index_list.contains(&match_first_index) {
                match_first_index_list.push(match_first_index);
            }
        }

        match_first_index_list.sort_unstable();

        for (idx, match_first_index) in match_first_index_list.into_iter().enumerate() {
            component_map.insert(format!("Component {}", idx + 1), match_first_index);
        }

        Ok(component_map)
    }

    /// Read draw call index list for a specified DrawIB + ComponentName.
    ///
    /// Equivalent to C# `Read_DrawCallIndexList`:
    /// - resolve `ComponentName -> MatchFirstIndex`
    /// - scan `*-ib={draw_ib}*.txt`
    /// - keep `trianglelist`
    /// - return indexes whose `first index` equals resolved match value
    pub fn read_draw_call_index_list(
        &self,
        draw_ib: &str,
        component_name: &str,
    ) -> Result<Vec<String>, String> {
        let component_map = self.read_component_name_match_first_index_dict(draw_ib)?;

        let match_first_index = component_map
            .get(component_name)
            .ok_or_else(|| format!("ComponentName not found: {}", component_name))?;

        let ib_txt_file_name_list = self.filter_filelist(&format!("-ib={}", draw_ib), ".txt");

        let mut draw_call_index_list: Vec<String> = Vec::new();

        for ib_txt_file_name in ib_txt_file_name_list {
            let ib_txt_path = self.dir.join(&ib_txt_file_name);
            let ib_txt_file = IndexBufferTxtFile::new(&ib_txt_path, false)?;

            if ib_txt_file.topology != ExtractTechnique::TRIANGLELIST {
                continue;
            }

            let file_match_first_index = ib_txt_file.first_index.parse::<u64>().map_err(|e| {
                format!(
                    "Failed to parse first index '{}' from {}: {}",
                    ib_txt_file.first_index,
                    ib_txt_path.to_string_lossy(),
                    e
                )
            })?;

            if file_match_first_index == *match_first_index {
                draw_call_index_list.push(ib_txt_file.index.clone());
            }
        }

        Ok(draw_call_index_list)
    }
}

#[cfg(test)]
mod tests {
    use super::FrameAnalysisData;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ssmt4-{name}-{unique}"));
        fs::create_dir_all(&dir).expect("failed to create temp dir");
        dir
    }

    #[test]
    fn get_trianglelist_index_list_supports_txt_shortcuts() {
        let dir = create_temp_dir("frameanalysis-txt-shortcuts");
        let shortcut_name = "000081-ib=6c751ecd-vs=bcab0367ebd6d16c-ps=0bad79cb2b324614.txt.lnk";
        fs::write(dir.join(shortcut_name), b"").expect("failed to create shortcut placeholder");

        let data = FrameAnalysisData::new(dir.to_string_lossy().as_ref())
            .expect("failed to build frame analysis data");
        assert_eq!(
            data.get_trianglelist_index_list("6c751ecd"),
            vec!["000081".to_string()]
        );

        fs::remove_dir_all(&dir).expect("failed to remove temp dir");
    }

    #[test]
    fn get_trianglelist_index_list_supports_buf_shortcuts_fallback() {
        let dir = create_temp_dir("frameanalysis-buf-shortcuts");
        let shortcut_name = "000095-ib=6c751ecd-vs=fc3bda962441764a-ps=d87752191b506b3d.buf.lnk";
        fs::write(dir.join(shortcut_name), b"").expect("failed to create shortcut placeholder");

        let data = FrameAnalysisData::new(dir.to_string_lossy().as_ref())
            .expect("failed to build frame analysis data");
        assert_eq!(
            data.get_trianglelist_index_list("6c751ecd"),
            vec!["000095".to_string()]
        );

        fs::remove_dir_all(&dir).expect("failed to remove temp dir");
    }
}
