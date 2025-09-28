use crate::shared_formats;
use binrw::{BinRead, binread};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;

/// Mesh type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MeshType {
    /// Static mesh (buildings, props) - can still be transformed but no deformation
    Static = 0,
    /// Dynamic mesh with CPU-processed deformers
    Dynamic = 1,
    /// GPU-skinned mesh processed with skinned shaders
    GpuSkinned = 2,
}

/// Phoneme sets for facial animation (bit flags)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhonemeSet(pub u32);

impl PhonemeSet {
    pub const NONE: Self = Self(0);
    pub const NEUTRAL_POSE: Self = Self(1 << 0);
    pub const M_B_P_X: Self = Self(1 << 1);
    pub const AA_AO_OW: Self = Self(1 << 2);
    pub const IH_AE_AH_EY_AY_H: Self = Self(1 << 3);
    pub const AW: Self = Self(1 << 4);
    pub const N_NG_CH_J_DH_D_G_T_K_Z_ZH_TH_S_SH: Self = Self(1 << 5);
    pub const IY_EH_Y: Self = Self(1 << 6);
    pub const UW_UH_OY: Self = Self(1 << 7);
    pub const F_V: Self = Self(1 << 8);
    pub const L_EL: Self = Self(1 << 9);
    pub const W: Self = Self(1 << 10);
    pub const R_ER: Self = Self(1 << 11);

    /// Checks if a specific phoneme set is enabled
    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    /// Combines phoneme sets
    pub fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

/// Wavelet types for compression
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum WaveletType {
    /// Haar wavelet - fastest option
    Haar = 0,
    /// Daubechies 4 wavelet - better compression, slower than Haar
    Daub4 = 1,
    /// CDF97 wavelet - best compression, slowest (used in JPEG)
    Cdf97 = 2,
}

/// Node clone flags for data sharing control
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeCloneFlags(pub u32);

impl NodeCloneFlags {
    /// Clone node attributes
    pub const ATTRIBUTES: Self = Self(1 << 0);
    /// Clone node stacks
    pub const NODE_STACKS: Self = Self(1 << 1);
    /// Clone collision node stacks
    pub const NODE_COLLISION_STACKS: Self = Self(1 << 2);
    /// Clone mesh data
    pub const MESHES: Self = Self(1 << 3);
    /// Clone collision mesh data
    pub const COLLISION_MESHES: Self = Self(1 << 4);
    /// Default cloning (only attributes)
    pub const DEFAULT: Self = Self::ATTRIBUTES;
    /// Clone everything
    pub const ALL: Self = Self(
        Self::ATTRIBUTES.0
            | Self::NODE_STACKS.0
            | Self::NODE_COLLISION_STACKS.0
            | Self::MESHES.0
            | Self::COLLISION_MESHES.0,
    );
}

/// Node flags for various properties
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeFlags(pub u8);

impl NodeFlags {
    /// Include in bounds calculation
    pub const INCLUDE_IN_BOUNDS_CALC: Self = Self(1 << 0);
    /// Is an attachment node
    pub const ATTACHMENT: Self = Self(1 << 1);
}

/// Coordinate planes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Plane {
    /// XY plane (Z is constant)
    Xy = 0,
    /// XZ plane (Y is constant)
    Xz = 1,
    /// YZ plane (X is constant)
    Yz = 2,
}

/// Dependency types for shared data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DependencyType(pub u32);

impl DependencyType {
    /// Shared meshes
    pub const MESHES: Self = Self(1 << 0);
    /// Shared transforms
    pub const TRANSFORMS: Self = Self(1 << 1);
}

/// Actor clone flags for controlling data duplication
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorCloneFlags(pub u32);

impl ActorCloneFlags {
    /// Clone materials
    pub const MATERIALS: Self = Self(1 << 0);
    /// Clone node attributes
    pub const NODE_ATTRIBUTES: Self = Self(1 << 1);
    /// Clone controllers
    pub const CONTROLLERS: Self = Self(1 << 2);
    /// Clone meshes
    pub const MESHES: Self = Self(1 << 3);
    /// Clone collision meshes
    pub const COLLISION_MESHES: Self = Self(1 << 4);
    /// Default cloning
    pub const DEFAULT: Self = Self(Self::NODE_ATTRIBUTES.0 | Self::CONTROLLERS.0);
    /// Clone everything
    pub const ALL: Self = Self(
        Self::MATERIALS.0
            | Self::NODE_ATTRIBUTES.0
            | Self::CONTROLLERS.0
            | Self::MESHES.0
            | Self::COLLISION_MESHES.0,
    );
}

