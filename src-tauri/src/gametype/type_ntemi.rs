use crate::common::d3d11_element::D3D11Element;
use crate::common::d3d11_gametype::D3D11GameType;
use crate::constants::gametype::{
    CategoryName, DxgiFormat, ElementName, ExtractSlot, ExtractTechnique,
};

#[derive(Debug, Clone)]
pub struct D3D11GameTypeWrapper {
    pub d3d11gametype: D3D11GameType,
    pub pointlist_index: String,
    pub matched_trianglelistindex: String,
}

impl D3D11GameTypeWrapper {
    pub fn new(
        d3d11gametype: D3D11GameType,
        pointlist_index: impl Into<String>,
        matched_trianglelistindex: impl Into<String>,
    ) -> Self {
        Self {
            d3d11gametype,
            pointlist_index: pointlist_index.into(),
            matched_trianglelistindex: matched_trianglelistindex.into(),
        }
    }
}

pub struct NTEMIGameType;

impl NTEMIGameType {
    pub fn initialize() -> Vec<D3D11GameType> {
        vec![
            //
            // NTEMI GPU-PreSkinning 数据类型
            //
            // 流水线: CS(compute shader) skinning → VS(vertex shader) render
            //
            // CS stage (pointlist):
            //   cs-t1: Blend    - BLENDINDICES(R8G8B8A8_UINT) + BLENDWEIGHTS(R8G8B8A8_UNORM) = 8 bytes/vertex
            //   cs-t2: Normal   - TANGENT(R8G8B8A8_SNORM) + NORMAL(R8G8B8A8_SNORM) = 8 bytes/vertex
            //                      Frame A: tangent.xyz + 1.0, Frame B: normal.xyz + bitangent_sign
            //   cs-t3: Position - POSITION(R32G32B32_FLOAT) = 12 bytes/vertex
            //
            // VS stage (trianglelist):
            //   vs-t5: Texcoord - TEXCOORD x4 (R16G16_FLOAT each) = 16 bytes/vertex
            //                      Packed as 4 half2 UV sets (UV0-UV3)
            //   vs-t8: Color    - COLOR(R8B8G8A8_UNORM) = 4 bytes/vertex (outline params)
            //
            // cb4: BoneMatrix - 48 bytes/bone (float3x4 matrix), vs-cb4 buffer
            // IB:  R32_UINT or R16_UINT
            //

            // --- 有轮廓线 (Outline) 的变体 ---
            D3D11GameType::from_parts(
                "GPU_P12_BI8_BW8_T8_T1-8_TA4_N4_",
                vec![
                    // cs-t1: Blend indices + weights (pointlist, 8 bytes/vertex)
                    D3D11Element::new(
                        ElementName::BLENDINDICES,
                        DxgiFormat::R8G8B8A8_UINT,
                        ExtractSlot::CS_T1,
                        ExtractTechnique::POINTLIST,
                        CategoryName::BLEND,
                        CategoryName::BLEND,
                        "4",
                    ),
                    D3D11Element::new(
                        ElementName::BLENDWEIGHTS,
                        DxgiFormat::R8G8B8A8_UNORM,
                        ExtractSlot::CS_T1,
                        ExtractTechnique::POINTLIST,
                        CategoryName::BLEND,
                        CategoryName::BLEND,
                        "4",
                    ),
                    // cs-t2: Tangent frame pre-CS (pointlist, 8 bytes/vertex)
                    D3D11Element::new(
                        ElementName::TANGENT,
                        DxgiFormat::R8G8B8A8_SNORM,
                        ExtractSlot::CS_T2,
                        ExtractTechnique::POINTLIST,
                        CategoryName::NORMAL,
                        CategoryName::NORMAL,
                        "4",
                    ),
                    D3D11Element::new(
                        ElementName::NORMAL,
                        DxgiFormat::R8G8B8A8_SNORM,
                        ExtractSlot::CS_T2,
                        ExtractTechnique::POINTLIST,
                        CategoryName::NORMAL,
                        CategoryName::NORMAL,
                        "4",
                    ),
                    // cs-t3: Position (pointlist, 12 bytes/vertex)
                    D3D11Element::new(
                        ElementName::POSITION,
                        DxgiFormat::R32G32B32_FLOAT,
                        ExtractSlot::CS_T3,
                        ExtractTechnique::POINTLIST,
                        CategoryName::POSITION,
                        CategoryName::POSITION,
                        "12",
                    ),
                    // vs-t5: UV layers (trianglelist, 16 bytes/vertex = 4 x half2)
                    D3D11Element::new(
                        ElementName::TEXCOORD,
                        DxgiFormat::R16G16_FLOAT,
                        ExtractSlot::VS_T5,
                        ExtractTechnique::TRIANGLELIST,
                        CategoryName::TEXCOORD,
                        CategoryName::TEXCOORD,
                        "4",
                    ),
                    D3D11Element::new(
                        ElementName::TEXCOORD,
                        DxgiFormat::R16G16_FLOAT,
                        ExtractSlot::VS_T5,
                        ExtractTechnique::TRIANGLELIST,
                        CategoryName::TEXCOORD,
                        CategoryName::TEXCOORD,
                        "4",
                    ),
                    D3D11Element::new(
                        ElementName::TEXCOORD,
                        DxgiFormat::R16G16_FLOAT,
                        ExtractSlot::VS_T5,
                        ExtractTechnique::TRIANGLELIST,
                        CategoryName::TEXCOORD,
                        CategoryName::TEXCOORD,
                        "4",
                    ),
                    D3D11Element::new(
                        ElementName::TEXCOORD,
                        DxgiFormat::R16G16_FLOAT,
                        ExtractSlot::VS_T5,
                        ExtractTechnique::TRIANGLELIST,
                        CategoryName::TEXCOORD,
                        CategoryName::TEXCOORD,
                        "4",
                    ),
                    // vs-t8: Outline/color params (trianglelist, 4 bytes/vertex)
                    D3D11Element::new(
                        ElementName::COLOR,
                        DxgiFormat::R8B8G8A8_UNORM,
                        ExtractSlot::VS_T8,
                        ExtractTechnique::TRIANGLELIST,
                        CategoryName::COLOR,
                        CategoryName::COLOR,
                        "4",
                    ),
                ],
            ),
            // --- 无轮廓线 (No Outline) 的变体 ---
            D3D11GameType::from_parts(
                "GPU_P12_BI8_BW8_T8_T1-8_TA4_N4_NoOutline_",
                vec![
                    D3D11Element::new(
                        ElementName::BLENDINDICES,
                        DxgiFormat::R8G8B8A8_UINT,
                        ExtractSlot::CS_T1,
                        ExtractTechnique::POINTLIST,
                        CategoryName::BLEND,
                        CategoryName::BLEND,
                        "4",
                    ),
                    D3D11Element::new(
                        ElementName::BLENDWEIGHTS,
                        DxgiFormat::R8G8B8A8_UNORM,
                        ExtractSlot::CS_T1,
                        ExtractTechnique::POINTLIST,
                        CategoryName::BLEND,
                        CategoryName::BLEND,
                        "4",
                    ),
                    D3D11Element::new(
                        ElementName::TANGENT,
                        DxgiFormat::R8G8B8A8_SNORM,
                        ExtractSlot::CS_T2,
                        ExtractTechnique::POINTLIST,
                        CategoryName::NORMAL,
                        CategoryName::NORMAL,
                        "4",
                    ),
                    D3D11Element::new(
                        ElementName::NORMAL,
                        DxgiFormat::R8G8B8A8_SNORM,
                        ExtractSlot::CS_T2,
                        ExtractTechnique::POINTLIST,
                        CategoryName::NORMAL,
                        CategoryName::NORMAL,
                        "4",
                    ),
                    D3D11Element::new(
                        ElementName::POSITION,
                        DxgiFormat::R32G32B32_FLOAT,
                        ExtractSlot::CS_T3,
                        ExtractTechnique::POINTLIST,
                        CategoryName::POSITION,
                        CategoryName::POSITION,
                        "12",
                    ),
                    D3D11Element::new(
                        ElementName::TEXCOORD,
                        DxgiFormat::R16G16_FLOAT,
                        ExtractSlot::VS_T5,
                        ExtractTechnique::TRIANGLELIST,
                        CategoryName::TEXCOORD,
                        CategoryName::TEXCOORD,
                        "4",
                    ),
                    D3D11Element::new(
                        ElementName::TEXCOORD,
                        DxgiFormat::R16G16_FLOAT,
                        ExtractSlot::VS_T5,
                        ExtractTechnique::TRIANGLELIST,
                        CategoryName::TEXCOORD,
                        CategoryName::TEXCOORD,
                        "4",
                    ),
                    D3D11Element::new(
                        ElementName::TEXCOORD,
                        DxgiFormat::R16G16_FLOAT,
                        ExtractSlot::VS_T5,
                        ExtractTechnique::TRIANGLELIST,
                        CategoryName::TEXCOORD,
                        CategoryName::TEXCOORD,
                        "4",
                    ),
                    D3D11Element::new(
                        ElementName::TEXCOORD,
                        DxgiFormat::R16G16_FLOAT,
                        ExtractSlot::VS_T5,
                        ExtractTechnique::TRIANGLELIST,
                        CategoryName::TEXCOORD,
                        CategoryName::TEXCOORD,
                        "4",
                    ),
                    // No Color/Outline slot in this variant
                ],
            ),
            // --- CPU-PreSkinning (Post-CS) 数据类型 ---
            //
            // 流水线: CPU skinning → VS(vertex shader) render
            // 所有 buffer 均为 trianglelist，无需 CS stage
            //
            // vb0:   Position - POSITION(R32G32B32_FLOAT) = 12 bytes/vertex (已蒙皮)
            // vb1:   Texcoord - TEXCOORD x4 (R16G16_FLOAT each) = 16 bytes/vertex
            // vs-t7: Normal   - TANGENT(R8G8B8A8_SNORM) + NORMAL(R8G8B8A8_SNORM) = 8 bytes/vertex (post-CS)
            // vs-t8: Color    - COLOR(R8B8G8A8_UNORM) = 4 bytes/vertex (outline params, optional)
            //
            // IB:  R32_UINT or R16_UINT

            // --- 有轮廓线 (Outline) 的 Post-CS 变体 ---
            D3D11GameType::from_parts(
                "CPU_P12_T8_TA4_N4_",
                vec![
                    // vb0: Skinned Position (trianglelist, 12 bytes/vertex)
                    D3D11Element::new(
                        ElementName::POSITION,
                        DxgiFormat::R32G32B32_FLOAT,
                        ExtractSlot::VB0,
                        ExtractTechnique::TRIANGLELIST,
                        CategoryName::POSITION,
                        CategoryName::POSITION,
                        "12",
                    ),
                    // vb1: UV layers (trianglelist, 16 bytes/vertex = 4 x half2)
                    D3D11Element::new(
                        ElementName::TEXCOORD,
                        DxgiFormat::R16G16_FLOAT,
                        ExtractSlot::VB1,
                        ExtractTechnique::TRIANGLELIST,
                        CategoryName::TEXCOORD,
                        CategoryName::TEXCOORD,
                        "4",
                    ),
                    D3D11Element::new(
                        ElementName::TEXCOORD,
                        DxgiFormat::R16G16_FLOAT,
                        ExtractSlot::VB1,
                        ExtractTechnique::TRIANGLELIST,
                        CategoryName::TEXCOORD,
                        CategoryName::TEXCOORD,
                        "4",
                    ),
                    D3D11Element::new(
                        ElementName::TEXCOORD,
                        DxgiFormat::R16G16_FLOAT,
                        ExtractSlot::VB1,
                        ExtractTechnique::TRIANGLELIST,
                        CategoryName::TEXCOORD,
                        CategoryName::TEXCOORD,
                        "4",
                    ),
                    D3D11Element::new(
                        ElementName::TEXCOORD,
                        DxgiFormat::R16G16_FLOAT,
                        ExtractSlot::VB1,
                        ExtractTechnique::TRIANGLELIST,
                        CategoryName::TEXCOORD,
                        CategoryName::TEXCOORD,
                        "4",
                    ),
                    // vs-t7: Tangent frame post-CS (trianglelist, 8 bytes/vertex)
                    D3D11Element::new(
                        ElementName::TANGENT,
                        DxgiFormat::R8G8B8A8_SNORM,
                        ExtractSlot::VS_T7,
                        ExtractTechnique::TRIANGLELIST,
                        CategoryName::NORMAL,
                        CategoryName::NORMAL,
                        "4",
                    ),
                    D3D11Element::new(
                        ElementName::NORMAL,
                        DxgiFormat::R8G8B8A8_SNORM,
                        ExtractSlot::VS_T7,
                        ExtractTechnique::TRIANGLELIST,
                        CategoryName::NORMAL,
                        CategoryName::NORMAL,
                        "4",
                    ),
                    // vs-t8: Outline/color params (trianglelist, 4 bytes/vertex)
                    D3D11Element::new(
                        ElementName::COLOR,
                        DxgiFormat::R8B8G8A8_UNORM,
                        ExtractSlot::VS_T8,
                        ExtractTechnique::TRIANGLELIST,
                        CategoryName::COLOR,
                        CategoryName::COLOR,
                        "4",
                    ),
                ],
            ),
            // D3D11GameType::from_parts("CPU_P12_T4_T14_TA4_N4_", vec![
            //     D3D11Element::new(ElementName::POSITION, DxgiFormat::R32G32B32_FLOAT, ExtractSlot::VB0, ExtractTechnique::TRIANGLELIST, CategoryName::POSITION, CategoryName::POSITION, "12"),

            //     D3D11Element::new(ElementName::TEXCOORD, DxgiFormat::R16G16_FLOAT, ExtractSlot::VS_T0, ExtractTechnique::TRIANGLELIST, CategoryName::TEXCOORD, CategoryName::TEXCOORD, "4"),
            //     D3D11Element::new(ElementName::TEXCOORD, DxgiFormat::R16G16_FLOAT, ExtractSlot::VS_T0, ExtractTechnique::TRIANGLELIST, CategoryName::TEXCOORD, CategoryName::TEXCOORD, "4"),

            //     D3D11Element::new(ElementName::TANGENT, DxgiFormat::R8G8B8A8_SNORM, ExtractSlot::VS_T1, ExtractTechnique::TRIANGLELIST, CategoryName::NORMAL, CategoryName::NORMAL, "4"),
            //     D3D11Element::new(ElementName::NORMAL, DxgiFormat::R8G8B8A8_SNORM, ExtractSlot::VS_T1, ExtractTechnique::TRIANGLELIST, CategoryName::NORMAL, CategoryName::NORMAL, "4"),

            // ]),

            // --- 无轮廓线 (No Outline) 的 Post-CS 变体 ---
            D3D11GameType::from_parts(
                "CPU_P12_T8_TA4_N4_NoOutline_",
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
                        ElementName::TEXCOORD,
                        DxgiFormat::R16G16_FLOAT,
                        ExtractSlot::VB1,
                        ExtractTechnique::TRIANGLELIST,
                        CategoryName::TEXCOORD,
                        CategoryName::TEXCOORD,
                        "4",
                    ),
                    D3D11Element::new(
                        ElementName::TEXCOORD,
                        DxgiFormat::R16G16_FLOAT,
                        ExtractSlot::VB1,
                        ExtractTechnique::TRIANGLELIST,
                        CategoryName::TEXCOORD,
                        CategoryName::TEXCOORD,
                        "4",
                    ),
                    D3D11Element::new(
                        ElementName::TEXCOORD,
                        DxgiFormat::R16G16_FLOAT,
                        ExtractSlot::VB1,
                        ExtractTechnique::TRIANGLELIST,
                        CategoryName::TEXCOORD,
                        CategoryName::TEXCOORD,
                        "4",
                    ),
                    D3D11Element::new(
                        ElementName::TEXCOORD,
                        DxgiFormat::R16G16_FLOAT,
                        ExtractSlot::VB1,
                        ExtractTechnique::TRIANGLELIST,
                        CategoryName::TEXCOORD,
                        CategoryName::TEXCOORD,
                        "4",
                    ),
                    D3D11Element::new(
                        ElementName::TANGENT,
                        DxgiFormat::R8G8B8A8_SNORM,
                        ExtractSlot::VS_T7,
                        ExtractTechnique::TRIANGLELIST,
                        CategoryName::NORMAL,
                        CategoryName::NORMAL,
                        "4",
                    ),
                    D3D11Element::new(
                        ElementName::NORMAL,
                        DxgiFormat::R8G8B8A8_SNORM,
                        ExtractSlot::VS_T7,
                        ExtractTechnique::TRIANGLELIST,
                        CategoryName::NORMAL,
                        CategoryName::NORMAL,
                        "4",
                    ),
                    // No Color/Outline slot in this variant
                ],
            ),
        ]
    }
}
