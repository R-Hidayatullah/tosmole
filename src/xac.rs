use binrw::{BinRead, BinReaderExt, BinResult, binread};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
enum SkeletalMotionType {
    SkelmotiontypeNormal = 0, // A regular keyframe and keytrack based skeletal motion.
    SkelmotiontypeWavelet = 1, // A wavelet compressed skeletal motion.
}

#[derive(Debug, Serialize, Deserialize)]
enum FileType {
    FiletypeUnknown = 0,           // An unknown file, or something went wrong.
    FiletypeActor,                 // An actor file (.xac).
    FiletypeSkeletalmotion,        // A skeletal motion file (.xsm).
    FiletypeWaveletskeletalmotion, // A wavelet compressed skeletal motion (.xsm).
    FiletypePmorphmotion,          // A progressive morph motion file (.xpm).
}

// shared chunk ID's
#[derive(Debug, Serialize, Deserialize)]
pub enum SharedChunk {
    SharedChunkMotioneventtable = 50,
    SharedChunkTimestamp = 51,
}

// matrix multiplication order
#[derive(Debug, Serialize, Deserialize)]
pub enum MatrixMulOrder {
    MulorderScaleRotTrans = 0,
    MulorderRotScaleTrans = 1,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MeshType {
    MeshtypeStatic = 0, //< Static mesh, like a cube or building (can still be position/scale/rotation animated though).
    MeshtypeDynamic = 1, //< Has mesh deformers that have to be processed on the CPU.
    MeshtypeGpuskinned = 2, //< Just a skinning mesh deformer that gets processed on the GPU with skinned shader.
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PhonemeSet {
    PhonemesetNone = 0,
    PhonemesetNeutralPose = 1 << 0,
    PhonemesetMBPX = 1 << 1,
    PhonemesetAaAoOw = 1 << 2,
    PhonemesetIhAeAhEyAyH = 1 << 3,
    PhonemesetAw = 1 << 4,
    PhonemesetNNgChJDhDGTKZZhThSSh = 1 << 5,
    PhonemesetIyEhY = 1 << 6,
    PhonemesetUwUhOy = 1 << 7,
    PhonemesetFV = 1 << 8,
    PhonemesetLEl = 1 << 9,
    PhonemesetW = 1 << 10,
    PhonemesetREr = 1 << 11,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WaveletType {
    WaveletHaar = 0, // The Haar wavelet, which is most likely what you want to use. It is the fastest also.
    WaveletDaub4 = 1, // Daubechies 4 wavelet, can result in bit better compression ratios, but slower than Haar.
    WaveletCdf97 = 2, // The CDF97 wavelet, used in JPG as well. This is the slowest, but often results in the best compression ratios.
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NodeFlags {
    FlagIncludeinboundscalc = 1 << 0, // Specifies whether we have to include this node in the bounds calculation or not (true on default).
    FlagAttachment = 1 << 1, // Indicates if this node is an attachment node or not (false on default).
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Plane {
    PlaneXy = 0, // The XY plane, so where Z is constant.
    PlaneXz = 1, // The XZ plane, so where Y is constant.
    PlaneYz = 2, // The YZ plane, so where X is constant.
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DependencyType {
    DependencyMeshes = 1 << 0,     // Shared meshes.
    DependencyTransforms = 1 << 1, // Shared transforms.
}

/// The motion based actor repositioning mask
#[derive(Debug, Serialize, Deserialize)]
pub enum RepositioningMask {
    RepositionPosition = 1 << 0, // Update the actor position based on the repositioning node.
    RepositionRotation = 1 << 1, // Update the actor rotation based on the repositioning node.
    RepositionScale = 1 << 2, // [CURRENTLY UNSUPPORTED] Update the actor scale based on the repositioning node.
}

/// The order of multiplication when composing a transformation matrix from a translation, rotation and scale.
#[derive(Debug, Serialize, Deserialize)]
pub enum MultiplicationOrder {
    ScaleRotationTranslation = 0, // LocalTM = scale * rotation * translation (Maya style).
    RotationScaleTranslation = 1, // LocalTM = rotation * scale * translation (3DSMax style) [default].
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LimitType {
    TranslationX = 1 << 0, // Position limit on the x axis.
    TranslationY = 1 << 1, // Position limit on the y axis.
    TranslationZ = 1 << 2, // Position limit on the z axis.
    RotationX = 1 << 3,    // Rotation limit on the x axis.
    RotationY = 1 << 4,    // Rotation limit on the y axis.
    RotationZ = 1 << 5,    // Rotation limit on the z axis.
    ScaleX = 1 << 6,       // Scale limit on the x axis.
    ScaleY = 1 << 7,       // Scale limit on the y axis.
    ScaleZ = 1 << 8,       // Scale limit on the z axis.
}

#[derive(Debug, Serialize, Deserialize)]
pub enum XACAttribute {
    AttribPositions = 0, // Vertex positions. Typecast to MCore::Vector3. Positions are always exist.
    AttribNormals = 1,   // Vertex normals. Typecast to MCore::Vector3. Normals are always exist.
    AttribTangents = 2,  // Vertex tangents. Typecast to <b> MCore::Vector4 </b>.
    AttribUvcoords = 3,  // Vertex uv coordinates. Typecast to MCore::Vector2.
    AttribColors32 = 4,  // Vertex colors in 32-bits. Typecast to uint32.
    AttribOrgvtxnumbers = 5, // Original vertex numbers. Typecast to uint32. Original vertex numbers always exist.
    AttribColors128 = 6,     // Vertex colors in 128-bits. Typecast to MCore::RGBAColor.
    AttribBitangents = 7, // Vertex bitangents (aka binormal). Typecast to MCore::Vector3. When tangents exists bitangents may still not exist!
}

impl XACAttribute {
    pub fn from_u32(val: u32) -> Option<Self> {
        match val {
            0 => Some(XACAttribute::AttribPositions),
            1 => Some(XACAttribute::AttribNormals),
            2 => Some(XACAttribute::AttribTangents),
            3 => Some(XACAttribute::AttribUvcoords),
            4 => Some(XACAttribute::AttribColors32),
            5 => Some(XACAttribute::AttribOrgvtxnumbers),
            6 => Some(XACAttribute::AttribColors128),
            7 => Some(XACAttribute::AttribBitangents),
            _ => None,
        }
    }
}

// collection of XAC chunk IDs
#[derive(Debug, Serialize, Deserialize)]
pub enum XACChunk {
    XACChunkNode = 0,
    XACChunkMesh = 1,
    XACChunkSkinninginfo = 2,
    XACChunkStdmaterial = 3,
    XACChunkStdmateriallayer = 4,
    XACChunkFxmaterial = 5,
    XACChunkLimit = 6,
    XACChunkInfo = 7,
    XACChunkMeshlodlevels = 8,
    XACChunkStdprogmorphtarget = 9,
    XACChunkNodegroups = 10,
    XACChunkNodes = 11,             // XAC_Nodes
    XACChunkStdpmorphtargets = 12,  // XAC_PMorphTargets
    XACChunkMaterialinfo = 13,      // XAC_MaterialInfo
    XACChunkNodemotionsources = 14, // XAC_NodeMotionSources
    XACChunkAttachmentnodes = 15,   // XAC_AttachmentNodes
    XACForce32bit = 0xFFFFFFFF,
}

// material layer map types
#[derive(Debug, Serialize, Deserialize)]
pub enum XACMaterialLayer {
    XACLayeridUnknown = 0,       // unknown layer
    XACLayeridAmbient = 1,       // ambient layer
    XACLayeridDiffuse = 2,       // a diffuse layer
    XACLayeridSpecular = 3,      // specular layer
    XACLayeridOpacity = 4,       // opacity layer
    XACLayeridBump = 5,          // bump layer
    XACLayeridSelfillum = 6,     // self illumination layer
    XACLayeridShine = 7,         // shininess (for specular)
    XACLayeridShinestrength = 8, // shine strength (for specular)
    XACLayeridFiltercolor = 9,   // filter color layer
    XACLayeridReflect = 10,      // reflection layer
    XACLayeridRefract = 11,      // refraction layer
    XACLayeridEnvironment = 12,  // environment map layer
    XACLayeridDisplacement = 13, // displacement map layer
    XACLayeridForce8bit = 0xFF,  // don't use more than 8 bit values
}

#[derive(Debug, Serialize, Deserialize)]
pub enum XACChunkData {
    XACInfo(XACInfo),
    XACInfo2(XACInfo2),
    XACInfo3(XACInfo3),
    XACInfo4(XACInfo4),

    XACNode(XACNode),
    XACNode2(XACNode2),
    XACNode3(XACNode3),
    XACNode4(XACNode4),

    XACSkinningInfo(XACSkinningInfo),
    XACSkinningInfo2(XACSkinningInfo2),
    XACSkinningInfo3(XACSkinningInfo3),
    XACSkinningInfo4(XACSkinningInfo4),

    XACStandardMaterial(XACStandardMaterial),
    XACStandardMaterial2(XACStandardMaterial2),
    XACStandardMaterial3(XACStandardMaterial3),

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
pub struct XACHeader {
    pub fourcc: u32,     // Must be "XAC "
    pub hi_version: u8,  // High version (e.g., 2 in v2.34)
    pub lo_version: u8,  // Low version (e.g., 34 in v2.34)
    pub endian_type: u8, // Endianness: 0 = little, 1 = big
    pub mul_order: u8,   // See enum MULORDER_...
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACInfo {
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
pub struct XACInfo2 {
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
pub struct XACInfo3 {
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
pub struct XACInfo4 {
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
pub struct XACNode {
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
pub struct XACNode2 {
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
pub struct XACNode3 {
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
pub struct XACNode4 {
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
pub struct XACUv {
    pub axis_u: f32, // U texture coordinate
    pub axis_v: f32, // V texture coordinate
}

#[derive(Default, Debug, Serialize, Deserialize, BinRead)]
#[br(little)]
pub struct XACSkinInfoPerVertex {
    pub num_influences: u8,
    #[br(count = num_influences)]
    pub influences: Vec<XACSkinInfluence>,
}

#[derive(Default, Debug, Serialize, Deserialize, BinRead)]
#[br(import(num_org_verts:u32))]
#[br(little)]
pub struct XACSkinningInfo {
    pub node_index: u32,
    pub is_for_collision_mesh: u8,
    pub padding: [u8; 3],

    #[br(count = num_org_verts)]
    pub skinning_influence: Vec<XACSkinInfoPerVertex>,
}

#[derive(Default, Debug, Serialize, Deserialize, BinRead)]
#[br(import(num_org_verts:u32))]
#[br(little)]
pub struct XACSkinningInfo2 {
    pub node_index: u32,           // The node number in the actor
    pub num_total_influences: u32, // Total number of influences of all vertices together
    pub is_for_collision_mesh: u8, // Is it for a collision mesh?
    pub padding: [u8; 3],

    #[br(count = num_total_influences)]
    pub skinning_influence: Vec<XACSkinInfluence>,

    #[br(count = num_org_verts)]
    pub skinning_info_table_entry: Vec<XACSkinningInfoTableEntry>,
}

#[derive(Default, Debug, Serialize, Deserialize, BinRead)]
#[br(import(num_org_verts:u32))]
#[br(little)]
pub struct XACSkinningInfo3 {
    pub node_index: u32,           // The node number in the actor
    pub num_local_bones: u32,      // Number of local bones used by the mesh
    pub num_total_influences: u32, // Total number of influences of all vertices together
    pub is_for_collision_mesh: u8, // Is it for a collision mesh?
    pub padding: [u8; 3],

    #[br(count = num_total_influences)]
    pub skinning_influence: Vec<XACSkinInfluence>,

    #[br(count = num_org_verts)]
    pub skinning_info_table_entry: Vec<XACSkinningInfoTableEntry>,
}

#[derive(Default, Debug, Serialize, Deserialize, BinRead)]
#[br(import(num_org_verts:u32))]
#[br(little)]
pub struct XACSkinningInfo4 {
    pub node_index: u32,           // The node number in the actor
    pub lod: u32,                  // Level of detail
    pub num_local_bones: u32,      // Number of local bones used by the mesh
    pub num_total_influences: u32, // Total number of influences of all vertices together
    pub is_for_collision_mesh: u8, // Is it for a collision mesh?
    pub padding: [u8; 3],

    #[br(count = num_total_influences)]
    pub skinning_influence: Vec<XACSkinInfluence>,

    #[br(count = num_org_verts)]
    pub skinning_info_table_entry: Vec<XACSkinningInfoTableEntry>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACSkinningInfoTableEntry {
    pub start_index: u32,  // Index inside the SkinInfluence array
    pub num_elements: u32, // Number of influences for this item/entry
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACSkinInfluence {
    pub weight: f32,
    pub node_number: u32,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XACStandardMaterial {
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
pub struct XACStandardMaterial2 {
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
pub struct XACStandardMaterial3 {
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
    pub xac_node: Vec<XACNode4>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct XACChunkEntry {
    pub chunk: FileChunk,
    pub chunk_data: XACChunkData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct XACRoot {
    pub header: XACHeader,
    pub chunks: Vec<XACChunkEntry>,
}

impl XACRoot {
    /// Read XACRoot from a file path, accepting &str or &Path
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path_ref = path.as_ref();
        let file = File::open(path_ref)?;
        let mut reader = BufReader::new(file);
        let root = XACRoot {
            header: reader
                .read_le()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("binrw error: {}", e)))?,
            chunks: Self::read_chunks(&mut reader)?,
        };

        Ok(root)
    }

    /// Read XACRoot from a byte slice in memory
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let root = XACRoot {
            header: cursor
                .read_le()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("binrw error: {}", e)))?,
            chunks: Self::read_chunks(&mut cursor)?,
        };

        Ok(root)
    }

    fn read_chunks<R: Read + Seek>(reader: &mut R) -> io::Result<Vec<XACChunkEntry>> {
        let mut chunks = Vec::new();
        while let Ok(chunk) = FileChunk::read(reader) {
            let pos = reader.seek(SeekFrom::Current(0))?;
            let mut chunk_data_buf = vec![0u8; chunk.size_in_bytes as usize];
            reader.read_exact(&mut chunk_data_buf)?;

            // parse chunk_data_buf based on chunk_id
            let chunk_data = Self::parse_chunk_data(&chunk, &chunk_data_buf).unwrap();
            reader.seek(SeekFrom::Start(pos + chunk.size_in_bytes as u64))?;

            chunks.push(XACChunkEntry { chunk, chunk_data });
        }
        Ok(chunks)
    }

    fn parse_chunk_data(chunk: &FileChunk, data: &[u8]) -> Result<XACChunkData, binrw::Error> {
        let mut cursor = Cursor::new(data);

        match chunk.chunk_id {
            x if x == XACChunk::XACChunkInfo as u32 => match chunk.version {
                1 => Ok(XACChunkData::XACInfo(cursor.read_le()?)),
                2 => Ok(XACChunkData::XACInfo2(cursor.read_le()?)),
                3 => Ok(XACChunkData::XACInfo3(cursor.read_le()?)),
                4 => Ok(XACChunkData::XACInfo4(cursor.read_le()?)),
                _ => Self::unsupported(chunk, &cursor),
            },

            x if x == XACChunk::XACChunkNode as u32 => match chunk.version {
                1 => Ok(XACChunkData::XACNode(cursor.read_le()?)),
                2 => Ok(XACChunkData::XACNode2(cursor.read_le()?)),
                3 => Ok(XACChunkData::XACNode3(cursor.read_le()?)),
                4 => Ok(XACChunkData::XACNode4(cursor.read_le()?)),

                _ => Self::unsupported(chunk, &cursor),
            },

            x if x == XACChunk::XACChunkSkinninginfo as u32 => match chunk.version {
                1 => Ok(XACChunkData::XACSkinningInfo(cursor.read_le()?)),
                2 => Ok(XACChunkData::XACSkinningInfo2(cursor.read_le()?)),
                3 => Ok(XACChunkData::XACSkinningInfo3(cursor.read_le()?)),
                4 => Ok(XACChunkData::XACSkinningInfo4(cursor.read_le()?)),

                _ => Self::unsupported(chunk, &cursor),
            },

            x if x == XACChunk::XACChunkStdmaterial as u32 => match chunk.version {
                1 => Ok(XACChunkData::XACStandardMaterial(cursor.read_le()?)),
                2 => Ok(XACChunkData::XACStandardMaterial2(cursor.read_le()?)),
                3 => Ok(XACChunkData::XACStandardMaterial3(cursor.read_le()?)),

                _ => Self::unsupported(chunk, &cursor),
            },

            x if x == XACChunk::XACChunkStdmateriallayer as u32 => match chunk.version {
                1 => Ok(XACChunkData::XACStandardMaterialLayer(cursor.read_le()?)),
                2 => Ok(XACChunkData::XACStandardMaterialLayer(cursor.read_le()?)),

                _ => Self::unsupported(chunk, &cursor),
            },

            x if x == XACChunk::XACChunkMesh as u32 => match chunk.version {
                1 => Ok(XACChunkData::XACMesh(cursor.read_le()?)),
                2 => Ok(XACChunkData::XACMesh2(cursor.read_le()?)),
                _ => Self::unsupported(chunk, &cursor),
            },

            x if x == XACChunk::XACChunkLimit as u32 => {
                Ok(XACChunkData::XACLimit(cursor.read_le()?))
            }

            x if x == XACChunk::XACChunkStdprogmorphtarget as u32 => {
                Ok(XACChunkData::XACPMorphTarget(cursor.read_le()?))
            }

            x if x == XACChunk::XACChunkStdpmorphtargets as u32 => {
                Ok(XACChunkData::XACPMorphTargets(cursor.read_le()?))
            }

            x if x == XACChunk::XACChunkFxmaterial as u32 => match chunk.version {
                1 => Ok(XACChunkData::XACFXMaterial(cursor.read_le()?)),
                2 => Ok(XACChunkData::XACFXMaterial2(cursor.read_le()?)),
                3 => Ok(XACChunkData::XACFXMaterial3(cursor.read_le()?)),

                _ => Self::unsupported(chunk, &cursor),
            },

            x if x == XACChunk::XACChunkNodegroups as u32 => {
                Ok(XACChunkData::XACNodeGroup(cursor.read_le()?))
            }

            x if x == XACChunk::XACChunkNodes as u32 => {
                Ok(XACChunkData::XACNodes(cursor.read_le()?))
            }

            x if x == XACChunk::XACChunkMaterialinfo as u32 => match chunk.version {
                1 => Ok(XACChunkData::XACMaterialInfo(cursor.read_le()?)),
                2 => Ok(XACChunkData::XACMaterialInfo2(cursor.read_le()?)),
                _ => Self::unsupported(chunk, &cursor),
            },

            x if x == XACChunk::XACChunkMeshlodlevels as u32 => {
                Ok(XACChunkData::XACMeshLodLevel(cursor.read_le()?))
            }

            x if x == XACChunk::XACChunkNodemotionsources as u32 => {
                Ok(XACChunkData::XACNodeMotionSources(cursor.read_le()?))
            }

            x if x == XACChunk::XACChunkAttachmentnodes as u32 => {
                Ok(XACChunkData::XACAttachmentNodes(cursor.read_le()?))
            }

            _ => Self::unsupported(chunk, &cursor),
        }
    }

    /// helper for unsupported chunk/version
    fn unsupported(
        chunk: &FileChunk,
        cursor: &Cursor<&[u8]>,
    ) -> Result<XACChunkData, binrw::Error> {
        Err(binrw::Error::AssertFail {
            pos: cursor.position(),
            message: format!(
                "Unknown or unsupported chunk_id {} with version {}",
                chunk.chunk_id, chunk.version
            ),
        })
    }

    pub fn get_texture_names(&self) -> Vec<String> {
        let mut textures = Vec::new();

        for entry in &self.chunks {
            match entry.chunk.chunk_id {
                x if x == XACChunk::XACChunkStdmaterial as u32 => match entry.chunk.version {
                    1 => {
                        if let XACChunkData::XACStandardMaterial(mat) = &entry.chunk_data {
                            textures.push(mat.material_name.clone());
                        }
                    }
                    2 => {
                        if let XACChunkData::XACStandardMaterial2(mat) = &entry.chunk_data {
                            textures.push(mat.material_name.clone());
                        }
                    }
                    3 => {
                        if let XACChunkData::XACStandardMaterial3(mat) = &entry.chunk_data {
                            textures.push(mat.material_name.clone());
                        }
                    }
                    _ => {}
                },

                x if x == XACChunk::XACChunkFxmaterial as u32 => match entry.chunk.version {
                    1 => {
                        if let XACChunkData::XACFXMaterial(mat) = &entry.chunk_data {
                            if let Some(bitmaps) = &mat.xac_fx_bitmap_parameter {
                                for bitmap in bitmaps {
                                    textures.push(bitmap.value_name.clone());
                                }
                            }
                        }
                    }
                    2 => {
                        if let XACChunkData::XACFXMaterial2(mat) = &entry.chunk_data {
                            if let Some(bitmaps) = &mat.xac_fx_bitmap_parameter {
                                for bitmap in bitmaps {
                                    textures.push(bitmap.value_name.clone());
                                }
                            }
                        }
                    }
                    3 => {
                        if let XACChunkData::XACFXMaterial3(mat) = &entry.chunk_data {
                            if let Some(bitmaps) = &mat.xac_fx_bitmap_parameter {
                                for bitmap in bitmaps {
                                    textures.push(bitmap.value_name.clone());
                                }
                            }
                        }
                    }
                    _ => {}
                },

                _ => {}
            }
        }

        textures
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_read_xac_root() -> io::Result<()> {
        // Path to your test IES file
        let path = "tests/mage_m_bodybase.xac";

        // Read XACRoot from file
        let root = XACRoot::from_file(path)?;

        // Print for debugging (optional)
        println!("Header: {:#?}", root.header);

        println!("Textures Name: {:#?}", root.get_texture_names());

        Ok(())
    }

    #[test]
    fn test_read_xac_from_memory() -> io::Result<()> {
        // Load file into memory first
        let data = std::fs::read("tests/archer_m_falconer01.xac")?;

        // Parse from memory instead of directly from file
        let root = XACRoot::from_bytes(&data)?;

        println!("Header: {:?}", root.header);

        Ok(())
    }
}
