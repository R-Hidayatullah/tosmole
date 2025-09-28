use binrw::{BinRead, BinReaderExt, binread};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub enum XacChunkData {
    XacInfo(XacInfo),
    XacInfo2(XacInfo2),
    XacInfo3(XacInfo3),
    XacInfo4(XacInfo4),

    XacNode(XacNode),
    XacNode2(XacNode2),
    XacNode3(XacNode3),
    XacNode4(XacNode4),

    XacSkinningInfo(XacSkinningInfo),
    XacSkinningInfo2(XacSkinningInfo2),
    XacSkinningInfo3(XacSkinningInfo3),
    XacSkinningInfo4(XacSkinningInfo4),

    XacStandardMaterial(XacStandardMaterial),
    XacStandardMaterial2(XacStandardMaterial2),
    XacStandardMaterial3(XacStandardMaterial3),

    XACStandardMaterialLayer(XACStandardMaterialLayer),
    XACStandardMaterialLayer2(XACStandardMaterialLayer2),

    XACSubMesh(XACSubMesh),
    XACMesh(XACMesh),
    XACMesh2(XACMesh2),

    XACLimit(XACLimit),
    XACPMorphTarget(XACPMorphTarget),
    XACPMorphTargets(XACPMorphTargets),

    XACFXMaterial(XACFXMaterial),
    XACFXMaterial2(XACFXMaterial2),
    XACFXMaterial3(XACFXMaterial3),

    XACNodeGroup(XACNodeGroup),
    XACNodes(XACNodes),

    XACMaterialInfo(XACMaterialInfo),
    XACMaterialInfo2(XACMaterialInfo2),

    XACMeshLodLevel(XACMeshLodLevel),

    XACNodeMotionSources(XACNodeMotionSources),
    XACAttachmentNodes(XACAttachmentNodes),
}

/// File chunk header
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct FileChunk {
    /// The chunk identifier
    pub chunk_id: u32,
    /// The size in bytes of this chunk (excluding this chunk struct)
    pub size_in_bytes: u32,
    /// The version of the chunk
    pub version: u32,
}

/// RGBA color with values in [0..1] range
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct FileColor {
    /// Red component
    pub r: f32,
    /// Green component
    pub g: f32,
    /// Blue component
    pub b: f32,
    /// Alpha component
    pub a: f32,
}

/// 3D vector with 32-bit floating point components
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct FileVector3 {
    /// X coordinate (positive = to the right)
    pub x: f32,
    /// Y coordinate (positive = up)
    pub y: f32,
    /// Z coordinate (positive = forwards into the depth)
    pub z: f32,
}

/// Compressed 3D vector with 16-bit integer components
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct File16BitVector3 {
    /// X coordinate (positive = to the right)
    pub x: u16,
    /// Y coordinate (positive = up)
    pub y: u16,
    /// Z coordinate (positive = forwards into the depth)
    pub z: u16,
}

/// Compressed 3D vector with 8-bit integer components
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct File8BitVector3 {
    /// X coordinate (positive = to the right)
    pub x: u8,
    /// Y coordinate (positive = up)
    pub y: u8,
    /// Z coordinate (positive = forwards into the depth)
    pub z: u8,
}

/// Quaternion with 32-bit floating point components
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct FileQuaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

