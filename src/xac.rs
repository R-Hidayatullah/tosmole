#![allow(dead_code)]
use binrw::{BinRead, binread};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;

enum SkeletalMotionType {
    SkelmotiontypeNormal = 0, // A regular keyframe and keytrack based skeletal motion.
    SkelmotiontypeWavelet = 1, // A wavelet compressed skeletal motion.
}

enum FileType {
    FiletypeUnknown = 0,           // An unknown file, or something went wrong.
    FiletypeActor,                 // An actor file (.xac).
    FiletypeSkeletalmotion,        // A skeletal motion file (.xsm).
    FiletypeWaveletskeletalmotion, // A wavelet compressed skeletal motion (.xsm).
    FiletypePmorphmotion,          // A progressive morph motion file (.xpm).
}

// shared chunk ID's
enum SharedChunk {
    SharedChunkMotioneventtable = 50,
    SharedChunkTimestamp = 51,
}

// matrix multiplication order
enum MatrixMulOrder {
    MulorderScaleRotTrans = 0,
    MulorderRotScaleTrans = 1,
}

enum MeshType {
    MeshtypeStatic = 0, //< Static mesh, like a cube or building (can still be position/scale/rotation animated though).
    MeshtypeDynamic = 1, //< Has mesh deformers that have to be processed on the CPU.
    MeshtypeGpuskinned = 2, //< Just a skinning mesh deformer that gets processed on the GPU with skinned shader.
}

enum PhonemeSet {
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

enum WaveletType {
    WaveletHaar = 0, // The Haar wavelet, which is most likely what you want to use. It is the fastest also.
    WaveletDaub4 = 1, // Daubechies 4 wavelet, can result in bit better compression ratios, but slower than Haar.
    WaveletCdf97 = 2, // The CDF97 wavelet, used in JPG as well. This is the slowest, but often results in the best compression ratios.
}

enum NodeFlags {
    FlagIncludeinboundscalc = 1 << 0, // Specifies whether we have to include this node in the bounds calculation or not (true on default).
    FlagAttachment = 1 << 1, // Indicates if this node is an attachment node or not (false on default).
}

enum Plane {
    PlaneXy = 0, // The XY plane, so where Z is constant.
    PlaneXz = 1, // The XZ plane, so where Y is constant.
    PlaneYz = 2, // The YZ plane, so where X is constant.
}

enum DependencyType {
    DependencyMeshes = 1 << 0,     // Shared meshes.
    DependencyTransforms = 1 << 1, // Shared transforms.
}

/// The motion based actor repositioning mask
enum RepositioningMask {
    RepositionPosition = 1 << 0, // Update the actor position based on the repositioning node.
    RepositionRotation = 1 << 1, // Update the actor rotation based on the repositioning node.
    RepositionScale = 1 << 2, // [CURRENTLY UNSUPPORTED] Update the actor scale based on the repositioning node.
}

/// The order of multiplication when composing a transformation matrix from a translation, rotation and scale.
enum MultiplicationOrder {
    ScaleRotationTranslation = 0, // LocalTM = scale * rotation * translation (Maya style).
    RotationScaleTranslation = 1, // LocalTM = rotation * scale * translation (3DSMax style) [default].
}

enum LimitType {
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

enum XacAttribute {
    AttribPositions = 0, // Vertex positions. Typecast to MCore::Vector3. Positions are always exist.
    AttribNormals = 1,   // Vertex normals. Typecast to MCore::Vector3. Normals are always exist.
    AttribTangents = 2,  // Vertex tangents. Typecast to <b> MCore::Vector4 </b>.
    AttribUvcoords = 3,  // Vertex uv coordinates. Typecast to MCore::Vector2.
    AttribColors32 = 4,  // Vertex colors in 32-bits. Typecast to uint32.
    AttribOrgvtxnumbers = 5, // Original vertex numbers. Typecast to uint32. Original vertex numbers always exist.
    AttribColors128 = 6,     // Vertex colors in 128-bits. Typecast to MCore::RGBAColor.
    AttribBitangents = 7, // Vertex bitangents (aka binormal). Typecast to MCore::Vector3. When tangents exists bitangents may still not exist!
}

// collection of XAC chunk IDs
enum XacChunk {
    XacChunkNode = 0,
    XacChunkMesh = 1,
    XacChunkSkinninginfo = 2,
    XacChunkStdmaterial = 3,
    XacChunkStdmateriallayer = 4,
    XacChunkFxmaterial = 5,
    XacLimit = 6,
    XacChunkInfo = 7,
    XacChunkMeshlodlevels = 8,
    XacChunkStdprogmorphtarget = 9,
    XacChunkNodegroups = 10,
    XacChunkNodes = 11,             // XAC_Nodes
    XacChunkStdpmorphtargets = 12,  // XAC_PMorphTargets
    XacChunkMaterialinfo = 13,      // XAC_MaterialInfo
    XacChunkNodemotionsources = 14, // XAC_NodeMotionSources
    XacChunkAttachmentnodes = 15,   // XAC_AttachmentNodes
    XacForce32bit = 0xFFFFFFFF,
}

// material layer map types
enum XacMaterialLayer {
    XacLayeridUnknown = 0,       // unknown layer
    XacLayeridAmbient = 1,       // ambient layer
    XacLayeridDiffuse = 2,       // a diffuse layer
    XacLayeridSpecular = 3,      // specular layer
    XacLayeridOpacity = 4,       // opacity layer
    XacLayeridBump = 5,          // bump layer
    XacLayeridSelfillum = 6,     // self illumination layer
    XacLayeridShine = 7,         // shininess (for specular)
    XacLayeridShinestrength = 8, // shine strength (for specular)
    XacLayeridFiltercolor = 9,   // filter color layer
    XacLayeridReflect = 10,      // reflection layer
    XacLayeridRefract = 11,      // refraction layer
    XacLayeridEnvironment = 12,  // environment map layer
    XacLayeridDisplacement = 13, // displacement map layer
    XacLayeridForce8bit = 0xFF,  // don't use more than 8 bit values
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