/// Motion-based actor repositioning mask
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RepositioningMask(pub u32);

impl RepositioningMask {
    /// Update position based on repositioning node
    pub const POSITION: Self = Self(1 << 0);
    /// Update rotation based on repositioning node
    pub const ROTATION: Self = Self(1 << 1);
    /// Update scale based on repositioning node (currently unsupported)
    pub const SCALE: Self = Self(1 << 2);
}

/// Transform limit types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LimitType(pub u32);

impl LimitType {
    pub const TRANSLATION_X: Self = Self(1 << 0);
    pub const TRANSLATION_Y: Self = Self(1 << 1);
    pub const TRANSLATION_Z: Self = Self(1 << 2);
    pub const ROTATION_X: Self = Self(1 << 3);
    pub const ROTATION_Y: Self = Self(1 << 4);
    pub const ROTATION_Z: Self = Self(1 << 5);
    pub const SCALE_X: Self = Self(1 << 6);
    pub const SCALE_Y: Self = Self(1 << 7);
    pub const SCALE_Z: Self = Self(1 << 8);
}

/// Vertex attribute types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum VertexAttribute {
    /// Vertex positions (always exist)
    Positions = 0,
    /// Vertex normals (always exist)
    Normals = 1,
    /// Vertex tangents (Vector4)
    Tangents = 2,
    /// UV coordinates
    UvCoords = 3,
    /// 32-bit vertex colors
    Colors32 = 4,
    /// Original vertex numbers (always exist)
    OriginalVertexNumbers = 5,
    /// 128-bit vertex colors
    Colors128 = 6,
    /// Vertex bitangents/binormals
    Bitangents = 7,
}

/// XAC-specific chunk identifiers
pub mod xac_chunk_ids {
    pub const NODE: u32 = 0;
    pub const MESH: u32 = 1;
    pub const SKINNING_INFO: u32 = 2;
    pub const STD_MATERIAL: u32 = 3;
    pub const STD_MATERIAL_LAYER: u32 = 4;
    pub const FX_MATERIAL: u32 = 5;
    pub const LIMIT: u32 = 6;
    pub const INFO: u32 = 7;
    pub const MESH_LOD_LEVELS: u32 = 8;
    pub const STD_PROG_MORPH_TARGET: u32 = 9;
    pub const NODE_GROUPS: u32 = 10;
    pub const NODES: u32 = 11;
    pub const STD_PMORPH_TARGETS: u32 = 12;
    pub const MATERIAL_INFO: u32 = 13;
    pub const NODE_MOTION_SOURCES: u32 = 14;
    pub const ATTACHMENT_NODES: u32 = 15;
    pub const XACFORCE32BIT: u32 = 0xFFFFFFFF;
}

/// Material layer map types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LayerId {
    Unknown = 0,
    Ambient = 1,
    Diffuse = 2,
    Specular = 3,
    Opacity = 4,
    Bump = 5,
    SelfIllum = 6,
    Shine = 7,
    ShineStrength = 8,
    FilterColor = 9,
    Reflect = 10,
    Refract = 11,
    Environment = 12,
    Displacement = 13,
}

#[derive(Debug, Serialize, Deserialize)]
enum XacChunkData {
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

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct FileChunk {
    chunk_id: u32,      // The chunk ID
    size_in_bytes: u32, // The size in bytes of this chunk (excluding this struct)
    version: u32,       // The version of the chunk
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)] // Color [0..1] range
struct FileColor {
    color_red: f32,   // Red
    color_green: f32, // Green
    color_blue: f32,  // Blue
    color_alpha: f32, // Alpha
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)] // A 3D vector
struct FileVector3 {
    axis_x: f32, // x+ = to the right
    axis_y: f32, // y+ = up
    axis_z: f32, // z+ = forwards (into the depth)
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)] // A compressed 3D vector
struct File16BitVector3 {
    axis_x: u16, // x+ = to the right
    axis_y: u16, // y+ = up
    axis_z: u16, // z+ = forwards (into the depth)
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)] // A compressed 3D vector
struct File8BitVector3 {
    axis_x: u8, // x+ = to the right
    axis_y: u8, // y+ = up
    axis_z: u8, // z+ = forwards (into the depth)
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)] // A quaternion
struct FileQuaternion {
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
    axis_w: f32,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)] // The 16-bit component quaternion
