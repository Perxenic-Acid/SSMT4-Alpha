use std::fs;
use std::path::Path;

use crate::common::d3d11_element::D3D11Element;
use crate::common::d3d11_gametype::D3D11GameType;
use crate::constants::gametype::ExtractTechnique;

#[derive(Debug, Clone)]
pub struct FmtFile {
    pub game_type_name: String,
    pub topology: String,
    pub d3d11_element_list: Vec<D3D11Element>,
    pub format: String,
    pub prefix: String,
    pub scale: String,
    pub rotate_angle: bool,
    pub rotate_angle_x: String,
    pub rotate_angle_y: String,
    pub rotate_angle_z: String,
    pub logic_name: String,
    pub flip_face_orientation: bool,
    pub d3d11_game_type: D3D11GameType,
    pub element_name_list: Vec<String>,
    pub stride: u64,
}

impl FmtFile {
    pub fn new(d3d11_game_type: D3D11GameType) -> Self {
        let element_name_list = d3d11_game_type.ordered_full_element_list.clone();
        let stride = d3d11_game_type.get_element_list_total_stride(&element_name_list);

        Self {
            game_type_name: d3d11_game_type.game_type_name.clone(),
            topology: ExtractTechnique::TRIANGLELIST.to_string(),
            d3d11_element_list: Vec::new(),
            format: "DXGI_FORMAT_R32_UINT".to_string(),
            prefix: String::new(),
            scale: "1.0".to_string(),
            rotate_angle: false,
            rotate_angle_x: "0".to_string(),
            rotate_angle_y: "0".to_string(),
            rotate_angle_z: "0".to_string(),
            logic_name: String::new(),
            flip_face_orientation: false,
            d3d11_game_type,
            element_name_list,
            stride,
        }
    }

    pub fn output_fmt_file<P: AsRef<Path>>(
        &mut self,
        output_fmt_file_path: P,
    ) -> Result<(), String> {
        if self.stride == 0 {
            self.stride = self
                .d3d11_game_type
                .get_element_list_total_stride(&self.element_name_list);
        }

        let mut output_content: Vec<String> = Vec::new();
        output_content.push(format!("stride: {}", self.stride));
        output_content.push(format!("topology: {}", self.topology));
        output_content.push(format!("format: {}", self.format));
        output_content.push(format!("gametypename: {}", self.game_type_name));
        output_content.push(format!("prefix: {}", self.prefix));
        output_content.push(format!("scale: {}", self.scale));
        output_content.push(format!("rotate_angle: {}", self.rotate_angle));
        output_content.push(format!("rotate_angle_x: {}", self.rotate_angle_x));
        output_content.push(format!("rotate_angle_y: {}", self.rotate_angle_y));
        output_content.push(format!("rotate_angle_z: {}", self.rotate_angle_z));
        output_content.push(format!(
            "flip_face_orientation: {}",
            self.flip_face_orientation
        ));
        output_content.push(format!("logic_name: {}", self.logic_name));

        let mut element_number: u64 = 0;
        let mut aligned_byte_offset: u64 = 0;

        for element_name in &self.element_name_list {
            let d3d11_element = self
                .d3d11_game_type
                .element_name_d3d11_element_dict
                .get(element_name)
                .ok_or_else(|| format!("Element not found in game type: {}", element_name))?;

            output_content.push(format!("element[{}]:", element_number));
            output_content.push(format!("  SemanticName: {}", d3d11_element.semantic_name));
            output_content.push(format!("  SemanticIndex: {}", d3d11_element.semantic_index));
            output_content.push(format!("  Format: {}", d3d11_element.format));
            output_content.push(format!("  ByteWidth: {}", d3d11_element.byte_width));
            output_content.push("  InputSlot: 0".to_string());
            output_content.push(format!("  AlignedByteOffset: {}", aligned_byte_offset));

            aligned_byte_offset += d3d11_element.byte_width_int();
            output_content.push("  InputSlotClass: per-vertex".to_string());
            output_content.push("  InstanceDataStepRate: 0".to_string());

            element_number += 1;
        }

        fs::write(output_fmt_file_path.as_ref(), output_content.join("\n"))
            .map_err(|e| e.to_string())
    }

    pub fn output_fmt_file_by_d3d11_element_list<P: AsRef<Path>>(
        &mut self,
        output_fmt_path: P,
    ) -> Result<(), String> {
        let total_stride: u64 = self
            .d3d11_element_list
            .iter()
            .map(|d3d11_element| d3d11_element.byte_width_int())
            .sum();

        let mut output_content: Vec<String> = Vec::new();
        output_content.push(format!("stride: {}", total_stride));
        output_content.push(format!("topology: {}", self.topology));
        output_content.push(format!("format: {}", self.format));
        output_content.push(format!("gametypename: {}", self.game_type_name));
        output_content.push(format!("prefix: {}", self.prefix));
        output_content.push(format!("scale: {}", self.scale));
        output_content.push(format!("rotate_angle: {}", self.rotate_angle));
        output_content.push(format!("rotate_angle_x: {}", self.rotate_angle_x));
        output_content.push(format!("rotate_angle_y: {}", self.rotate_angle_y));
        output_content.push(format!("rotate_angle_z: {}", self.rotate_angle_z));
        output_content.push(format!(
            "flip_face_orientation: {}",
            self.flip_face_orientation
        ));
        output_content.push(format!("logic_name: {}", self.logic_name));

        let mut element_number: u64 = 0;
        let mut aligned_byte_offset: u64 = 0;
        for d3d11_element in &self.d3d11_element_list {
            output_content.push(format!("element[{}]:", element_number));
            output_content.push(format!("  SemanticName: {}", d3d11_element.semantic_name));
            output_content.push(format!("  SemanticIndex: {}", d3d11_element.semantic_index));
            output_content.push(format!("  Format: {}", d3d11_element.format));
            output_content.push(format!("  ByteWidth: {}", d3d11_element.byte_width));
            output_content.push("  InputSlot: 0".to_string());
            output_content.push(format!("  AlignedByteOffset: {}", aligned_byte_offset));

            aligned_byte_offset += d3d11_element.byte_width_int();
            output_content.push("  InputSlotClass: per-vertex".to_string());
            output_content.push("  InstanceDataStepRate: 0".to_string());

            element_number += 1;
        }

        fs::write(output_fmt_path.as_ref(), output_content.join("\n")).map_err(|e| e.to_string())
    }
}
