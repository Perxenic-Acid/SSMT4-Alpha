use crate::common::d3d11_element::D3D11Element;
use crate::common::d3d11_gametype::D3D11GameType;
use crate::constants::gametype::{
    CategoryName, DxgiFormat, ElementName, ExtractSlot, ExtractTechnique,
};

pub struct NeirRGameType;

impl NeirRGameType {
    pub fn initialize() -> Vec<D3D11GameType> {
        vec![D3D11GameType::from_parts(
            "GPU_P12_C4_T4_BI4_BW4_",
            vec![
                D3D11Element::new(
                    ElementName::POSITION,
                    DxgiFormat::R32G32B32_FLOAT,
                    ExtractSlot::VB0,
                    ExtractTechnique::TRIANGLELIST,
                    CategoryName::POSITION,
                    CategoryName::POSITION,
                    "12",
                ),
                D3D11Element::new(
                    ElementName::COLOR,
                    DxgiFormat::R8G8B8A8_UNORM,
                    ExtractSlot::VB0,
                    ExtractTechnique::TRIANGLELIST,
                    CategoryName::POSITION,
                    CategoryName::POSITION,
                    "4",
                ),
                D3D11Element::new(
                    ElementName::TEXCOORD,
                    DxgiFormat::R16G16_FLOAT,
                    ExtractSlot::VB0,
                    ExtractTechnique::TRIANGLELIST,
                    CategoryName::POSITION,
                    CategoryName::POSITION,
                    "4",
                ),
                D3D11Element::new(
                    ElementName::BLENDINDICES,
                    DxgiFormat::R8G8B8A8_UINT,
                    ExtractSlot::VB0,
                    ExtractTechnique::TRIANGLELIST,
                    CategoryName::POSITION,
                    CategoryName::POSITION,
                    "4",
                ),
                D3D11Element::new(
                    ElementName::BLENDWEIGHTS,
                    DxgiFormat::R8G8B8A8_UNORM,
                    ExtractSlot::VB0,
                    ExtractTechnique::TRIANGLELIST,
                    CategoryName::POSITION,
                    CategoryName::POSITION,
                    "4",
                ),
            ],
        )]
    }
}
