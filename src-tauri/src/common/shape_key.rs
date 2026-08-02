use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::common::d3d11_element::D3D11Element;
use crate::common::d3d11_gametype::D3D11GameType;
use crate::common::element_buffer::ElementBuffer;
use crate::common::index_buffer_buf_file::IndexBufferBufFile;
use crate::utils::ssmt_binary_utils::SSMTBinaryUtils;

#[derive(Debug, Clone, Default)]
pub struct ShapeKey {
    pub check_sum: i32,
    pub shape_key_offset_number_last: i32,
    pub shape_key_vertex_offset_count: i32,
    pub shape_key_vertex_id_list: Vec<i32>,

    pub shape_key_id_vertex_id_list_map: HashMap<i32, Vec<i32>>,
    pub shape_key_id_vertex_id_vertex_offset_map: HashMap<i32, HashMap<i32, Vec<u8>>>,
}

impl ShapeKey {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_files(
        in_shape_key_offset_path: impl AsRef<Path>,
        in_shape_key_vertex_id_path: impl AsRef<Path>,
        in_shape_key_vertex_offset_path: impl AsRef<Path>,
    ) -> Result<Self, String> {
        let offset_path = in_shape_key_offset_path.as_ref();
        let vertex_id_path = in_shape_key_vertex_id_path.as_ref();
        let vertex_offset_path = in_shape_key_vertex_offset_path.as_ref();

        if offset_path.as_os_str().is_empty() {
            return Err("InShapeKeyOffsetPath is empty".to_string());
        }
        if vertex_id_path.as_os_str().is_empty() {
            return Err("InShapeKeyVertexIdPath is empty".to_string());
        }
        if vertex_offset_path.as_os_str().is_empty() {
            return Err("InShapeKeyVertexOffsetPath is empty".to_string());
        }

        let cscb0_buf_data = fs::read(offset_path).map_err(|e| {
            format!(
                "Failed to read shapekey offset file {}: {}",
                offset_path.to_string_lossy(),
                e
            )
        })?;

        let shape_key_offset_buf = cscb0_buf_data[..cscb0_buf_data.len().min(512)].to_vec();
        if shape_key_offset_buf.len() < 4 {
            return Err("ShapeKeyOffset buffer is too short".to_string());
        }

        let shape_key_offsets = SSMTBinaryUtils::read_as_r32_uint(offset_path)?;
        let check_sum = shape_key_offsets
            .iter()
            .take(4)
            .map(|v| *v as i32)
            .sum::<i32>();

        let shape_key_offset_number_data =
            &shape_key_offset_buf[shape_key_offset_buf.len() - 4..shape_key_offset_buf.len()];
        let shape_key_offset_number_last = u32::from_le_bytes(
            shape_key_offset_number_data
                .try_into()
                .unwrap_or([0, 0, 0, 0]),
        ) as i32;

        let cst0_buf_data = fs::read(vertex_id_path).map_err(|e| {
            format!(
                "Failed to read shapekey vertex id file {}: {}",
                vertex_id_path.to_string_lossy(),
                e
            )
        })?;
        let shape_key_vertex_id_buf_len =
            (shape_key_offset_number_last.max(0) as usize).saturating_mul(4);
        let shape_key_vertex_id_buf =
            cst0_buf_data[..cst0_buf_data.len().min(shape_key_vertex_id_buf_len)].to_vec();

        let cst1_buf_data = fs::read(vertex_offset_path).map_err(|e| {
            format!(
                "Failed to read shapekey vertex offset file {}: {}",
                vertex_offset_path.to_string_lossy(),
                e
            )
        })?;
        let shape_key_vertex_offset_buf_len =
            (shape_key_offset_number_last.max(0) as usize).saturating_mul(12);
        let shape_key_vertex_offset_buf =
            cst1_buf_data[..cst1_buf_data.len().min(shape_key_vertex_offset_buf_len)].to_vec();

        let shape_key_vertex_offset_count = (cst1_buf_data.len() / 12) as i32;

        let mut shape_key_vertex_offset_list: Vec<Vec<u8>> = Vec::new();
        for chunk in shape_key_vertex_offset_buf.chunks_exact(12) {
            shape_key_vertex_offset_list.push(chunk[..6].to_vec());
        }

        let mut shape_key_vertex_id_list: Vec<i32> = Vec::new();
        let mut tmp_vertex_id_set: HashSet<i32> = HashSet::new();
        for chunk in shape_key_vertex_id_buf.chunks_exact(4) {
            let vertex_id = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) as i32;
            shape_key_vertex_id_list.push(vertex_id);
            tmp_vertex_id_set.insert(vertex_id);
        }
        let _ = tmp_vertex_id_set;

