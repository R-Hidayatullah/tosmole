use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct BevyMesh {
    pub(crate) name_texture: String,
    pub(crate) positions: Vec<[f32; 3]>,
    pub(crate) normals: Vec<[f32; 3]>,
    pub(crate) tangents: Vec<[f32; 4]>,
    pub(crate) bi_tangents: Vec<[f32; 4]>,
    pub(crate) uv_set: Vec<[f32; 2]>,
    pub(crate) mesh_indices: Vec<u32>,
}
