use crate::utils::ssmt_file_utils::SSMTFileUtils;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct SSMTBinaryUtils;

pub struct MigotoBufferMetadata {
    pub byte_offset: usize,
    pub vertex_count: usize,
    pub first_vertex: usize,
    pub stride: usize,
}

impl SSMTBinaryUtils {
    pub fn read_migoto_buffer_metadata(
        txt_file_path: impl AsRef<Path>,
    ) -> Result<MigotoBufferMetadata, String> {
        let stride =
            SSMTFileUtils::find_migoto_ini_attribute_in_file(txt_file_path.as_ref(), "stride")?;
        let vertex_count = SSMTFileUtils::find_migoto_ini_attribute_in_file(
            txt_file_path.as_ref(),
            "vertex count",
        )?;
        let byte_offset = SSMTFileUtils::find_migoto_ini_attribute_in_file(
            txt_file_path.as_ref(),
            "byte offset",
        )?;
        let first_vertex = SSMTFileUtils::find_migoto_ini_attribute_in_file(
            txt_file_path.as_ref(),
            "first vertex",
        )?;

        Ok(MigotoBufferMetadata {
            byte_offset: byte_offset.trim().parse::<usize>().unwrap_or(0),
            vertex_count: vertex_count.trim().parse::<usize>().unwrap_or(0),
            first_vertex: first_vertex.trim().parse::<usize>().unwrap_or(0),
            stride: stride.trim().parse::<usize>().unwrap_or(0),
        })
    }

    pub fn get_file_size_from_migoto_txt(txt_file_path: impl AsRef<Path>) -> Result<u64, String> {
        let metadata = Self::read_migoto_buffer_metadata(txt_file_path)?;
        (metadata.stride as u64)
            .checked_mul(metadata.vertex_count as u64)
            .ok_or_else(|| {
                format!(
                    "stride {} * vertex_count {} overflows u64",
                    metadata.stride, metadata.vertex_count
                )
            })
    }

    pub fn read_as_r32_uint(file_path: impl AsRef<Path>) -> Result<Vec<u32>, String> {
        let file_bytes = fs::read(file_path.as_ref()).map_err(|e| e.to_string())?;
        if file_bytes.len() % 4 != 0 {
            return Err(format!(
                "R32_UINT file size {} is not a multiple of 4",
                file_bytes.len()
            ));
        }

        let mut out: Vec<u32> = Vec::with_capacity(file_bytes.len() / 4);
        for chunk in file_bytes.chunks_exact(4) {
            out.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }

        Ok(out)
    }

    pub fn read_as_r16_uint(file_path: impl AsRef<Path>) -> Result<Vec<u32>, String> {
        let file_bytes = fs::read(file_path.as_ref()).map_err(|e| e.to_string())?;
        if file_bytes.len() % 2 != 0 {
            return Err(format!(
                "R16_UINT file size {} is not a multiple of 2",
                file_bytes.len()
            ));
        }

        let mut out: Vec<u32> = Vec::with_capacity(file_bytes.len() / 2);
        for chunk in file_bytes.chunks_exact(2) {
            out.push(u16::from_le_bytes([chunk[0], chunk[1]]) as u32);
        }

        Ok(out)
    }

    pub fn write_as_r32_uint(numbers: &[u32], output_path: impl AsRef<Path>) -> Result<(), String> {
        let mut bytes: Vec<u8> = Vec::with_capacity(numbers.len() * 4);
        for n in numbers {
            bytes.extend_from_slice(&n.to_le_bytes());
        }

        fs::write(output_path.as_ref(), bytes).map_err(|e| e.to_string())
    }

    pub fn write_as_r16_uint(numbers: &[u32], output_path: impl AsRef<Path>) -> Result<(), String> {
        let mut bytes: Vec<u8> = Vec::with_capacity(numbers.len() * 2);
        for n in numbers {
            let value = u16::try_from(*n)
                .map_err(|_| format!("Value {} exceeds u16 range for R16_UINT", n))?;
            bytes.extend_from_slice(&value.to_le_bytes());
        }

        fs::write(output_path.as_ref(), bytes).map_err(|e| e.to_string())
    }

    pub fn merge_dictionary_values(merged_vb0_dict: HashMap<usize, Vec<u8>>) -> Vec<u8> {
        let mut merged_bytes: Vec<u8> = Vec::new();

        // HashMap iteration order is non-deterministic, so we sort keys first.
        let mut keys: Vec<usize> = merged_vb0_dict.keys().copied().collect();
        keys.sort_unstable();

        for key in keys {
            if let Some(bytes) = merged_vb0_dict.get(&key) {
                merged_bytes.extend_from_slice(bytes);
            }
        }

        merged_bytes
    }