        let mut shape_key_offset_list: Vec<i32> = Vec::new();
        let mut seen_map: HashMap<i32, i32> = HashMap::new();
        for chunk in shape_key_offset_buf.chunks_exact(4) {
            let offset_start = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) as i32;

            let count = seen_map.entry(offset_start).or_insert(0);
            if *count < 2 {
                *count += 1;
                shape_key_offset_list.push(offset_start);
            }
        }

        let mut shape_key_id_vertex_id_list_map: HashMap<i32, Vec<i32>> = HashMap::new();
        let mut shape_key_id_vertex_offset_list_map: HashMap<i32, Vec<Vec<u8>>> = HashMap::new();

        let mut shape_key_id: i32 = 0;
        for i in 0..shape_key_offset_list.len() {
            let start_offset = shape_key_offset_list[i].max(0) as usize;
            let end_offset = if shape_key_offset_list.len() > i + 1 {
                shape_key_offset_list[i + 1].max(0) as usize
            } else {
                shape_key_vertex_id_buf.len() / 4
            };

            if start_offset == end_offset {
                continue;
            }

            let safe_start = start_offset.min(shape_key_vertex_id_list.len());
            let safe_end = end_offset.min(shape_key_vertex_id_list.len());
            if safe_start >= safe_end {
                continue;
            }

            let shape_key_vertex_id_sub_list =
                shape_key_vertex_id_list[safe_start..safe_end].to_vec();

            let mut sub_vertex_offset_list: Vec<Vec<u8>> = Vec::new();
            for (j, offset_bytes) in shape_key_vertex_offset_list.iter().enumerate() {
                if j < start_offset || j > end_offset {
                    continue;
                }
                sub_vertex_offset_list.push(offset_bytes.clone());
            }

            shape_key_id_vertex_offset_list_map.insert(shape_key_id, sub_vertex_offset_list);
            shape_key_id_vertex_id_list_map.insert(shape_key_id, shape_key_vertex_id_sub_list);
            shape_key_id += 1;
        }

        let mut shape_key_id_vertex_id_vertex_offset_map: HashMap<i32, HashMap<i32, Vec<u8>>> =
            HashMap::new();

        for (shape_key_id_in, shape_key_vertex_id_sub_list) in &shape_key_id_vertex_id_list_map {
            let vertex_offset_list = shape_key_id_vertex_offset_list_map
                .get(shape_key_id_in)
                .cloned()
                .unwrap_or_default();

            let mut vertex_id_vertex_offset_map: HashMap<i32, Vec<u8>> = HashMap::new();
            for (i, vertex_id) in shape_key_vertex_id_sub_list.iter().enumerate() {
                if let Some(vertex_offset) = vertex_offset_list.get(i) {
                    vertex_id_vertex_offset_map.insert(*vertex_id, vertex_offset.clone());
                }
            }

            shape_key_id_vertex_id_vertex_offset_map
                .insert(*shape_key_id_in, vertex_id_vertex_offset_map);
        }

        Ok(Self {
            check_sum,
            shape_key_offset_number_last,
            shape_key_vertex_offset_count,
            shape_key_vertex_id_list,
            shape_key_id_vertex_id_list_map,
            shape_key_id_vertex_id_vertex_offset_map,
        })
    }

    pub fn get_shape_key_id_list(&self, divide_ib_buf_file: &IndexBufferBufFile) -> Vec<i32> {
        let mut shape_key_id_list: Vec<i32> = Vec::new();
        let ib_set: HashSet<u32> = divide_ib_buf_file.number_list.iter().copied().collect();

        for (shape_key_id, shape_key_vertex_id_list) in &self.shape_key_id_vertex_id_list_map {
            let mut found = false;
            for shape_key_vertex_id in shape_key_vertex_id_list {
                if (*shape_key_vertex_id as u32) < divide_ib_buf_file.min_number
                    || (*shape_key_vertex_id as u32) > divide_ib_buf_file.max_number
                {
                    continue;
                }

                if *shape_key_vertex_id >= 0 && ib_set.contains(&(*shape_key_vertex_id as u32)) {
                    found = true;
                    break;
                }
            }

            if found {
                shape_key_id_list.push(*shape_key_id);
            }
        }

        shape_key_id_list
    }

    pub fn get_shape_key_element_buffer_list(
        &self,
        vertex_count: i32,
        shape_key_id_list: &[i32],
        offset: i32,
    ) -> Vec<ElementBuffer> {
        let default_offset_data = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

        let mut shape_key_element_buffer_list: Vec<ElementBuffer> = Vec::new();

        for shape_key_id in shape_key_id_list {
            let mut element_byte_dict: HashMap<usize, Vec<u8>> = HashMap::new();

            for i in 0..vertex_count.max(0) as usize {
                let global_vertex_id = i as i32 + offset;

                let vertex_offset_data = self
                    .shape_key_id_vertex_id_vertex_offset_map
                    .get(shape_key_id)
                    .and_then(|m| m.get(&global_vertex_id))
                    .cloned()
                    .unwrap_or_else(|| default_offset_data.clone());

                element_byte_dict.insert(i, vertex_offset_data);
            }

            let d3d11_element = D3D11Element {
                semantic_name: "SHAPEKEY".to_string(),
                semantic_index: (*shape_key_id).max(0) as u64,
                format: "R16G16B16_FLOAT".to_string(),
                byte_width: "6".to_string(),
                element_name: if *shape_key_id <= 0 {
                    "SHAPEKEY".to_string()
                } else {
                    format!("SHAPEKEY{}", shape_key_id)
                },
                ..D3D11Element::default()
            };

            shape_key_element_buffer_list
                .push(ElementBuffer::new(d3d11_element, element_byte_dict));
        }

        shape_key_element_buffer_list
    }

    pub fn append_shape_key_to_final_vb0_buf(
        &self,
        final_vb0_buf: &[u8],
        shape_key_id_list: &[i32],
        total_stride: usize,
        offset: i32,
    ) -> Result<Vec<u8>, String> {
        if total_stride == 0 {
            return Err("total_stride must be greater than 0".to_string());
        }
        if final_vb0_buf.len() % total_stride != 0 {
            return Err(format!(
                "FinalVB0Buf length {} is not divisible by stride {}",
                final_vb0_buf.len(),
                total_stride
            ));
        }

        let default_offset_data = [0x00u8; 6];
        let row_count = final_vb0_buf.len() / total_stride;
        let append_bytes_per_row = shape_key_id_list.len() * default_offset_data.len();
        let mut output: Vec<u8> =
            Vec::with_capacity(row_count.saturating_mul(total_stride + append_bytes_per_row));

        for (row_index, row_bytes) in final_vb0_buf.chunks_exact(total_stride).enumerate() {
            output.extend_from_slice(row_bytes);
            let global_vertex_id = row_index as i32 + offset;

            for shape_key_id in shape_key_id_list {
                let vertex_offset_data = self
                    .shape_key_id_vertex_id_vertex_offset_map
                    .get(shape_key_id)
                    .and_then(|m| m.get(&global_vertex_id))
                    .map(|v| v.as_slice())
                    .unwrap_or(&default_offset_data);
                output.extend_from_slice(vertex_offset_data);
            }
        }

        Ok(output)
    }

    pub fn get_d3d11_element_list_with_shape_key(
        &self,
        d3d11_game_type: &D3D11GameType,
        shape_key_id_list: &[i32],
    ) -> Vec<D3D11Element> {
        let mut d3d11_element_list: Vec<D3D11Element> = Vec::new();

        for element_name in &d3d11_game_type.ordered_full_element_list {
            if let Some(d3d11_element) = d3d11_game_type
                .element_name_d3d11_element_dict
                .get(element_name)
            {
                d3d11_element_list.push(d3d11_element.clone());
            }
        }

        for shape_key_id in shape_key_id_list {
            let d3d11_element = D3D11Element {
                semantic_name: "SHAPEKEY".to_string(),
                semantic_index: (*shape_key_id).max(0) as u64,
                format: "R16G16B16_FLOAT".to_string(),
                byte_width: "6".to_string(),
                element_name: if *shape_key_id <= 0 {
                    "SHAPEKEY".to_string()
                } else {
                    format!("SHAPEKEY{}", shape_key_id)
                },
                ..D3D11Element::default()
            };
            d3d11_element_list.push(d3d11_element);
        }

        d3d11_element_list
    }
}