/// Compressed quaternion with 16-bit signed integer components
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct File16BitQuaternion {
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub w: i16,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XacHeader {
    pub fourcc: u32,     // Must be "XAC "
    pub hi_version: u8,  // High version (e.g., 2 in v2.34)
    pub lo_version: u8,  // Low version (e.g., 34 in v2.34)
    pub endian_type: u8, // Endianness: 0 = little, 1 = big
    pub mul_order: u8,   // See enum MULORDER_...
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XacInfo {
    pub repositioning_mask: u32,
    pub repositioning_node_index: u32,
    pub exporter_high_version: u8,
    pub exporter_low_version: u8,
    pub padding: u16,

    #[br(temp)]
    pub source_app_length: u32,
    #[br(count = source_app_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub source_app: String,

    #[br(temp)]
    pub original_filename_length: u32,
    #[br(count = original_filename_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub original_filename: String,

    #[br(temp)]
    pub compilation_date_length: u32,
    #[br(count = compilation_date_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub compilation_date: String,

    #[br(temp)]
    pub actor_name_length: u32,
    #[br(count = actor_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub actor_name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XacInfo2 {
    pub repositioning_mask: u32,
    pub repositioning_node_index: u32,
    pub exporter_high_version: u8,
    pub exporter_low_version: u8,
    pub retarget_root_offset: f32,
    pub padding: u16,

    #[br(temp)]
    pub source_app_length: u32,
    #[br(count = source_app_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub source_app: String,

    #[br(temp)]
    pub original_filename_length: u32,
    #[br(count = original_filename_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub original_filename: String,

    #[br(temp)]
    pub compilation_date_length: u32,
    #[br(count = compilation_date_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub compilation_date: String,

    #[br(temp)]
    pub actor_name_length: u32,
    #[br(count = actor_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub actor_name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XacInfo3 {
    pub trajectory_node_index: u32,
    pub motion_extraction_node_index: u32,
    pub motion_extraction_mask: u32,
    pub exporter_high_version: u8,
    pub exporter_low_version: u8,
    pub retarget_root_offset: f32,
    pub padding: u16,

    #[br(temp)]
    pub source_app_length: u32,
    #[br(count = source_app_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub source_app: String,

    #[br(temp)]
    pub original_filename_length: u32,
    #[br(count = original_filename_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub original_filename: String,

    #[br(temp)]
    pub compilation_date_length: u32,
    #[br(count = compilation_date_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub compilation_date: String,

    #[br(temp)]
    pub actor_name_length: u32,
    #[br(count = actor_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub actor_name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XacInfo4 {
    pub num_lods: u32,
    pub trajectory_node_index: u32,
    pub motion_extraction_node_index: u32,
    pub exporter_high_version: u8,
    pub exporter_low_version: u8,
    pub retarget_root_offset: f32,
    pub padding: u16,

    #[br(temp)]
    pub source_app_length: u32,
    #[br(count = source_app_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub source_app: String,

    #[br(temp)]
    pub original_filename_length: u32,
    #[br(count = original_filename_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub original_filename: String,

    #[br(temp)]
    pub compilation_date_length: u32,
    #[br(count = compilation_date_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub compilation_date: String,

    #[br(temp)]
    pub actor_name_length: u32,
    #[br(count = actor_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub actor_name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XacNode {
    pub local_quat: FileQuaternion,
    pub scale_rot: FileQuaternion,
    pub local_pos: FileVector3,
    pub local_scale: FileVector3,
    pub shear: FileVector3,
    pub skeletal_lods: u32,
    pub parent_index: u32,

    #[br(temp)]
    pub node_name_length: u32,
    #[br(count = node_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub node_name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XacNode2 {
    pub local_quat: FileQuaternion,
    pub scale_rot: FileQuaternion,
    pub local_pos: FileVector3,
    pub local_scale: FileVector3,
    pub shear: FileVector3,
    pub skeletal_lods: u32,
    pub parent_index: u32,
    pub node_flags: u8,
    pub padding: [u8; 3],

    #[br(temp)]
    pub node_name_length: u32,
    #[br(count = node_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub node_name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XacNode3 {
    pub local_quat: FileQuaternion,
    pub scale_rot: FileQuaternion,
    pub local_pos: FileVector3,
    pub local_scale: FileVector3,
    pub shear: FileVector3,
    pub skeletal_lods: u32,
    pub parent_index: u32,
    pub node_flags: u8,
    pub obb: [f32; 16], // Oriented Bounding Box (OBB)
    pub padding: [u8; 3],

    #[br(temp)]
    pub node_name_length: u32,
    #[br(count = node_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub node_name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XacNode4 {
    pub local_quat: FileQuaternion,
    pub scale_rot: FileQuaternion,
    pub local_pos: FileVector3,
    pub local_scale: FileVector3,
    pub shear: FileVector3,
    pub skeletal_lods: u32,
    pub motion_lods: u32,
    pub parent_index: u32,
    pub num_children: u32,
    pub node_flags: u8,
    pub obb: [f32; 16],         // Oriented Bounding Box (OBB)
    pub importance_factor: f32, // Used for automatic motion LOD
    pub padding: [u8; 3],

    #[br(temp)]
    pub node_name_length: u32,
    #[br(count = node_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub node_name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACMeshLodLevel {
    pub lod_level: u32,
    pub size_in_bytes: u32,

    #[br(count = size_in_bytes)]
    pub lod_model_file: Vec<u8>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XacUv {
    pub axis_u: f32, // U texture coordinate
    pub axis_v: f32, // V texture coordinate
}

#[derive(Default, Debug, Serialize, Deserialize, BinRead)]
#[br(little)]
pub struct XacSkinInfoPerVertex {
    pub num_influences: u8,
    #[br(count = num_influences)]
    pub influences: Vec<XacSkinInfluence>,
}

#[derive(Default, Debug, Serialize, Deserialize, BinRead)]
#[br(import(num_org_verts:u32))]
#[br(little)]
pub struct XacSkinningInfo {
    pub node_index: u32,
    pub is_for_collision_mesh: u8,
    pub padding: [u8; 3],

    #[br(count = num_org_verts)]
    pub skinning_influence: Vec<XacSkinInfoPerVertex>,
}

#[derive(Default, Debug, Serialize, Deserialize, BinRead)]
#[br(import(num_org_verts:u32))]
#[br(little)]
pub struct XacSkinningInfo2 {
    pub node_index: u32,           // The node number in the actor
    pub num_total_influences: u32, // Total number of influences of all vertices together
    pub is_for_collision_mesh: u8, // Is it for a collision mesh?
    pub padding: [u8; 3],

    #[br(count = num_total_influences)]
    pub skinning_influence: Vec<XacSkinInfluence>,

    #[br(count = num_org_verts)]
    pub skinning_info_table_entry: Vec<XacSkinningInfoTableEntry>,
}

#[derive(Default, Debug, Serialize, Deserialize, BinRead)]
#[br(import(num_org_verts:u32))]
#[br(little)]
pub struct XacSkinningInfo3 {
    pub node_index: u32,           // The node number in the actor
    pub num_local_bones: u32,      // Number of local bones used by the mesh
    pub num_total_influences: u32, // Total number of influences of all vertices together
    pub is_for_collision_mesh: u8, // Is it for a collision mesh?
    pub padding: [u8; 3],

    #[br(count = num_total_influences)]
    pub skinning_influence: Vec<XacSkinInfluence>,

    #[br(count = num_org_verts)]
    pub skinning_info_table_entry: Vec<XacSkinningInfoTableEntry>,
}

#[derive(Default, Debug, Serialize, Deserialize, BinRead)]
#[br(import(num_org_verts:u32))]
#[br(little)]
pub struct XacSkinningInfo4 {
    pub node_index: u32,           // The node number in the actor
    pub lod: u32,                  // Level of detail
    pub num_local_bones: u32,      // Number of local bones used by the mesh
    pub num_total_influences: u32, // Total number of influences of all vertices together
    pub is_for_collision_mesh: u8, // Is it for a collision mesh?
    pub padding: [u8; 3],

    #[br(count = num_total_influences)]
    pub skinning_influence: Vec<XacSkinInfluence>,

    #[br(count = num_org_verts)]
    pub skinning_info_table_entry: Vec<XacSkinningInfoTableEntry>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XacSkinningInfoTableEntry {
    pub start_index: u32,  // Index inside the SkinInfluence array
    pub num_elements: u32, // Number of influences for this item/entry
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XacSkinInfluence {
    pub weight: f32,
    pub node_number: u32,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XacStandardMaterial {
    pub ambient: FileColor,    // Ambient color
    pub diffuse: FileColor,    // Diffuse color
    pub specular: FileColor,   // Specular color
    pub emissive: FileColor,   // Self-illumination color
    pub shine: f32,            // Shine
    pub shine_strength: f32,   // Shine strength
    pub opacity: f32,          // Opacity (1.0 = full opaque, 0.0 = full transparent)
    pub ior: f32,              // Index of refraction
    pub double_sided: u8,      // Double-sided?
    pub wireframe: u8,         // Render in wireframe?
    pub transparency_type: u8, // F=filter / S=subtractive / A=additive / U=unknown
    pub padding: u8,

    #[br(temp)]
    pub material_name_length: u32,
    #[br(count = material_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub material_name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XacStandardMaterial2 {
    pub ambient: FileColor,
    pub diffuse: FileColor,
    pub specular: FileColor,
    pub emissive: FileColor,
    pub shine: f32,
    pub shine_strength: f32,
    pub opacity: f32,
    pub ior: f32,
    pub double_sided: u8,
    pub wireframe: u8,
    pub transparency_type: u8,
    pub num_layers: u8, // Number of material layers

    #[br(temp)]
    pub material_name_length: u32,
    #[br(count = material_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub material_name: String,
    #[br(count = num_layers)]
    pub standard_material_layer2: Vec<XACStandardMaterialLayer2>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XacStandardMaterial3 {
    pub lod: u32, // Level of detail
    pub ambient: FileColor,
    pub diffuse: FileColor,
    pub specular: FileColor,
    pub emissive: FileColor,
    pub shine: f32,
    pub shine_strength: f32,
    pub opacity: f32,
    pub ior: f32,
    pub double_sided: u8,
    pub wireframe: u8,
    pub transparency_type: u8,
    pub num_layers: u8, // Number of material layers

    #[br(temp)]
    pub material_name_length: u32,
    #[br(count = material_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub material_name: String,
    #[br(count = num_layers)]
    pub standard_material_layer2: Vec<XACStandardMaterialLayer2>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACStandardMaterialLayer {
    pub amount: f32,           // the amount, between 0 and 1
    pub u_offset: f32,         // u offset (horizontal texture shift)
    pub v_offset: f32,         // v offset (vertical texture shift)
    pub u_tiling: f32,         // horizontal tiling factor
    pub v_tiling: f32,         // vertical tiling factor
    pub rotation_radians: f32, // texture rotation in radians
    pub material_number: u16,  // the parent material number (0 means first material)
    pub map_type: u8,          // the map type
    pub padding: u8,           // alignment

    #[br(temp)]
    pub texture_name_length: u32,
    #[br(count = texture_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub texture_name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACStandardMaterialLayer2 {
    pub amount: f32,
    pub u_offset: f32,
    pub v_offset: f32,
    pub u_tiling: f32,
    pub v_tiling: f32,
    pub rotation_radians: f32,
    pub material_number: u16,
    pub map_type: u8,
    pub blend_mode: u8, // blend mode for texture layering
    #[br(temp)]
    pub texture_name_length: u32,
    #[br(count = texture_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub texture_name: String,
}

#[derive(Default, Debug, Serialize, Deserialize, BinRead)]
#[br(import(total_verts:u32))]
#[br(little)]
pub struct XACVertexAttributeLayer {
    pub layer_type_id: u32,
    pub attrib_size_in_bytes: u32,
    pub enable_deformations: u8,
    pub is_scale: u8,
    pub padding: [u8; 2],

    #[br(count = attrib_size_in_bytes * total_verts )]
    pub mesh_data: Vec<u8>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[br(little)]
pub struct XACSubMesh {
    pub num_indices: u32,
    pub num_verts: u32,
    pub material_index: u32,
    pub num_bones: u32,

    #[br(count = num_indices)]
    pub indices: Vec<u32>,

    #[br(count = num_bones)]
    pub bones: Vec<u32>,
}

#[derive(Default, Debug, Serialize, Deserialize, BinRead)]
#[br(little)]
pub struct XACMesh {
    pub node_index: u32,
    pub num_org_verts: u32,
    pub total_verts: u32,
    pub total_indices: u32,
    pub num_sub_meshes: u32,
    pub num_layers: u32,
    pub is_collision_mesh: u8,
    pub padding: [u8; 3],

    #[br(args { inner: (total_verts,) })]
    #[br(count = num_layers)]
    pub vertex_attribute_layer: Vec<XACVertexAttributeLayer>,
    #[br(count = num_sub_meshes)]
    pub sub_meshes: Vec<XACSubMesh>,
}

#[derive(Default, Debug, Serialize, Deserialize, BinRead)]
#[br(little)]
pub struct XACMesh2 {
    pub node_index: u32,
    pub lod: u32,
    pub num_org_verts: u32,
    pub total_verts: u32,
    pub total_indices: u32,
    pub num_sub_meshes: u32,
    pub num_layers: u32,
    pub is_collision_mesh: u8,
    pub padding: [u8; 3],

    #[br(args { inner: (total_verts,) })]
    #[br(count = num_layers)]
    pub vertex_attribute_layer: Vec<XACVertexAttributeLayer>,
    #[br(count = num_sub_meshes)]
    pub sub_meshes: Vec<XACSubMesh>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACLimit {
    pub translation_min: FileVector3,
    pub translation_max: FileVector3,
    pub rotation_min: FileVector3,
    pub rotation_max: FileVector3,
    pub scale_min: FileVector3,
    pub scale_max: FileVector3,
    pub limit_flags: [u8; 9], // limit type activation flags
    pub node_number: u32,     // the node number where this info belongs
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACPMorphTarget {
    pub range_min: f32,              // the slider min
    pub range_max: f32,              // the slider max
    pub lod: u32,                    // LOD level
    pub num_mesh_deform_deltas: u32, // number of mesh deform data objects
    pub num_transformations: u32,    // number of transformations
    pub phoneme_sets: u32,           // number of phoneme sets

    #[br(temp)]
    pub name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub name: String,
    #[br(count = num_mesh_deform_deltas)]
    pub morph_target_mesh_deltas: Vec<XACPMorphTargetMeshDeltas>,
    #[br(count = num_transformations)]
    pub morph_target_transform: Vec<XACPMorphTargetTransform>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACPMorphTargets {
    pub num_morph_targets: u32, // number of morph targets
    pub lod: u32,               // LOD level
    #[br(count = num_morph_targets)]
    pub morph_targets: Vec<XACPMorphTargets>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACPMorphTargetMeshDeltas {
    pub node_index: u32,
    pub min_value: f32,    // min range for x, y, z of compressed position vectors
    pub max_value: f32,    // max range for x, y, z of compressed position vectors
    pub num_vertices: u32, // number of deltas
    #[br(count = num_vertices)]
    pub delta_position_values: Vec<File16BitVector3>,
    #[br(count = num_vertices)]
    pub delta_normal_values: Vec<File8BitVector3>,
    #[br(count = num_vertices)]
    pub delta_tangent_values: Vec<File8BitVector3>,
    #[br(count = num_vertices)]
    pub vertex_numbers: Vec<u32>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACPMorphTargetTransform {
    pub node_index: u32,                // node name where transform belongs
    pub rotation: FileQuaternion,       // node rotation
    pub scale_rotation: FileQuaternion, // node delta scale rotation
    pub position: FileVector3,          // node delta position
    pub scale: FileVector3,             // node delta scale
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACFXMaterial {
    pub num_int_params: u32,
    pub num_float_params: u32,
    pub num_color_params: u32,
    pub num_bitmap_params: u32,
    #[br(temp)]
    pub name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub name: String,
    #[br(temp)]
    pub effect_file_length: u32,
    #[br(count = effect_file_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub effect_file: String,
    #[br(temp)]
    pub shader_technique_length: u32,
    #[br(count = shader_technique_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub shader_technique: String,

    #[br(if(num_int_params > 0), count = num_int_params)]
    pub xac_fx_int_parameter: Option<Vec<XACFXIntParameter>>,

    #[br(if(num_float_params > 0), count = num_float_params)]
    pub xac_fx_float_parameter: Option<Vec<XACFXFloatParameter>>,

    #[br(if(num_color_params > 0), count = num_color_params)]
    pub xac_fx_color_parameter: Option<Vec<XACFXColorParameter>>,

    #[br(if(num_bitmap_params > 0), count = num_bitmap_params)]
    pub xac_fx_bitmap_parameter: Option<Vec<XACFXBitmapParameter>>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACFXMaterial2 {
    pub num_int_params: u32,
    pub num_float_params: u32,
    pub num_color_params: u32,
    pub num_bool_params: u32,
    pub num_vector3_params: u32,
    pub num_bitmap_params: u32,
    #[br(temp)]
    pub name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub name: String,
    #[br(temp)]
    pub effect_file_length: u32,
    #[br(count = effect_file_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub effect_file: String,
    #[br(temp)]
    pub shader_technique_length: u32,
    #[br(count = shader_technique_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub shader_technique: String,

    #[br(if(num_int_params > 0), count = num_int_params)]
    pub xac_fx_int_parameter: Option<Vec<XACFXIntParameter>>,

    #[br(if(num_float_params > 0), count = num_float_params)]
    pub xac_fx_float_parameter: Option<Vec<XACFXFloatParameter>>,

    #[br(if(num_color_params > 0), count = num_color_params)]
    pub xac_fx_color_parameter: Option<Vec<XACFXColorParameter>>,

    #[br(if(num_bool_params > 0), count = num_bool_params)]
    pub xac_fx_bool_parameter: Option<Vec<XACFXBoolParameter>>,

    #[br(if(num_vector3_params > 0), count = num_vector3_params)]
    pub xac_fx_vector3_parameter: Option<Vec<XACFXVector3Parameter>>,

    #[br(if(num_bitmap_params > 0), count = num_bitmap_params)]
    pub xac_fx_bitmap_parameter: Option<Vec<XACFXBitmapParameter>>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACFXMaterial3 {
    pub lod: u32, // level of detail
    pub num_int_params: u32,
    pub num_float_params: u32,
    pub num_color_params: u32,
    pub num_bool_params: u32,
    pub num_vector3_params: u32,
    pub num_bitmap_params: u32,
    #[br(temp)]
    pub name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub name: String,
    #[br(temp)]
    pub effect_file_length: u32,
    #[br(count = effect_file_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub effect_file: String,
    #[br(temp)]
    pub shader_technique_length: u32,
    #[br(count = shader_technique_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub shader_technique: String,

    #[br(if(num_int_params > 0), count = num_int_params)]
    pub xac_fx_int_parameter: Option<Vec<XACFXIntParameter>>,

    #[br(if(num_float_params > 0), count = num_float_params)]
    pub xac_fx_float_parameter: Option<Vec<XACFXFloatParameter>>,

    #[br(if(num_color_params > 0), count = num_color_params)]
    pub xac_fx_color_parameter: Option<Vec<XACFXColorParameter>>,

    #[br(if(num_bool_params > 0), count = num_bool_params)]
    pub xac_fx_bool_parameter: Option<Vec<XACFXBoolParameter>>,

    #[br(if(num_vector3_params > 0), count = num_vector3_params)]
    pub xac_fx_vector3_parameter: Option<Vec<XACFXVector3Parameter>>,

    #[br(if(num_bitmap_params > 0), count = num_bitmap_params)]
    pub xac_fx_bitmap_parameter: Option<Vec<XACFXBitmapParameter>>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACFXIntParameter {
    pub value: i32, // Beware, signed integer since negative values are allowed
    #[br(temp)]
    pub name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACFXFloatParameter {
    pub value: f32,
    #[br(temp)]
    pub name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACFXColorParameter {
    pub value: FileColor,
    #[br(temp)]
    pub name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACFXVector3Parameter {
    pub value: FileVector3,
    #[br(temp)]
    pub name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACFXBoolParameter {
    pub value: u8, // 0 = no, 1 = yes
    #[br(temp)]
    pub name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACFXBitmapParameter {
    #[br(temp)]
    pub name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub name: String,

    #[br(temp)]
    pub value_name_length: u32,
    #[br(count = value_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub value_name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACNodeGroup {
    pub num_nodes: u16,
    pub disabled_on_default: u8, // 0 = no, 1 = yes

    #[br(temp)]
    pub name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub name: String,

    #[br(count = num_nodes)]
    pub data: Vec<u16>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACNodes {
    pub num_nodes: u32,
    pub num_root_nodes: u32,

    #[br(count = num_nodes)]
    pub xac_node: Vec<XacNode4>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACMaterialInfo {
    pub num_total_materials: u32, // Total number of materials to follow (including default/extra material)
    pub num_standard_materials: u32, // Number of standard materials in the file
    pub num_fx_materials: u32,    // Number of FX materials in the file
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACMaterialInfo2 {
    pub lod: u32,                    // Level of detail
    pub num_total_materials: u32, // Total number of materials to follow (including default/extra material)
    pub num_standard_materials: u32, // Number of standard materials in the file
    pub num_fx_materials: u32,    // Number of FX materials in the file
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACNodeMotionSources {
    pub num_nodes: u32,

    #[br(count = num_nodes)]
    pub node_indices: Vec<u16>, // List of node indices (optional if mirroring is not set)
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACAttachmentNodes {
    pub num_nodes: u32,

    #[br(count = num_nodes)]
    pub attachment_indices: Vec<u16>, // List of node indices for attachments
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct XACFile {
    pub header: XacHeader,
    pub chunk: Vec<FileChunk>,
    pub chunk_data: Vec<XacChunkData>,
}
