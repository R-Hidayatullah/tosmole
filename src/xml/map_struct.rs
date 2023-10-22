#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct World {
    pub(crate) model_dir: ModelDir,
    pub(crate) sub_model_dir: SubModelDir,
    pub(crate) texture_dir: TextureDir,
    pub(crate) model: Vec<Model>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct ModelDir {
    pub(crate) ipf_name: String,
    pub(crate) ipf_path: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct SubModelDir {
    pub(crate) ipf_name: String,
    pub(crate) ipf_path: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct TextureDir {
    pub(crate) ipf_name: String,
    pub(crate) ipf_path: String,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Model {
    pub(crate) filename: String,
    pub(crate) model_name: String,
    pub(crate) position: [f32; 3],
    pub(crate) rotation: [f32; 4],
    pub(crate) scale: [f32; 3],
}
