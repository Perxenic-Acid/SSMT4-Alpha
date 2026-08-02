use serde::{Deserialize, Serialize};

use crate::common::d3d11_gametype::D3D11GameType;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct D3D11GameTypeWrapper {
    #[serde(rename = "PositionExtractSlot")]
    pub position_extract_slot: String,

    #[serde(rename = "PositionExtractIndex")]
    pub position_extract_index: String,

    #[serde(rename = "BlendExtractSlot")]
    pub blend_extract_slot: String,

    #[serde(rename = "BlendExtractIndex")]
    pub blend_extract_index: String,

    #[serde(rename = "d3d11GameType")]
    pub d3d11_game_type: D3D11GameType,
}

impl D3D11GameTypeWrapper {
    pub fn new(d3d11_game_type: D3D11GameType) -> Self {
        Self {
            position_extract_slot: String::new(),
            position_extract_index: String::new(),
            blend_extract_slot: String::new(),
            blend_extract_index: String::new(),
            d3d11_game_type,
        }
    }
}
