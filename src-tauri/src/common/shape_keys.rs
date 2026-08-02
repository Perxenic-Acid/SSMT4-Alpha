use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShapeKeys {
    pub offsets_hash: String,
    pub scale_hash: String,
    /// cs-t0 buffer 的 hash（ShapeKeyVertexId）
    pub vertex_ids_hash: String,
    /// cs-t1 buffer 的 hash（ShapeKeyVertexOffset）
    pub vertex_offsets_hash: String,
    /// ShapeKey 影响的顶点总数（从 cs-cb0 前128个u32的最后一项读取）
    pub vertex_count: i32,
    pub dispatch_y: i32,
    /// checksum = cb0_u32[0] + cb0_u32[1] + cb0_u32[2] + cb0_u32[3]
    pub checksum: i32,
}
