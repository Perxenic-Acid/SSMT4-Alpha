use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct D3D11Element {
    #[serde(rename = "SemanticName")]
    pub semantic_name: String,

    #[serde(rename = "Format")]
    pub format: String,

    #[serde(rename = "ExtractSlot")]
    pub extract_slot: String,

    #[serde(rename = "ExtractTechnique")]
    pub extract_technique: String,

    #[serde(rename = "Category")]
    pub category: String,

    #[serde(rename = "DrawCategory")]
    pub draw_category: String,

    /// 可选字段：缺失时反序列化为 ""，后续由 D3D11GameType::initialize 回退计算。
    #[serde(rename = "ByteWidth", default)]
    pub byte_width: String,

    /// 在列表中的次数/顺序得到的索引（内部使用，不参与序列化）。
    #[serde(skip_serializing, skip_deserializing)]
    pub semantic_index: u64,

    /// 内部使用的名称（不参与序列化）。
    #[serde(skip_serializing, skip_deserializing)]
    pub element_name: String,
}

impl D3D11Element {
    pub fn new(
        semantic_name: impl Into<String>,
        format: impl Into<String>,
        extract_slot: impl Into<String>,
        extract_technique: impl Into<String>,
        category: impl Into<String>,
        draw_category: impl Into<String>,
        byte_width: impl Into<String>,
    ) -> Self {
        Self {
            semantic_name: semantic_name.into(),
            format: format.into(),
            extract_slot: extract_slot.into(),
            extract_technique: extract_technique.into(),
            category: category.into(),
            draw_category: draw_category.into(),
            byte_width: byte_width.into(),
            semantic_index: 0,
            element_name: String::new(),
        }
    }

    /// 尝试把 ByteWidth 解析为整数，失败则返回 0。
    pub fn byte_width_int(&self) -> u64 {
        self.byte_width.parse::<u64>().unwrap_or(0)
    }
}
