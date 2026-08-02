use std::collections::HashMap;

use crate::common::d3d11_element::D3D11Element;

/// Record bytes for one element across vertices.
/// A list of ElementBuffer can be merged later to rebuild final vertex bytes and fmt output.
#[derive(Debug, Clone, Default)]
pub struct ElementBuffer {
    pub d3d11_element: D3D11Element,
    pub element_byte_dict: HashMap<usize, Vec<u8>>,
}

impl ElementBuffer {
    pub fn new(d3d11_element: D3D11Element, element_byte_dict: HashMap<usize, Vec<u8>>) -> Self {
        Self {
            d3d11_element,
            element_byte_dict,
        }
    }
}
