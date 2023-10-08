#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XsmVec3d {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) z: f32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XsmQuaternion16 {
    pub(crate) x: i16,
    pub(crate) y: i16,
    pub(crate) z: i16,
    pub(crate) w: i16,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Xsm {
    pub(crate) header: XsmHeader,
    pub(crate) metadata: XsmMetadata,
    pub(crate) bone_animation: XsmBoneAnimation,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XsmHeader {
    pub(crate) magic: String,
    pub(crate) major_version: u8,
    pub(crate) minor_version: u8,
    pub(crate) big_endian: bool,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XsmChunk {
    pub(crate) chunk_type: i32,
    pub(crate) length: i32,
    pub(crate) version: i32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XsmMetadata {
    pub(crate) unused: f32,
    pub(crate) max_acceptable_error: f32,
    pub(crate) fps: i32,
    pub(crate) exporter_major_version: u8,
    pub(crate) exporter_minor_version: u8,
    pub(crate) source_app: String,
    pub(crate) original_filename: String,
    pub(crate) export_date: String,
    pub(crate) motion_name: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XsmBoneAnimation {
    pub(crate) num_submotion: i32,
    pub(crate) skeletal_submotion: Vec<XsmSubMotion>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XsmSubMotion {
    pub(crate) pose_rot: XsmQuaternion16,
    pub(crate) bind_pose_rot: XsmQuaternion16,
    pub(crate) pose_scale_rot: XsmQuaternion16,
    pub(crate) bind_pose_scale_rot: XsmQuaternion16,
    pub(crate) pose_pos: XsmVec3d,
    pub(crate) pose_scale: XsmVec3d,
    pub(crate) bind_pose_pos: XsmVec3d,
    pub(crate) bind_pose_scale_pos: XsmVec3d,
    pub(crate) num_pos_keys: i32,
    pub(crate) num_rot_keys: i32,
    pub(crate) num_scale_keys: i32,
    pub(crate) num_scale_rot_keys: i32,
    pub(crate) max_error: f32,
    pub(crate) node_name: String,
    pub(crate) pos_key: Vec<XsmPosKey>,
    pub(crate) rot_key: Vec<XsmRotKey>,
    pub(crate) scale_key: Vec<XsmScaleKey>,
    pub(crate) scale_rot_key: Vec<XsmScaleRotKey>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XsmPosKey {
    pub(crate) pos: XsmVec3d,
    pub(crate) time: f32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XsmRotKey {
    pub(crate) rot: XsmQuaternion16,
    pub(crate) time: f32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XsmScaleKey {
    pub(crate) scale: XsmVec3d,
    pub(crate) time: f32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XsmScaleRotKey {
    pub(crate) rot: XsmQuaternion16,
    pub(crate) time: f32,
}
