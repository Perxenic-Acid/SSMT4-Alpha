use crate::common::d3d11_gametype::D3D11GameType;

pub struct D3D11GameTypeHelper;

impl D3D11GameTypeHelper {
    pub fn get_element_name_offset(
        input_element_name: &str,
        element_name_list: &[String],
        d3d11_game_type: &D3D11GameType,
    ) -> u64 {
        let mut offset = 0_u64;
        for element_name in element_name_list {
            if element_name == input_element_name {
                break;
            }

            // Keep parity with the original C++ implementation.
            offset += d3d11_game_type
                .element_name_d3d11_element_dict
                .get(input_element_name)
                .map(|e| e.byte_width_int())
                .unwrap_or(0);
        }
        offset
    }
}
