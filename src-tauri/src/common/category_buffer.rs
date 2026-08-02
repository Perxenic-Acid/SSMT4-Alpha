use std::collections::HashMap;

use crate::common::d3d11_element::D3D11Element;
use crate::common::d3d11_gametype::D3D11GameType;
use crate::common::element_buffer::ElementBuffer;
use crate::utils::ssmt_binary_utils::SSMTBinaryUtils;

/// Parse one category `.buf` into per-element buffers.
#[derive(Debug, Clone, Default)]
pub struct CategoryBuffer {
    pub buf_file_path: String,
    pub category_name: String,
    pub d3d11_game_type: D3D11GameType,
    pub element_buffer_list: Vec<ElementBuffer>,
}

impl CategoryBuffer {
    pub fn new(
        in_buf_file_path: String,
        in_category_name: String,
        in_d3d11_game_type: D3D11GameType,
    ) -> Result<Self, String> {
        let category_stride = in_d3d11_game_type
            .category_stride_dict
            .get(&in_category_name)
            .ok_or_else(|| format!("Category stride not found for '{}'", in_category_name))?;

        let category_stride = usize::try_from(*category_stride)
            .map_err(|_| format!("Category stride too large for usize: {}", category_stride))?;

        if category_stride == 0 {
            return Err(format!(
                "Category '{}' stride must be greater than 0",
                in_category_name
            ));
        }

        // Split file bytes by category stride. Last chunk can be shorter.
        let category_index_bytes =
            SSMTBinaryUtils::read_binary_file_by_stride(&in_buf_file_path, category_stride, false)?;

        let mut category_d3d11_element_list: Vec<D3D11Element> = Vec::new();
        for element_name in &in_d3d11_game_type.ordered_full_element_list {
            let d3d11_element = in_d3d11_game_type
                .element_name_d3d11_element_dict
                .get(element_name)
                .ok_or_else(|| format!("Element '{}' not found in game type dict", element_name))?;

            if d3d11_element.category != in_category_name {
                continue;
            }

            category_d3d11_element_list.push(d3d11_element.clone());
        }

        let mut element_buffer_list: Vec<ElementBuffer> = Vec::new();
        let mut start_index = 0usize;

        for category_d3d11_element in category_d3d11_element_list {
            let byte_width =
                usize::try_from(category_d3d11_element.byte_width_int()).map_err(|_| {
                    format!(
                        "ByteWidth too large for usize: {}",
                        category_d3d11_element.byte_width
                    )
                })?;
            let end_index = start_index
                .checked_add(byte_width)
                .ok_or_else(|| "Element byte range overflow".to_string())?;

            let mut element_byte_dict: HashMap<usize, Vec<u8>> = HashMap::new();
            for (index, value) in &category_index_bytes {
                let length = end_index.saturating_sub(start_index);
                if length > 0 && start_index + length <= value.len() {
                    element_byte_dict.insert(*index, value[start_index..end_index].to_vec());
                }
            }

            element_buffer_list.push(ElementBuffer::new(
                category_d3d11_element,
                element_byte_dict,
            ));
            start_index = end_index;
        }

        Ok(Self {
            buf_file_path: in_buf_file_path,
            category_name: in_category_name,
            d3d11_game_type: in_d3d11_game_type,
            element_buffer_list,
        })
    }
}
