use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::utils::ssmt_binary_utils::SSMTBinaryUtils;

#[derive(Debug, Clone)]
pub struct IndexBufferBufFile {
    pub index: String,
    pub match_first_index: String,
    pub read_draw_number: u32,

    pub min_number: u32,
    pub max_number: u32,
    pub number_count: u32,
    pub unique_vertex_count: u32,

    pub number_list: Vec<u32>,
}

impl Default for IndexBufferBufFile {
    fn default() -> Self {
        Self {
            index: String::new(),
            match_first_index: String::new(),
            read_draw_number: 0,
            min_number: 0,
            max_number: 0,
            number_count: 0,
            unique_vertex_count: 0,
            number_list: Vec::new(),
        }
    }
}

impl IndexBufferBufFile {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_file<P: AsRef<Path>>(
        index_buffer_file_path: P,
        format: &str,
    ) -> Result<Self, String> {
        let path = index_buffer_file_path.as_ref();
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();

        let index = file_name.get(0..6).unwrap_or_default().to_string();
        let format_lower = format.to_ascii_lowercase();

        let number_list = match format_lower.as_str() {
            "dxgi_format_r32_uint" => SSMTBinaryUtils::read_as_r32_uint(path)?,
            "dxgi_format_r16_uint" => SSMTBinaryUtils::read_as_r16_uint(path)?,
            other => {
                return Err(format!("Unsupported IB format: {}", other));
            }
        };

        let mut out = Self {
            index,
            number_list,
            ..Self::default()
        };
        out.recalculate_cached_stats();

        Ok(out)
    }

    pub fn from_file_byteoffset_read_length<P: AsRef<Path>>(
        index_buffer_file_path: P,
        format: &str,
        byte_offset: usize,
        read_length: usize,
    ) -> Result<Self, String> {
        let path = index_buffer_file_path.as_ref();
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();

        let index = file_name.get(0..6).unwrap_or_default().to_string();
        let format_lower = format.to_ascii_lowercase();

        let file_bytes = fs::read(path).map_err(|e| {
            format!(
                "Failed to read IndexBufferBufFile {}: {}",
                path.to_string_lossy(),
                e
            )
        })?;

        if byte_offset > file_bytes.len() {
            return Err(format!(
                "byte_offset {} is out of range for file size {}",
                byte_offset,
                file_bytes.len()
            ));
        }

        let end = byte_offset.checked_add(read_length).ok_or_else(|| {
            format!(
                "byte_offset {} + read_length {} overflows usize",
                byte_offset, read_length
            )
        })?;

        if end > file_bytes.len() {
            return Err(format!(
                "requested range [{}..{}) exceeds file size {}",
                byte_offset,
                end,
                file_bytes.len()
            ));
        }

        let selected = &file_bytes[byte_offset..end];

        let number_list: Vec<u32> = match format_lower.as_str() {
            "dxgi_format_r32_uint" => {
                if selected.len() % 4 != 0 {
                    return Err(format!(
                        "Selected R32_UINT range size {} is not a multiple of 4",
                        selected.len()
                    ));
                }

                selected
                    .chunks_exact(4)
                    .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect()
            }
            "dxgi_format_r16_uint" => {
                if selected.len() % 2 != 0 {
                    return Err(format!(
                        "Selected R16_UINT range size {} is not a multiple of 2",
                        selected.len()
                    ));
                }

                selected
                    .chunks_exact(2)
                    .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]) as u32)
                    .collect()
            }
            other => {
                return Err(format!("Unsupported IB format: {}", other));
            }
        };

        let mut out = Self {
            index,
            number_list,
            ..Self::default()
        };
        out.recalculate_cached_stats();

        Ok(out)
    }

    pub fn save_to_file_uint32<P: AsRef<Path>>(
        &self,
        output_ib_buf_file_path: P,
        offset: i32,
    ) -> Result<(), String> {
        let write_number_list = self.number_list_with_offset(offset)?;
        SSMTBinaryUtils::write_as_r32_uint(&write_number_list, output_ib_buf_file_path)
    }

    pub fn save_to_file_uint16<P: AsRef<Path>>(
        &self,
        output_ib_buf_file_path: P,
        offset: i32,
    ) -> Result<(), String> {
        let write_number_list = self.number_list_with_offset(offset)?;
        SSMTBinaryUtils::write_as_r16_uint(&write_number_list, output_ib_buf_file_path)
    }

    /// Replace invalid indices (>= mesh vertex count) with zero.
    ///
    /// Keep element count unchanged to avoid breaking drawindexed-style expanded IB data.
    pub fn self_clean_extra_vertex_index(&mut self, mesh_vertex_count: u32) {
        let mut new_number_list: Vec<u32> = Vec::with_capacity(self.number_list.len());

        for n in &self.number_list {
            if *n >= mesh_vertex_count {
                new_number_list.push(0);
            } else {
                new_number_list.push(*n);
            }
        }

        self.number_list = new_number_list;
        self.recalculate_cached_stats();
    }

    pub fn self_divide(&mut self, first_index: usize, index_count: usize) {
        if first_index >= self.number_list.len() || index_count == 0 {
            self.number_list.clear();
            self.recalculate_cached_stats();
            return;
        }

        let end = first_index
            .saturating_add(index_count)
            .min(self.number_list.len());
        self.number_list = self.number_list[first_index..end].to_vec();
        self.recalculate_cached_stats();
    }

    pub fn self_flip(&mut self) {
        if self.number_list.len() < 3 {
            return;
        }

        for i in (0..self.number_list.len().saturating_sub(2)).step_by(3) {
            self.number_list.swap(i, i + 2);
        }

        self.recalculate_cached_stats();
    }

    pub fn recalculate_cached_stats(&mut self) {
        if self.number_list.is_empty() {
            self.min_number = 0;
            self.max_number = 0;
            self.number_count = 0;
            self.unique_vertex_count = 0;
            return;
        }

        let mut tmp_max_number: u32 = 0;
        let mut tmp_min_number: u32 = u32::MAX;
        let mut unique_numbers: HashSet<u32> = HashSet::with_capacity(self.number_list.len());

        for n in &self.number_list {
            if *n > tmp_max_number {
                tmp_max_number = *n;
            }
            if *n < tmp_min_number {
                tmp_min_number = *n;
            }
            unique_numbers.insert(*n);
        }

        self.min_number = tmp_min_number;
        self.max_number = tmp_max_number;
        self.number_count = self.number_list.len() as u32;
        self.unique_vertex_count = unique_numbers.len() as u32;
    }

    fn number_list_with_offset(&self, offset: i32) -> Result<Vec<u32>, String> {
        let mut out: Vec<u32> = Vec::with_capacity(self.number_list.len());

        for n in &self.number_list {
            let shifted = (*n as i64) + (offset as i64);
            if shifted < 0 || shifted > u32::MAX as i64 {
                return Err(format!(
                    "Index value overflow after offset: original={}, offset={}",
                    n, offset
                ));
            }
            out.push(shifted as u32);
        }

        Ok(out)
    }
}
