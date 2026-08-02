use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::common::frame_analysis::frameanalysis_data::FrameAnalysisData;
use crate::utils::ssmt_file_utils::SSMTFileUtils;

#[derive(Debug, Clone, Default)]
pub struct VertexBufferCombFile {
    pub first_vertex: i32,
    pub byte_offset: i32,
    pub stride: i32,
    pub vertex_count: i32,
    pub byte_length: i32,

    pub category_buffer_bytes: Vec<u8>,

    pub txt_file_name: String,
    pub buf_file_name: String,

    pub buf_dict: HashMap<usize, Vec<u8>>,
}

impl VertexBufferCombFile {
    pub fn new(
        frame_analysis_folder_path: &str,
        trianglelist_index: &str,
        category_slot: &str,
    ) -> Result<Self, String> {
        let fa_data = FrameAnalysisData::new(frame_analysis_folder_path)?;

        let search_key = format!("{}-{}=", trianglelist_index, category_slot);

        let vb_txt_file_name = fa_data
            .filter_first_file(&search_key, ".txt")
            .unwrap_or_default();
        if vb_txt_file_name.is_empty() {
            return Err(format!(
                "无法找到匹配的 VB txt 文件: key={} folder={}",
                search_key, frame_analysis_folder_path
            ));
        }

        let vb_buf_file_name = fa_data
            .filter_first_file(&search_key, ".buf")
            .unwrap_or_default();
        if vb_buf_file_name.is_empty() {
            return Err(format!(
                "无法找到匹配的 VB buf 文件: key={} folder={}",
                search_key, frame_analysis_folder_path
            ));
        }

        let vb_txt_file_path = Path::new(frame_analysis_folder_path).join(&vb_txt_file_name);
        let vb_buf_file_path = Path::new(frame_analysis_folder_path).join(&vb_buf_file_name);

        if !vb_txt_file_path.exists() {
            return Err(format!(
                "txt 文件不存在: {}",
                vb_txt_file_path.to_string_lossy()
            ));
        }
        if !vb_buf_file_path.exists() {
            return Err(format!(
                "buf 文件不存在: {}",
                vb_buf_file_path.to_string_lossy()
            ));
        }

        let byte_offset =
            SSMTFileUtils::find_migoto_ini_attribute_in_file(&vb_txt_file_path, "byte offset")?
                .parse::<i32>()
                .unwrap_or(0);

        let stride = SSMTFileUtils::find_migoto_ini_attribute_in_file(&vb_txt_file_path, "stride")?
            .parse::<i32>()
            .unwrap_or(0);

        let vertex_count =
            SSMTFileUtils::find_migoto_ini_attribute_in_file(&vb_txt_file_path, "vertex count")?
                .parse::<i32>()
                .unwrap_or(0);

        let first_vertex =
            SSMTFileUtils::find_migoto_ini_attribute_in_file(&vb_txt_file_path, "first vertex")?
                .parse::<i32>()
                .unwrap_or(0);

        if stride <= 0 || vertex_count < 0 || first_vertex < 0 || byte_offset < 0 {
            return Err(format!(
                "无效的 VB 属性: stride={} vertex_count={} first_vertex={} byte_offset={}",
                stride, vertex_count, first_vertex, byte_offset
            ));
        }

        let vb_buf_bytes = fs::read(&vb_buf_file_path).map_err(|e| {
            format!(
                "读取 VB buf 文件失败 {}: {}",
                vb_buf_file_path.to_string_lossy(),
                e
            )
        })?;

        let byte_length = stride
            .checked_mul(vertex_count)
            .ok_or_else(|| "ByteLength 计算溢出".to_string())?;

        let byte_offset_usize =
            usize::try_from(byte_offset).map_err(|_| "byte_offset 转换失败".to_string())?;
        let byte_length_usize =
            usize::try_from(byte_length).map_err(|_| "byte_length 转换失败".to_string())?;
        let dst_offset_usize = usize::try_from(
            first_vertex
                .checked_mul(stride)
                .ok_or_else(|| "destination offset 计算溢出".to_string())?,
        )
        .map_err(|_| "destination offset 转换失败".to_string())?;

        if byte_offset_usize.saturating_add(byte_length_usize) > vb_buf_bytes.len() {
            return Err(format!(
                "源数据范围越界: offset={} len={} src_len={}",
                byte_offset_usize,
                byte_length_usize,
                vb_buf_bytes.len()
            ));
        }

        let mut category_buffer_bytes = vec![0u8; byte_length_usize];
        if dst_offset_usize.saturating_add(byte_length_usize) > category_buffer_bytes.len() {
            return Err(format!(
                "目标写入范围越界: dst_offset={} len={} dst_len={}",
                dst_offset_usize,
                byte_length_usize,
                category_buffer_bytes.len()
            ));
        }

        let src_slice = &vb_buf_bytes[byte_offset_usize..byte_offset_usize + byte_length_usize];
        let dst_slice =
            &mut category_buffer_bytes[dst_offset_usize..dst_offset_usize + byte_length_usize];
        dst_slice.copy_from_slice(src_slice);

        let buf_dict = Self::split_bytes_by_stride(
            &category_buffer_bytes,
            usize::try_from(stride).map_err(|_| "stride 转换失败".to_string())?,
        )?;

        Ok(Self {
            first_vertex,
            byte_offset,
            stride,
            vertex_count,
            byte_length,
            category_buffer_bytes,
            txt_file_name: vb_txt_file_name,
            buf_file_name: vb_buf_file_name,
            buf_dict,
        })
    }

    fn split_bytes_by_stride(
        bytes: &[u8],
        stride: usize,
    ) -> Result<HashMap<usize, Vec<u8>>, String> {
        if stride == 0 {
            return Err("stride must be greater than 0".to_string());
        }

        let mut result: HashMap<usize, Vec<u8>> = HashMap::new();
        for (i, chunk) in bytes.chunks(stride).enumerate() {
            result.insert(i, chunk.to_vec());
        }

        Ok(result)
    }
}
