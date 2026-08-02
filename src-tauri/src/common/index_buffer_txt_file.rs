use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct IndexBufferTxtFile {
    pub file_name: String,
    pub fa_filename: String,
    pub index: String,
    pub hash: String,

    //注意，first_index和index_count不一定在txt中存在
    //此时first_index默认为0
    //此时index_count不可使用，而是使用index_number_count
    pub first_index: String,
    pub index_count: String,
    pub topology: String,
    pub format: String,
    pub byte_offset: String,

    pub max_number: u64,
    pub min_number: u64,
    pub unique_vertex_count: u64,
    pub index_number_count: u64,

    pub number_list: Vec<u64>,
}

impl Default for IndexBufferTxtFile {
    fn default() -> Self {
        Self {
            file_name: String::new(),
            fa_filename: String::new(),
            index: String::new(),
            hash: String::new(),
            first_index: "0".to_string(),

            //如果没有的话默认为0，这里为0的话生成Mod时就不生成match_index_count了
            index_count: "0".to_string(),
            topology: String::new(),
            format: String::new(),
            byte_offset: "0".to_string(),
            max_number: 0,
            min_number: 999_999_999,
            unique_vertex_count: 0,
            index_number_count: 0,
            number_list: Vec::new(),
        }
    }
}

impl IndexBufferTxtFile {
    pub fn new<P: AsRef<Path>>(
        ib_txt_file_path: P,
        read_txt_number_data: bool,
    ) -> Result<Self, String> {
        let path = ib_txt_file_path.as_ref();
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();

        let mut out = Self {
            file_name: file_name.clone(),
            index: file_name.get(0..6).unwrap_or_default().to_string(),
            hash: file_name.get(10..18).unwrap_or_default().to_string(),
            ..Default::default()
        };

        let raw = std::fs::read(path).map_err(|e| {
            format!(
                "Failed to read IndexBufferTxtFile {}: {}",
                path.to_string_lossy(),
                e
            )
        })?;
        let content = String::from_utf8_lossy(&raw).into_owned();
        let file_line_array: Vec<&str> = content.lines().collect();

        let topology_str = "topology:";
        let first_index_str = "first index:";
        let index_count_str = "index count:";
        let format_str = "format:";
        let byte_offset_str = "byte offset:";

        let mut count: u64 = 0;
        let mut unique_vertex_number_set: HashSet<u64> = HashSet::new();

        for line in file_line_array {
            if line.starts_with(topology_str) {
                out.topology = line[topology_str.len()..].trim().to_string();
            } else if line.starts_with(first_index_str) {
                out.first_index = line[first_index_str.len()..].trim().to_string();
            } else if line.starts_with(index_count_str) {
                out.index_count = line[index_count_str.len()..].trim().to_string();
            } else if line.starts_with(format_str) {
                out.format = line[format_str.len()..].trim().to_string();
            } else if line.starts_with(byte_offset_str) {
                out.byte_offset = line[byte_offset_str.len()..].trim().to_string();
            } else {
                if !read_txt_number_data && count > 8 {
                    break;
                }

                let trim_line = line.trim();
                if trim_line.is_empty() {
                    count += 1;
                    continue;
                }

                let line_splits: Vec<&str> = trim_line.split_whitespace().collect();
                if line_splits.len() == 3 {
                    for line_split in line_splits {
                        let number = line_split.parse::<u64>().map_err(|e| e.to_string())?;
                        unique_vertex_number_set.insert(number);
                        out.number_list.push(number);

                        if number > out.max_number {
                            out.max_number = number;
                        } else if number < out.min_number {
                            out.min_number = number;
                        }
                    }
                }
            }

            count += 1;
        }

        out.index_number_count = out.number_list.len() as u64;
        out.unique_vertex_count = unique_vertex_number_set.len() as u64;

        Ok(out)
    }
}
