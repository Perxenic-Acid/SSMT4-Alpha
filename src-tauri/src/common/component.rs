use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Component {
    pub vertex_offset: i32,
    pub vertex_count: i32,
    pub index_offset: i32,
    pub index_count: i32,
    pub vg_offset: i32,
    pub vg_count: i32,
    pub vg_map: HashMap<String, i32>,
}
