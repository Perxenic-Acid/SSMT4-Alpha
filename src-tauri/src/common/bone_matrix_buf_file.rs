use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct BoneMatrixBufFile {
    // Bone matrix is 3x4 float matrix per blend index: 12 * 4 = 48 bytes.
    pub bone_matrix_buf: Vec<u8>,
}

impl BoneMatrixBufFile {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_file(file_path: impl AsRef<Path>) -> Result<Self, String> {
        let bone_matrix_buf = fs::read(file_path.as_ref()).map_err(|e| {
            format!(
                "Failed to read bone matrix buf file {}: {}",
                file_path.as_ref().to_string_lossy(),
                e
            )
        })?;

        Ok(Self { bone_matrix_buf })
    }

    pub fn get_buf_data_by_blend_index(&self, blend_index: usize) -> Vec<u8> {
        const MATRIX_STRIDE: usize = 48;
        let start_index = blend_index.saturating_mul(MATRIX_STRIDE);
        let end_index = start_index.saturating_add(MATRIX_STRIDE);

        if end_index > self.bone_matrix_buf.len() {
            return Vec::new();
        }

        self.bone_matrix_buf[start_index..end_index].to_vec()
    }
}