    pub fn merge_byte_dicts(
        byte_dicts: Vec<HashMap<usize, Vec<u8>>>,
    ) -> Result<HashMap<usize, Vec<u8>>, String> {
        let mut merged_dict: HashMap<usize, Vec<u8>> = HashMap::new();

        if byte_dicts.is_empty() {
            return Ok(merged_dict);
        }

        for key in byte_dicts[0].keys() {
            let mut total_length = 0usize;

            for dict in &byte_dicts {
                let Some(bytes) = dict.get(key) else {
                    return Err(format!("Key {} not found in all dictionaries.", key));
                };
                total_length += bytes.len();
            }

            let mut combined = Vec::with_capacity(total_length);
            for dict in &byte_dicts {
                if let Some(bytes) = dict.get(key) {
                    combined.extend_from_slice(bytes);
                }
            }

            merged_dict.insert(*key, combined);
        }

        Ok(merged_dict)
    }

    /// Get valid bytes by trimming trailing all-zero chunks scanned from end by `stride`.
    pub fn get_valid_file_data_null_terminated_by_stride(
        file_path: impl AsRef<Path>,
        stride: usize,
    ) -> Result<Vec<u8>, String> {
        if stride == 0 {
            return Err("stride must be greater than 0".to_string());
        }

        let file_bytes = fs::read(file_path.as_ref()).map_err(|e| e.to_string())?;
        let mut end = file_bytes.len();

        while end > 0 {
            let read_size = stride.min(end);
            let start = end - read_size;
            let is_all_zero = file_bytes[start..end].iter().all(|b| *b == 0);

            if is_all_zero {
                end = start;
            } else {
                break;
            }
        }

        Ok(file_bytes[..end].to_vec())
    }

    /// Read a binary file and split it into chunks by `stride`.
    ///
    /// The map key is the chunk index (0-based), and the value is the chunk bytes.
    /// When `valid_check_read` is true, trailing all-zero chunks are trimmed first.
    pub fn read_binary_file_by_stride(
        file_path: impl AsRef<Path>,
        stride: usize,
        valid_check_read: bool,
    ) -> Result<HashMap<usize, Vec<u8>>, String> {
        if stride == 0 {
            return Err("stride must be greater than 0".to_string());
        }

        let file_bytes = if valid_check_read {
            Self::get_valid_file_data_null_terminated_by_stride(file_path.as_ref(), stride)?
        } else {
            fs::read(file_path.as_ref()).map_err(|e| e.to_string())?
        };
        let mut result: HashMap<usize, Vec<u8>> = HashMap::new();

        for (i, chunk) in file_bytes.chunks(stride).enumerate() {
            result.insert(i, chunk.to_vec());
        }

        Ok(result)
    }

    /// Read `vertex_count` chunks (each chunk is `stride` bytes) starting from `byte_offset`.
    ///
    /// The returned map key is chunk index (0-based within the read window),
    /// and the value is the chunk bytes. Returned chunk count equals `vertex_count`.
    pub fn read_binary_file_by_stride_with_offset_and_vertex_count(
        file_path: impl AsRef<Path>,
        stride: usize,
        valid_check_read: bool,
        byte_offset: usize,
        vertex_count: usize,
    ) -> Result<HashMap<usize, Vec<u8>>, String> {
        if stride == 0 {
            return Err("stride must be greater than 0".to_string());
        }

        let file_bytes = if valid_check_read {
            Self::get_valid_file_data_null_terminated_by_stride(file_path.as_ref(), stride)?
        } else {
            fs::read(file_path.as_ref()).map_err(|e| e.to_string())?
        };

        if byte_offset > file_bytes.len() {
            return Err(format!(
                "byte_offset {} is out of range for file size {}",
                byte_offset,
                file_bytes.len()
            ));
        }

        let read_len = vertex_count.checked_mul(stride).ok_or_else(|| {
            format!(
                "vertex_count {} * stride {} overflows usize",
                vertex_count, stride
            )
        })?;

        let end = byte_offset.checked_add(read_len).ok_or_else(|| {
            format!(
                "byte_offset {} + read_len {} overflows usize",
                byte_offset, read_len
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

        let mut result: HashMap<usize, Vec<u8>> = HashMap::new();
        for (i, chunk) in file_bytes[byte_offset..end].chunks(stride).enumerate() {
            result.insert(i, chunk.to_vec());
        }

        Ok(result)
    }

    pub fn get_range_bytes(buf: &[u8], start: usize, end: usize) -> Result<Vec<u8>, String> {
        if start > end || end > buf.len() {
            return Err(format!(
                "Invalid byte range [{}..{}) for buffer len {}",
                start,
                end,
                buf.len()
            ));
        }
        Ok(buf[start..end].to_vec())
    }
}
