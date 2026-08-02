use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct DrawIBEntry {
    #[serde(rename = "DrawIB")]
    pub draw_ib: String,

    #[serde(rename = "Alias")]
    pub alias: String,
}

/// `DrawIBConfig` holds the list of `DrawIBEntry` loaded from a JSON file.
pub struct DrawIBConfig {
    pub path: String,
    pub entries: Vec<DrawIBEntry>,
}

impl DrawIBConfig {
    fn read_drawib_list(path: &str) -> Result<Vec<DrawIBEntry>, Box<dyn Error>> {
        let s = fs::read_to_string(path)?;
        let list: Vec<DrawIBEntry> = serde_json::from_str(&s)?;
        Ok(list)
    }

    /// Load from `path` and return a `DrawIBConfig` with entries populated.
    pub fn new<P: Into<String>>(path: P) -> Result<Self, Box<dyn Error>> {
        let path_s = path.into();
        let entries = Self::read_drawib_list(&path_s)?;
        Ok(Self {
            path: path_s,
            entries,
        })
    }

    pub fn new_from_workspace(workspace_path: &str) -> Result<Self, Box<dyn Error>> {
        let config_file_path = PathBuf::from(&workspace_path).join("Config.json");
        Self::new(config_file_path.to_string_lossy().to_string())
    }

    pub fn read_drawib_list_from_workspace(
        workspace_path: &str,
    ) -> Result<Vec<DrawIBEntry>, Box<dyn Error>> {
        let config_file_path = PathBuf::from(&workspace_path).join("Config.json");
        Self::read_drawib_list(&config_file_path.to_string_lossy().to_string())
    }
}
