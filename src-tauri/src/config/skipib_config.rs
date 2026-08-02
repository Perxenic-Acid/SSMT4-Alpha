use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct SkipIBEntry {
    #[serde(rename = "SkipIB")]
    pub skip_ib: String,

    #[serde(rename = "Alias")]
    pub alias: String,

    #[serde(rename = "IndexCount", default)]
    pub index_count: String,

    #[serde(rename = "FirstIndex", default)]
    pub first_index: String,
}

/// `SkipIBConfig` holds the list of `SkipIBEntry` loaded from a JSON file.
pub struct SkipIBConfig {
    pub path: String,
    pub entries: Vec<SkipIBEntry>,
}

impl SkipIBConfig {
    fn read_skipib_list(path: &str) -> Result<Vec<SkipIBEntry>, Box<dyn Error>> {
        let s = fs::read_to_string(path)?;
        let list: Vec<SkipIBEntry> = serde_json::from_str(&s)?;
        Ok(list)
    }

    /// Load from `path` and return a `SkipIBConfig` with entries populated.
    pub fn new<P: Into<String>>(path: P) -> Result<Self, Box<dyn Error>> {
        let path_s = path.into();
        let entries = Self::read_skipib_list(&path_s)?;
        Ok(Self {
            path: path_s,
            entries,
        })
    }

    pub fn new_from_workspace(workspace_path: &str) -> Result<Self, Box<dyn Error>> {
        let config_file_path = PathBuf::from(&workspace_path).join("SkipIBConfig.json");
        Self::new(config_file_path.to_string_lossy().to_string())
    }

    pub fn read_skipib_list_from_workspace(
        workspace_path: &str,
    ) -> Result<Vec<SkipIBEntry>, Box<dyn Error>> {
        let config_file_path = PathBuf::from(&workspace_path).join("SkipIBConfig.json");
        Self::read_skipib_list(&config_file_path.to_string_lossy().to_string())
    }
}
