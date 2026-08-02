use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::common::d3d11_element::D3D11Element;
use crate::utils::ssmt_binary_utils::SSMTBinaryUtils;

#[derive(Debug, Clone, Default)]
pub struct VertexBufferBufFile {
    pub final_vb0_bytes: Vec<u8>,
}

impl VertexBufferBufFile {
    pub fn new_from_bytes(vb0_bytes: Vec<u8>) -> Self {
        Self {
            final_vb0_bytes: vb0_bytes,
        }
    }

    pub fn new_from_buf_dict_list(
        buf_dict_list: Vec<HashMap<usize, Vec<u8>>>,
    ) -> Result<Self, String> {
        let merged_vb0_dict = SSMTBinaryUtils::merge_byte_dicts(buf_dict_list)?;
        let final_vb0 = SSMTBinaryUtils::merge_dictionary_values(merged_vb0_dict);

        Ok(Self {
            final_vb0_bytes: final_vb0,
        })
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, output_vb_buf_file_path: P) -> Result<(), String> {
        fs::write(output_vb_buf_file_path.as_ref(), &self.final_vb0_bytes)
            .map_err(|e| e.to_string())
    }

    pub fn self_divide(
        &mut self,
        min_number: usize,
        max_number: usize,
        stride: usize,
    ) -> Result<(), String> {
        if stride == 0 {
            return Err("Stride must be greater than 0".to_string());
        }
        if self.final_vb0_bytes.is_empty() {
            return Ok(());
        }
        if max_number < min_number {
            return Err(format!(
                "Invalid range: max_number ({}) < min_number ({})",
                max_number, min_number
            ));
        }

        let start_index = min_number
            .checked_mul(stride)
            .ok_or_else(|| "start_index overflow".to_string())?;
        let end_index_exclusive = max_number
            .checked_add(1)
            .and_then(|v| v.checked_mul(stride))
            .ok_or_else(|| "end_index overflow".to_string())?;

        if start_index >= self.final_vb0_bytes.len() {
            return Err(format!(
                "start_index {} out of range (len={})",
                start_index,
                self.final_vb0_bytes.len()
            ));
        }
        if end_index_exclusive > self.final_vb0_bytes.len() {
            return Err(format!(
                "end_index {} out of range (len={})",
                end_index_exclusive,
                self.final_vb0_bytes.len()
            ));
        }

        self.final_vb0_bytes = self.final_vb0_bytes[start_index..end_index_exclusive].to_vec();
        Ok(())
    }

    pub fn get_element_name_vb_data_map(
        &self,
        current_d3d11_element_list: &[D3D11Element],
        real_stride: usize,
    ) -> Result<HashMap<String, Vec<u8>>, String> {
        if real_stride == 0 {
            return Err("RealStride must be greater than 0".to_string());
        }
        if self.final_vb0_bytes.len() % real_stride != 0 {
            return Err(format!(
                "FinalVB0Bytes length {} is not a multiple of RealStride {}",
                self.final_vb0_bytes.len(),
                real_stride
            ));
        }

        let mut element_name_vb_data_map: HashMap<String, Vec<u8>> = HashMap::new();
        for element in current_d3d11_element_list {
            element_name_vb_data_map.insert(element.element_name.clone(), Vec::new());
        }

        for i in (0..self.final_vb0_bytes.len()).step_by(real_stride) {
            let mut offset = 0usize;

            for element in current_d3d11_element_list {
                let element_name = element.element_name.clone();
                let byte_width = usize::try_from(element.byte_width_int()).map_err(|_| {
                    format!("ByteWidth too large for usize: {}", element.byte_width)
                })?;

                let start = i
                    .checked_add(offset)
                    .ok_or_else(|| "element start overflow".to_string())?;
                let end = start
                    .checked_add(byte_width)
                    .ok_or_else(|| "element end overflow".to_string())?;

                if end > self.final_vb0_bytes.len() || end > i + real_stride {
                    return Err(format!(
						"Element range out of bounds: element={}, start={}, end={}, vertex_start={}, stride={}",
						element_name, start, end, i, real_stride
					));
                }

                if let Some(target) = element_name_vb_data_map.get_mut(&element_name) {
                    target.extend_from_slice(&self.final_vb0_bytes[start..end]);
                }

                offset = offset
                    .checked_add(byte_width)
                    .ok_or_else(|| "offset overflow".to_string())?;
            }
        }

        Ok(element_name_vb_data_map)
    }
}