struct File16BitQuaternion {
    axis_x: i16,
    axis_y: i16,
    axis_z: i16,
    axis_w: i16,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XacHeader {
    fourcc: u32,     // Must be "XAC "
    hi_version: u8,  // High version (e.g., 2 in v2.34)
    lo_version: u8,  // Low version (e.g., 34 in v2.34)
    endian_type: u8, // Endianness: 0 = little, 1 = big
    mul_order: u8,   // See enum MULORDER_...
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XacInfo {
    repositioning_mask: u32,
    repositioning_node_index: u32,
    exporter_high_version: u8,
    exporter_low_version: u8,
    padding: u16,

    #[br(temp)]
    source_app_length: u32,
    #[br(count = source_app_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    source_app: String,

    #[br(temp)]
    original_filename_length: u32,
    #[br(count = original_filename_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    original_filename: String,

    #[br(temp)]
    compilation_date_length: u32,
    #[br(count = compilation_date_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    compilation_date: String,

    #[br(temp)]
    actor_name_length: u32,
    #[br(count = actor_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    actor_name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XacInfo2 {
    repositioning_mask: u32,
    repositioning_node_index: u32,
    exporter_high_version: u8,
    exporter_low_version: u8,
    retarget_root_offset: f32,
    padding: u16,

    #[br(temp)]
    source_app_length: u32,
    #[br(count = source_app_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    source_app: String,

    #[br(temp)]
    original_filename_length: u32,
    #[br(count = original_filename_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    original_filename: String,

    #[br(temp)]
    compilation_date_length: u32,
    #[br(count = compilation_date_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    compilation_date: String,

    #[br(temp)]
    actor_name_length: u32,
    #[br(count = actor_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    actor_name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XacInfo3 {
    trajectory_node_index: u32,
    motion_extraction_node_index: u32,
    motion_extraction_mask: u32,
    exporter_high_version: u8,
    exporter_low_version: u8,
    retarget_root_offset: f32,
    padding: u16,

    #[br(temp)]
    source_app_length: u32,
    #[br(count = source_app_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    source_app: String,

    #[br(temp)]
    original_filename_length: u32,
    #[br(count = original_filename_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    original_filename: String,

    #[br(temp)]
    compilation_date_length: u32,
    #[br(count = compilation_date_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    compilation_date: String,

    #[br(temp)]
    actor_name_length: u32,
    #[br(count = actor_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    actor_name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XacInfo4 {
    num_lods: u32,
    trajectory_node_index: u32,
    motion_extraction_node_index: u32,
    exporter_high_version: u8,
    exporter_low_version: u8,
    retarget_root_offset: f32,
    padding: u16,

    #[br(temp)]
    source_app_length: u32,
    #[br(count = source_app_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    source_app: String,

    #[br(temp)]
    original_filename_length: u32,
    #[br(count = original_filename_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    original_filename: String,

    #[br(temp)]
    compilation_date_length: u32,
    #[br(count = compilation_date_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    compilation_date: String,

    #[br(temp)]
    actor_name_length: u32,
    #[br(count = actor_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    actor_name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XacNode {
    local_quat: FileQuaternion,
    scale_rot: FileQuaternion,
    local_pos: FileVector3,
    local_scale: FileVector3,
    shear: FileVector3,
    skeletal_lods: u32,
    parent_index: u32,

    #[br(temp)]
    node_name_length: u32,
    #[br(count = node_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    node_name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XacNode2 {
    local_quat: FileQuaternion,
    scale_rot: FileQuaternion,
    local_pos: FileVector3,
    local_scale: FileVector3,
    shear: FileVector3,
    skeletal_lods: u32,
    parent_index: u32,
    node_flags: u8,
    padding: [u8; 3],

    #[br(temp)]
    node_name_length: u32,
    #[br(count = node_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    node_name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XacNode3 {
    local_quat: FileQuaternion,
    scale_rot: FileQuaternion,
    local_pos: FileVector3,
    local_scale: FileVector3,
    shear: FileVector3,
    skeletal_lods: u32,
    parent_index: u32,
    node_flags: u8,
    obb: [f32; 16], // Oriented Bounding Box (OBB)
    padding: [u8; 3],

    #[br(temp)]
    node_name_length: u32,
    #[br(count = node_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    node_name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XacNode4 {
    local_quat: FileQuaternion,
    scale_rot: FileQuaternion,
    local_pos: FileVector3,
    local_scale: FileVector3,
    shear: FileVector3,
    skeletal_lods: u32,
    motion_lods: u32,
    parent_index: u32,
    num_children: u32,
    node_flags: u8,
    obb: [f32; 16],         // Oriented Bounding Box (OBB)
    importance_factor: f32, // Used for automatic motion LOD
    padding: [u8; 3],

    #[br(temp)]
    node_name_length: u32,
    #[br(count = node_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    node_name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XACMeshLodLevel {
    lod_level: u32,
    size_in_bytes: u32,

    #[br(count = size_in_bytes)]
    lod_model_file: Vec<u8>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XacUv {
    axis_u: f32, // U texture coordinate
    axis_v: f32, // V texture coordinate
}

#[derive(Default, Debug, Serialize, Deserialize, BinRead)]
#[br(little)]
struct XacSkinInfoPerVertex {
    num_influences: u8,
    #[br(count = num_influences)]
    influences: Vec<XacSkinInfluence>,
}

#[derive(Default, Debug, Serialize, Deserialize, BinRead)]
#[br(import(num_org_verts:u32))]
#[br(little)]
struct XacSkinningInfo {
    node_index: u32,
    is_for_collision_mesh: u8,
    padding: [u8; 3],

    #[br(count = num_org_verts)]
    skinning_influence: Vec<XacSkinInfoPerVertex>,
}

#[derive(Default, Debug, Serialize, Deserialize, BinRead)]
#[br(import(num_org_verts:u32))]
#[br(little)]
struct XacSkinningInfo2 {
    node_index: u32,           // The node number in the actor
    num_total_influences: u32, // Total number of influences of all vertices together
    is_for_collision_mesh: u8, // Is it for a collision mesh?
    padding: [u8; 3],

    #[br(count = num_total_influences)]
    skinning_influence: Vec<XacSkinInfluence>,

    #[br(count = num_org_verts)]
    skinning_info_table_entry: Vec<XacSkinningInfoTableEntry>,
}

#[derive(Default, Debug, Serialize, Deserialize, BinRead)]
#[br(import(num_org_verts:u32))]
#[br(little)]
struct XacSkinningInfo3 {
    node_index: u32,           // The node number in the actor
    num_local_bones: u32,      // Number of local bones used by the mesh
    num_total_influences: u32, // Total number of influences of all vertices together
    is_for_collision_mesh: u8, // Is it for a collision mesh?
    padding: [u8; 3],

    #[br(count = num_total_influences)]
    skinning_influence: Vec<XacSkinInfluence>,

    #[br(count = num_org_verts)]
    skinning_info_table_entry: Vec<XacSkinningInfoTableEntry>,
}

#[derive(Default, Debug, Serialize, Deserialize, BinRead)]
#[br(import(num_org_verts:u32))]
#[br(little)]
struct XacSkinningInfo4 {
    node_index: u32,           // The node number in the actor
    lod: u32,                  // Level of detail
    num_local_bones: u32,      // Number of local bones used by the mesh
    num_total_influences: u32, // Total number of influences of all vertices together
    is_for_collision_mesh: u8, // Is it for a collision mesh?
    padding: [u8; 3],

    #[br(count = num_total_influences)]
    skinning_influence: Vec<XacSkinInfluence>,

    #[br(count = num_org_verts)]
    skinning_info_table_entry: Vec<XacSkinningInfoTableEntry>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XacSkinningInfoTableEntry {
    start_index: u32,  // Index inside the SkinInfluence array
    num_elements: u32, // Number of influences for this item/entry
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XacSkinInfluence {
    weight: f32,
    node_number: u32,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XacStandardMaterial {
    ambient: FileColor,    // Ambient color
    diffuse: FileColor,    // Diffuse color
    specular: FileColor,   // Specular color
    emissive: FileColor,   // Self-illumination color
    shine: f32,            // Shine
    shine_strength: f32,   // Shine strength
    opacity: f32,          // Opacity (1.0 = full opaque, 0.0 = full transparent)
    ior: f32,              // Index of refraction
    double_sided: u8,      // Double-sided?
    wireframe: u8,         // Render in wireframe?
    transparency_type: u8, // F=filter / S=subtractive / A=additive / U=unknown
    padding: u8,

    #[br(temp)]
    material_name_length: u32,
    #[br(count = material_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    material_name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XacStandardMaterial2 {
    ambient: FileColor,
    diffuse: FileColor,
    specular: FileColor,
    emissive: FileColor,
    shine: f32,
    shine_strength: f32,
    opacity: f32,
    ior: f32,
    double_sided: u8,
    wireframe: u8,
    transparency_type: u8,
    num_layers: u8, // Number of material layers

    #[br(temp)]
    material_name_length: u32,
    #[br(count = material_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    material_name: String,
    #[br(count = num_layers)]
    standard_material_layer2: Vec<XACStandardMaterialLayer2>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XacStandardMaterial3 {
    lod: u32, // Level of detail
    ambient: FileColor,
    diffuse: FileColor,
    specular: FileColor,
    emissive: FileColor,
    shine: f32,
    shine_strength: f32,
    opacity: f32,
    ior: f32,
    double_sided: u8,
    wireframe: u8,
    transparency_type: u8,
    num_layers: u8, // Number of material layers

    #[br(temp)]
    material_name_length: u32,
    #[br(count = material_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    material_name: String,
    #[br(count = num_layers)]
    standard_material_layer2: Vec<XACStandardMaterialLayer2>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XACStandardMaterialLayer {
    amount: f32,           // the amount, between 0 and 1
    u_offset: f32,         // u offset (horizontal texture shift)
    v_offset: f32,         // v offset (vertical texture shift)
    u_tiling: f32,         // horizontal tiling factor
    v_tiling: f32,         // vertical tiling factor
    rotation_radians: f32, // texture rotation in radians
    material_number: u16,  // the parent material number (0 means first material)
    map_type: u8,          // the map type
    padding: u8,           // alignment

    #[br(temp)]
    texture_name_length: u32,
    #[br(count = texture_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    texture_name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XACStandardMaterialLayer2 {
    amount: f32,
    u_offset: f32,
    v_offset: f32,
    u_tiling: f32,
    v_tiling: f32,
    rotation_radians: f32,
    material_number: u16,
    map_type: u8,
    blend_mode: u8, // blend mode for texture layering
    #[br(temp)]
    texture_name_length: u32,
    #[br(count = texture_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    texture_name: String,
}

#[derive(Default, Debug, Serialize, Deserialize, BinRead)]
#[br(import(total_verts:u32))]
#[br(little)]
struct XACVertexAttributeLayer {
    layer_type_id: u32,
    attrib_size_in_bytes: u32,
    enable_deformations: u8,
    is_scale: u8,
    padding: [u8; 2],

    #[br(count = attrib_size_in_bytes * total_verts )]
    mesh_data: Vec<u8>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[br(little)]
struct XACSubMesh {
    num_indices: u32,
    num_verts: u32,
    material_index: u32,
    num_bones: u32,

    #[br(count = num_indices)]
    indices: Vec<u32>,

    #[br(count = num_bones)]
    bones: Vec<u32>,
}

#[derive(Default, Debug, Serialize, Deserialize, BinRead)]
#[br(little)]
struct XACMesh {
    node_index: u32,
    num_org_verts: u32,
    total_verts: u32,
    total_indices: u32,
    num_sub_meshes: u32,
    num_layers: u32,
    is_collision_mesh: u8,
    padding: [u8; 3],

    #[br(args { inner: (total_verts,) })]
    #[br(count = num_layers)]
    vertex_attribute_layer: Vec<XACVertexAttributeLayer>,
    #[br(count = num_sub_meshes)]
    sub_meshes: Vec<XACSubMesh>,
}

#[derive(Default, Debug, Serialize, Deserialize, BinRead)]
#[br(little)]
struct XACMesh2 {
    node_index: u32,
    lod: u32,
    num_org_verts: u32,
    total_verts: u32,
    total_indices: u32,
    num_sub_meshes: u32,
    num_layers: u32,
    is_collision_mesh: u8,
    padding: [u8; 3],

    #[br(args { inner: (total_verts,) })]
    #[br(count = num_layers)]
    vertex_attribute_layer: Vec<XACVertexAttributeLayer>,
    #[br(count = num_sub_meshes)]
    sub_meshes: Vec<XACSubMesh>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XACLimit {
    translation_min: FileVector3,
    translation_max: FileVector3,
    rotation_min: FileVector3,
    rotation_max: FileVector3,
    scale_min: FileVector3,
    scale_max: FileVector3,
    limit_flags: [u8; 9], // limit type activation flags
    node_number: u32,     // the node number where this info belongs
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XACPMorphTarget {
    range_min: f32,              // the slider min
    range_max: f32,              // the slider max
    lod: u32,                    // LOD level
    num_mesh_deform_deltas: u32, // number of mesh deform data objects
    num_transformations: u32,    // number of transformations
    phoneme_sets: u32,           // number of phoneme sets

    #[br(temp)]
    name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    name: String,
    #[br(count = num_mesh_deform_deltas)]
    morph_target_mesh_deltas: Vec<XACPMorphTargetMeshDeltas>,
    #[br(count = num_transformations)]
    morph_target_transform: Vec<XACPMorphTargetTransform>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XACPMorphTargets {
    num_morph_targets: u32, // number of morph targets
    lod: u32,               // LOD level
    #[br(count = num_morph_targets)]
    morph_targets: Vec<XACPMorphTargets>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XACPMorphTargetMeshDeltas {
    node_index: u32,
    min_value: f32,    // min range for x, y, z of compressed position vectors
    max_value: f32,    // max range for x, y, z of compressed position vectors
    num_vertices: u32, // number of deltas
    #[br(count = num_vertices)]
    delta_position_values: Vec<File16BitVector3>,
    #[br(count = num_vertices)]
    delta_normal_values: Vec<File8BitVector3>,
    #[br(count = num_vertices)]
    delta_tangent_values: Vec<File8BitVector3>,
    #[br(count = num_vertices)]
    vertex_numbers: Vec<u32>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XACPMorphTargetTransform {
    node_index: u32,                // node name where transform belongs
    rotation: FileQuaternion,       // node rotation
    scale_rotation: FileQuaternion, // node delta scale rotation
    position: FileVector3,          // node delta position
    scale: FileVector3,             // node delta scale
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XACFXMaterial {
    num_int_params: u32,
    num_float_params: u32,
    num_color_params: u32,
    num_bitmap_params: u32,
    #[br(temp)]
    name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    name: String,
    #[br(temp)]
    effect_file_length: u32,
    #[br(count = effect_file_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    effect_file: String,
    #[br(temp)]
    shader_technique_length: u32,
    #[br(count = shader_technique_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    shader_technique: String,

    #[br(if(num_int_params > 0), count = num_int_params)]
    xac_fx_int_parameter: Option<Vec<XACFXIntParameter>>,

    #[br(if(num_float_params > 0), count = num_float_params)]
    xac_fx_float_parameter: Option<Vec<XACFXFloatParameter>>,

    #[br(if(num_color_params > 0), count = num_color_params)]
    xac_fx_color_parameter: Option<Vec<XACFXColorParameter>>,

    #[br(if(num_bitmap_params > 0), count = num_bitmap_params)]
    xac_fx_bitmap_parameter: Option<Vec<XACFXBitmapParameter>>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XACFXMaterial2 {
    num_int_params: u32,
    num_float_params: u32,
    num_color_params: u32,
    num_bool_params: u32,
    num_vector3_params: u32,
    num_bitmap_params: u32,
    #[br(temp)]
    name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    name: String,
    #[br(temp)]
    effect_file_length: u32,
    #[br(count = effect_file_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    effect_file: String,
    #[br(temp)]
    shader_technique_length: u32,
    #[br(count = shader_technique_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    shader_technique: String,

    #[br(if(num_int_params > 0), count = num_int_params)]
    xac_fx_int_parameter: Option<Vec<XACFXIntParameter>>,

    #[br(if(num_float_params > 0), count = num_float_params)]
    xac_fx_float_parameter: Option<Vec<XACFXFloatParameter>>,

    #[br(if(num_color_params > 0), count = num_color_params)]
    xac_fx_color_parameter: Option<Vec<XACFXColorParameter>>,

    #[br(if(num_bool_params > 0), count = num_bool_params)]
    xac_fx_bool_parameter: Option<Vec<XACFXBoolParameter>>,

    #[br(if(num_vector3_params > 0), count = num_vector3_params)]
    xac_fx_vector3_parameter: Option<Vec<XACFXVector3Parameter>>,

    #[br(if(num_bitmap_params > 0), count = num_bitmap_params)]
    xac_fx_bitmap_parameter: Option<Vec<XACFXBitmapParameter>>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XACFXMaterial3 {
    lod: u32, // level of detail
    num_int_params: u32,
    num_float_params: u32,
    num_color_params: u32,
    num_bool_params: u32,
    num_vector3_params: u32,
    num_bitmap_params: u32,
    #[br(temp)]
    name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    name: String,
    #[br(temp)]
    effect_file_length: u32,
    #[br(count = effect_file_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    effect_file: String,
    #[br(temp)]
    shader_technique_length: u32,
    #[br(count = shader_technique_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    shader_technique: String,

    #[br(if(num_int_params > 0), count = num_int_params)]
    xac_fx_int_parameter: Option<Vec<XACFXIntParameter>>,

    #[br(if(num_float_params > 0), count = num_float_params)]
    xac_fx_float_parameter: Option<Vec<XACFXFloatParameter>>,

    #[br(if(num_color_params > 0), count = num_color_params)]
    xac_fx_color_parameter: Option<Vec<XACFXColorParameter>>,

    #[br(if(num_bool_params > 0), count = num_bool_params)]
    xac_fx_bool_parameter: Option<Vec<XACFXBoolParameter>>,

    #[br(if(num_vector3_params > 0), count = num_vector3_params)]
    xac_fx_vector3_parameter: Option<Vec<XACFXVector3Parameter>>,

    #[br(if(num_bitmap_params > 0), count = num_bitmap_params)]
    xac_fx_bitmap_parameter: Option<Vec<XACFXBitmapParameter>>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XACFXIntParameter {
    value: i32, // Beware, signed integer since negative values are allowed
    #[br(temp)]
    name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XACFXFloatParameter {
    value: f32,
    #[br(temp)]
    name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XACFXColorParameter {
    value: FileColor,
    #[br(temp)]
    name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XACFXVector3Parameter {
    value: FileVector3,
    #[br(temp)]
    name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XACFXBoolParameter {
    value: u8, // 0 = no, 1 = yes
    #[br(temp)]
    name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XACFXBitmapParameter {
    #[br(temp)]
    name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    name: String,

    #[br(temp)]
    value_name_length: u32,
    #[br(count = value_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    value_name: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XACNodeGroup {
    num_nodes: u16,
    disabled_on_default: u8, // 0 = no, 1 = yes

    #[br(temp)]
    name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    name: String,

    #[br(count = num_nodes)]
    data: Vec<u16>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XACNodes {
    num_nodes: u32,
    num_root_nodes: u32,

    #[br(count = num_nodes)]
    xac_node: Vec<XacNode4>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XACMaterialInfo {
    num_total_materials: u32, // Total number of materials to follow (including default/extra material)
    num_standard_materials: u32, // Number of standard materials in the file
    num_fx_materials: u32,    // Number of FX materials in the file
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XACMaterialInfo2 {
    lod: u32,                    // Level of detail
    num_total_materials: u32, // Total number of materials to follow (including default/extra material)
    num_standard_materials: u32, // Number of standard materials in the file
    num_fx_materials: u32,    // Number of FX materials in the file
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XACNodeMotionSources {
    num_nodes: u32,

    #[br(count = num_nodes)]
    node_indices: Vec<u16>, // List of node indices (optional if mirroring is not set)
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
struct XACAttachmentNodes {
    num_nodes: u32,

    #[br(count = num_nodes)]
    attachment_indices: Vec<u16>, // List of node indices for attachments
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct XACFile {
    header: XacHeader,
    chunk: Vec<FileChunk>,
    chunk_data: Vec<XacChunkData>,
}
