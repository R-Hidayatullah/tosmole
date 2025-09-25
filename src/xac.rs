//! XAC (Actor) file format definitions.
//!
//! This module provides type definitions and data structures for handling
//! Actor files (.xac), which contain 3D model data including meshes, materials,
//! skeletal hierarchies, skinning information, and morph targets.

use std::io::{self, Read, Seek};

use crate::{
    binary::BinaryReader,
    shared_formats::{FileChunk, FileColor, FileQuaternion, FileVector3, MultiplicationOrder},
};

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

/// XAC file format header
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACHeader {
    /// File format identifier, must be b"XAC "
    pub fourcc: [u8; 4],
    /// High version number
    pub hi_version: u8,
    /// Low version number
    pub lo_version: u8,
    /// Endianness (0 = little, 1 = big)
    pub endian_type: u8,
    /// Matrix multiplication order
    pub mul_order: u8,
}

impl XACHeader {
    /// Standard XAC fourcc identifier
    pub const FOURCC: [u8; 4] = *b"XAC ";

    /// Creates a new XAC header
    pub fn new(hi_version: u8, lo_version: u8) -> Self {
        Self {
            fourcc: Self::FOURCC,
            hi_version,
            lo_version,
            endian_type: 0, // Little endian by default
            mul_order: MultiplicationOrder::ScaleRotTrans as u8,
        }
    }

    /// Validates the fourcc identifier
    pub fn is_valid_fourcc(&self) -> bool {
        self.fourcc == Self::FOURCC
    }

    /// Gets version as tuple
    pub fn version(&self) -> (u8, u8) {
        (self.hi_version, self.lo_version)
    }

    pub fn read_from<R: Read + Seek>(br: &mut BinaryReader<R>) -> io::Result<Self> {
        Ok(Self {
            fourcc: br.read_exact::<4>()?,
            hi_version: br.read_u8()?,
            lo_version: br.read_u8()?,
            endian_type: br.read_u8()?,
            mul_order: br.read_u8()?,
        })
    }
}

/// XAC file information (version 1)
#[derive(Debug, Clone)]
#[repr(C)]
pub struct XACInfo {
    /// Repositioning mask for transformation components
    pub repositioning_mask: u32,
    /// Node index for repositioning
    pub repositioning_node_index: u32,
    /// Exporter high version
    pub exporter_high_version: u8,
    /// Exporter low version
    pub exporter_low_version: u8,
    padding: u16,
}

/// XAC file information (version 2)
#[derive(Debug, Clone)]
#[repr(C)]
pub struct XACInfo2 {
    /// Repositioning mask for transformation components
    pub repositioning_mask: u32,
    /// Node index for repositioning
    pub repositioning_node_index: u32,
    /// Exporter high version
    pub exporter_high_version: u8,
    /// Exporter low version
    pub exporter_low_version: u8,
    /// Retarget root offset
    pub retarget_root_offset: f32,
    padding: u16,
}

/// XAC file information (version 3)
#[derive(Debug, Clone)]
#[repr(C)]
pub struct XACInfo3 {
    /// Trajectory node index
    pub trajectory_node_index: u32,
    /// Motion extraction node index
    pub motion_extraction_node_index: u32,
    /// Motion extraction mask
    pub motion_extraction_mask: u32,
    /// Exporter high version
    pub exporter_high_version: u8,
    /// Exporter low version
    pub exporter_low_version: u8,
    /// Retarget root offset
    pub retarget_root_offset: f32,
    padding: u16,
}

/// XAC file information (version 4)
#[derive(Debug, Clone)]
#[repr(C)]
pub struct XACInfo4 {
    /// Number of level of details
    pub num_lods: u32,
    /// Trajectory node index
    pub trajectory_node_index: u32,
    /// Motion extraction node index
    pub motion_extraction_node_index: u32,
    /// Exporter high version
    pub exporter_high_version: u8,
    /// Exporter low version
    pub exporter_low_version: u8,
    /// Retarget root offset
    pub retarget_root_offset: f32,
    padding: u16,
}

/// Node structure (version 1)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACNode {
    /// Local rotation quaternion
    pub local_quat: FileQuaternion,
    /// Scale rotation quaternion
    pub scale_rot: FileQuaternion,
    /// Local position
    pub local_pos: FileVector3,
    /// Local scale
    pub local_scale: FileVector3,
    /// Shear values (x=XY, y=XZ, z=YZ)
    pub shear: FileVector3,
    /// Skeletal LOD bits (each bit = active in that LOD level)
    pub skeletal_lods: u32,
    /// Parent node index (0xFFFFFFFF = root node)
    pub parent_index: u32,
}

impl XACNode {
    /// Root node indicator
    pub const ROOT_NODE_INDEX: u32 = 0xFFFFFFFF;

    /// Checks if this is a root node
    pub fn is_root(&self) -> bool {
        self.parent_index == Self::ROOT_NODE_INDEX
    }

    /// Checks if node is active in given LOD level
    pub fn is_active_in_lod(&self, lod_level: u32) -> bool {
        if lod_level < 32 {
            (self.skeletal_lods & (1 << lod_level)) != 0
        } else {
            false
        }
    }

    /// Creates a new default node
    pub fn new() -> Self {
        Self {
            local_quat: FileQuaternion::identity(),
            scale_rot: FileQuaternion::identity(),
            local_pos: FileVector3::default(),
            local_scale: FileVector3::default(),
            shear: FileVector3::default(),
            skeletal_lods: 0,
            parent_index: Self::ROOT_NODE_INDEX,
        }
    }

    /// Reads an XACNode from a binary stream
    pub fn read_from<R: Read + Seek>(br: &mut BinaryReader<R>, size: u32) -> io::Result<Self> {
        let start_pos = br.position()?;

        let local_quat = FileQuaternion::read_from(br)?;
        let scale_rot = FileQuaternion::read_from(br)?;
        let local_pos = FileVector3::read_from(br)?;
        let local_scale = FileVector3::read_from(br)?;
        let shear = FileVector3::read_from(br)?;
        let skeletal_lods = br.read_u32()?;
        let parent_index = br.read_u32()?;

        let end_pos = br.position()?;
        let parsed_bytes = (end_pos - start_pos) as u32;

        if parsed_bytes != size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "XACNode chunk size mismatch: expected {}, parsed {}",
                    size, parsed_bytes
                ),
            ));
        }
        Ok(Self {
            local_quat,
            scale_rot,
            local_pos,
            local_scale,
            shear,
            skeletal_lods,
            parent_index,
        })
    }
}

/// Node structure (version 2) with flags
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACNode2 {
    /// Local rotation quaternion
    pub local_quat: FileQuaternion,
    /// Scale rotation quaternion
    pub scale_rot: FileQuaternion,
    /// Local position
    pub local_pos: FileVector3,
    /// Local scale
    pub local_scale: FileVector3,
    /// Shear values
    pub shear: FileVector3,
    /// Skeletal LOD bits
    pub skeletal_lods: u32,
    /// Parent node index
    pub parent_index: u32,
    /// Node flags
    pub node_flags: u8,
    padding: [u8; 3],
}

/// Node structure (version 3) with OBB
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACNode3 {
    /// Local rotation quaternion
    pub local_quat: FileQuaternion,
    /// Scale rotation quaternion
    pub scale_rot: FileQuaternion,
    /// Local position
    pub local_pos: FileVector3,
    /// Local scale
    pub local_scale: FileVector3,
    /// Shear values
    pub shear: FileVector3,
    /// Skeletal LOD bits
    pub skeletal_lods: u32,
    /// Parent node index
    pub parent_index: u32,
    /// Node flags
    pub node_flags: u8,
    padding: [u8; 3],
    /// Oriented bounding box (4x4 matrix)
    pub obb: [f32; 16],
}

/// Node structure (version 4) with motion LODs
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACNode4 {
    /// Local rotation quaternion
    pub local_quat: FileQuaternion,
    /// Scale rotation quaternion
    pub scale_rot: FileQuaternion,
    /// Local position
    pub local_pos: FileVector3,
    /// Local scale
    pub local_scale: FileVector3,
    /// Shear values
    pub shear: FileVector3,
    /// Skeletal LOD bits
    pub skeletal_lods: u32,
    /// Motion LOD bits
    pub motion_lods: u32,
    /// Parent node index
    pub parent_index: u32,
    /// Number of child nodes
    pub num_children: u32,
    /// Node flags
    pub node_flags: u8,
    padding: [u8; 3],
    /// Oriented bounding box
    pub obb: [f32; 16],
    /// Importance factor for automatic motion LOD
    pub importance_factor: f32,
}

impl XACNode4 {
    /// Checks if node is active in given motion LOD level
    pub fn is_active_in_motion_lod(&self, lod_level: u32) -> bool {
        if lod_level < 32 {
            (self.motion_lods & (1 << lod_level)) != 0
        } else {
            false
        }
    }
}

/// Mesh LOD level
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACMeshLodLevel {
    /// LOD level number
    pub lod_level: u32,
    /// Size of LOD data in bytes
    pub size_in_bytes: u32,
    // Note: Followed by array[u8] - The LOD model memory file
}

/// UV texture coordinate
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[repr(C)]
pub struct XACUv {
    /// U coordinate
    pub u: f32,
    /// V coordinate
    pub v: f32,
}

impl XACUv {
    pub fn new(u: f32, v: f32) -> Self {
        Self { u, v }
    }
}

/// Skinning information (version 1)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACSkinningInfo {
    /// Node index this skinning belongs to
    pub node_index: u32,
    /// Is this for a collision mesh?
    pub is_for_collision_mesh: bool,
    padding: [u8; 3],
}

/// Skinning information (version 2)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACSkinningInfo2 {
    /// Node index this skinning belongs to
    pub node_index: u32,
    /// Total number of influences across all vertices
    pub num_total_influences: u32,
    /// Is this for a collision mesh?
    pub is_for_collision_mesh: bool,
    padding: [u8; 3],
}

/// Skinning information (version 3)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACSkinningInfo3 {
    /// Node index this skinning belongs to
    pub node_index: u32,
    /// Number of local bones to reserve space for
    pub num_local_bones: u32,
    /// Total number of influences across all vertices
    pub num_total_influences: u32,
    /// Is this for a collision mesh?
    pub is_for_collision_mesh: bool,
    padding: [u8; 3],
}

/// Skinning information (version 4) with LOD support
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACSkinningInfo4 {
    /// Node index this skinning belongs to
    pub node_index: u32,
    /// Level of detail
    pub lod: u32,
    /// Number of local bones to reserve space for
    pub num_local_bones: u32,
    /// Total number of influences across all vertices
    pub num_total_influences: u32,
    /// Is this for a collision mesh?
    pub is_for_collision_mesh: bool,
    padding: [u8; 3],
}

/// Skinning information table entry
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACSkinningInfoTableEntry {
    /// Start index in the SkinInfluence array
    pub start_index: u32,
    /// Number of influences for this vertex
    pub num_elements: u32,
}

/// Skinning influence data
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct XACSkinInfluence {
    /// Influence weight
    pub weight: f32,
    /// Node number
    pub node_nr: u16,
    padding: [u8; 2],
}

impl XACSkinInfluence {
    pub fn new(weight: f32, node_nr: u16) -> Self {
        Self {
            weight,
            node_nr,
            padding: [0; 2],
        }
    }
}

/// Standard material
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACStandardMaterial {
    /// Ambient color
    pub ambient: FileColor,
    /// Diffuse color
    pub diffuse: FileColor,
    /// Specular color
    pub specular: FileColor,
    /// Emissive color
    pub emissive: FileColor,
    /// Shininess
    pub shine: f32,
    /// Shine strength
    pub shine_strength: f32,
    /// Opacity (1.0 = opaque, 0.0 = transparent)
    pub opacity: f32,
    /// Index of refraction
    pub ior: f32,
    /// Is double-sided?
    pub double_sided: bool,
    /// Render in wireframe?
    pub wireframe: bool,
    /// Transparency type (F=filter, S=subtractive, A=additive, U=unknown)
    pub transparency_type: u8,
    padding: u8,
}

/// Standard material (version 2) with layers
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACStandardMaterial2 {
    /// Ambient color
    pub ambient: FileColor,
    /// Diffuse color
    pub diffuse: FileColor,
    /// Specular color
    pub specular: FileColor,
    /// Emissive color
    pub emissive: FileColor,
    /// Shininess
    pub shine: f32,
    /// Shine strength
    pub shine_strength: f32,
    /// Opacity
    pub opacity: f32,
    /// Index of refraction
    pub ior: f32,
    /// Is double-sided?
    pub double_sided: bool,
    /// Render in wireframe?
    pub wireframe: bool,
    /// Transparency type
    pub transparency_type: u8,
    /// Number of material layers
    pub num_layers: u8,
}

/// Standard material (version 3) with LOD support
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACStandardMaterial3 {
    /// Level of detail
    pub lod: u32,
    /// Ambient color
    pub ambient: FileColor,
    /// Diffuse color
    pub diffuse: FileColor,
    /// Specular color
    pub specular: FileColor,
    /// Emissive color
    pub emissive: FileColor,
    /// Shininess
    pub shine: f32,
    /// Shine strength
    pub shine_strength: f32,
    /// Opacity
    pub opacity: f32,
    /// Index of refraction
    pub ior: f32,
    /// Is double-sided?
    pub double_sided: bool,
    /// Render in wireframe?
    pub wireframe: bool,
    /// Transparency type
    pub transparency_type: u8,
    /// Number of material layers
    pub num_layers: u8,
}

/// Material layer (version 1)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACStandardMaterialLayer {
    /// Layer amount (0.0 to 1.0)
    pub amount: f32,
    /// Horizontal texture offset
    pub u_offset: f32,
    /// Vertical texture offset
    pub v_offset: f32,
    /// Horizontal tiling factor
    pub u_tiling: f32,
    /// Vertical tiling factor
    pub v_tiling: f32,
    /// Texture rotation in radians
    pub rotation_radians: f32,
    /// Parent material number
    pub material_number: u16,
    /// Map type (see LayerId)
    pub map_type: u8,
    padding: u8,
}

/// Material layer (version 2) with blend mode
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACStandardMaterialLayer2 {
    /// Layer amount (0.0 to 1.0)
    pub amount: f32,
    /// Horizontal texture offset
    pub u_offset: f32,
    /// Vertical texture offset
    pub v_offset: f32,
    /// Horizontal tiling factor
    pub u_tiling: f32,
    /// Vertical tiling factor
    pub v_tiling: f32,
    /// Texture rotation in radians
    pub rotation_radians: f32,
    /// Parent material number
    pub material_number: u16,
    /// Map type (see LayerId)
    pub map_type: u8,
    /// Blend mode for layer combination
    pub blend_mode: u8,
}

/// Vertex attribute layer
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACVertexAttributeLayer {
    /// Type of vertex attribute
    pub layer_type_id: u32,
    /// Size of single vertex attribute in bytes
    pub attrib_size_in_bytes: u32,
    /// Enable deformations on this layer?
    pub enable_deformations: bool,
    /// Is this a scale value?
    pub is_scale: bool,
    padding: [u8; 2],
}

/// Sub-mesh definition
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACSubMesh {
    /// Number of indices
    pub num_indices: u32,
    /// Number of vertices
    pub num_verts: u32,
    /// Material index
    pub material_index: u32,
    /// Number of bones used by this sub-mesh
    pub num_bones: u32,
}

/// Mesh structure (version 1)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACMesh {
    /// Node index this mesh belongs to
    pub node_index: u32,
    /// Number of original vertices
    pub num_org_verts: u32,
    /// Total number of vertices across all sub-meshes
    pub total_verts: u32,
    /// Total number of indices across all sub-meshes
    pub total_indices: u32,
    /// Number of sub-meshes
    pub num_sub_meshes: u32,
    /// Number of vertex attribute layers
    pub num_layers: u32,
    /// Is this a collision mesh?
    pub is_collision_mesh: bool,
    padding: [u8; 3],
}

/// Mesh structure (version 2) with LOD support
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACMesh2 {
    /// Node index this mesh belongs to
    pub node_index: u32,
    /// Level of detail
    pub lod: u32,
    /// Number of original vertices
    pub num_org_verts: u32,
    /// Total number of vertices
    pub total_verts: u32,
    /// Total number of indices
    pub total_indices: u32,
    /// Number of sub-meshes
    pub num_sub_meshes: u32,
    /// Number of vertex attribute layers
    pub num_layers: u32,
    /// Is this a collision mesh?
    pub is_collision_mesh: bool,
    padding: [u8; 3],
}

/// Node transformation limits
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACLimit {
    /// Minimum translation values
    pub translation_min: FileVector3,
    /// Maximum translation values
    pub translation_max: FileVector3,
    /// Minimum rotation values
    pub rotation_min: FileVector3,
    /// Maximum rotation values
    pub rotation_max: FileVector3,
    /// Minimum scale values
    pub scale_min: FileVector3,
    /// Maximum scale values
    pub scale_max: FileVector3,
    /// Limit type activation flags
    pub limit_flags: [u8; 9],
    padding: [u8; 3],
    /// Node number this limit applies to
    pub node_number: u32,
}

/// Progressive morph target
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACPMorphTarget {
    /// Slider minimum value
    pub range_min: f32,
    /// Slider maximum value
    pub range_max: f32,
    /// Level of detail this target belongs to
    pub lod: u32,
    /// Number of mesh deform deltas
    pub num_mesh_deform_deltas: u32,
    /// Number of transformations
    pub num_transformations: u32,
    /// Phoneme sets
    pub phoneme_sets: u32,
}

/// Progressive morph targets container
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACPMorphTargets {
    /// Number of morph targets
    pub num_morph_targets: u32,
    /// LOD level for these morph targets
    pub lod: u32,
}

/// Morph target mesh deltas
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACPMorphTargetMeshDeltas {
    /// Node index
    pub node_index: u32,
    /// Minimum range value for compressed position vectors
    pub min_value: f32,
    /// Maximum range value for compressed position vectors
    pub max_value: f32,
    /// Number of delta vertices
    pub num_vertices: u32,
    // Note: In the actual file format, this is followed by:
    // - File16BitVector3[num_vertices] (delta position values)
    // - File8BitVector3[num_vertices] (delta normal values)
    // - File8BitVector3[num_vertices] (delta tangent values)
    // - u32[num_vertices] (vertex numbers)
}

/// Progressive morph target transformation
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACPMorphTargetTransform {
    /// Node index where the transform belongs
    pub node_index: u32,
    /// Node rotation
    pub rotation: FileQuaternion,
    /// Node delta scale rotation
    pub scale_rotation: FileQuaternion,
    /// Node delta position
    pub position: FileVector3,
    /// Node delta scale
    pub scale: FileVector3,
}

/// FX material (version 1)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACFxMaterial {
    /// Number of integer parameters
    pub num_int_params: u32,
    /// Number of float parameters
    pub num_float_params: u32,
    /// Number of color parameters
    pub num_color_params: u32,
    /// Number of bitmap parameters
    pub num_bitmap_params: u32,
    // Note: In the actual file format, this is followed by:
    // - String: name
    // - String: effect file (path excluded, extension included)
    // - XACFxIntParameter[num_int_params]
    // - XACFxFloatParameter[num_float_params]
    // - XACFxColorParameter[num_color_params]
    // - [num_bitmap_params] bitmap parameter entries
}

/// FX material (version 2) with additional parameter types
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACFxMaterial2 {
    /// Number of integer parameters
    pub num_int_params: u32,
    /// Number of float parameters
    pub num_float_params: u32,
    /// Number of color parameters
    pub num_color_params: u32,
    /// Number of boolean parameters
    pub num_bool_params: u32,
    /// Number of Vector3 parameters
    pub num_vector3_params: u32,
    /// Number of bitmap parameters
    pub num_bitmap_params: u32,
    // Note: Followed by name, effect file, shader technique, and parameter arrays
}

/// FX material (version 3) with LOD support
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACFxMaterial3 {
    /// Level of detail
    pub lod: u32,
    /// Number of integer parameters
    pub num_int_params: u32,
    /// Number of float parameters
    pub num_float_params: u32,
    /// Number of color parameters
    pub num_color_params: u32,
    /// Number of boolean parameters
    pub num_bool_params: u32,
    /// Number of Vector3 parameters
    pub num_vector3_params: u32,
    /// Number of bitmap parameters
    pub num_bitmap_params: u32,
}

/// FX material integer parameter
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACFxIntParameter {
    /// Parameter value (can be negative)
    pub value: i32,
    // Note: Followed by string name
}

/// FX material float parameter
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACFxFloatParameter {
    /// Parameter value
    pub value: f32,
    // Note: Followed by string name
}

/// FX material color parameter
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACFxColorParameter {
    /// Color value
    pub value: FileColor,
    // Note: Followed by string name
}

/// FX material Vector3 parameter
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACFxVector3Parameter {
    /// Vector3 value
    pub value: FileVector3,
    // Note: Followed by string name
}

/// FX material boolean parameter
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACFxBoolParameter {
    /// Boolean value (0 = false, 1 = true)
    pub value: bool,
    // Note: Followed by string name
}

/// Node group
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACNodeGroup {
    /// Number of nodes in this group
    pub num_nodes: u16,
    /// Disabled by default? (0 = no, 1 = yes)
    pub disabled_on_default: bool,
    padding: u8,
    // Note: In the actual file format, this is followed by:
    // - String: group name
    // - u16[num_nodes] (node indices)
}

/// Collection of all nodes
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACNodes {
    /// Total number of nodes
    pub num_nodes: u32,
    /// Number of root nodes
    pub num_root_nodes: u32,
    // Note: Followed by XACNode4[num_nodes]
}

/// Material statistics (version 1)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACMaterialInfo {
    /// Total number of materials
    pub num_total_materials: u32,
    /// Number of standard materials
    pub num_standard_materials: u32,
    /// Number of FX materials
    pub num_fx_materials: u32,
}

/// Material statistics (version 2) with LOD support
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACMaterialInfo2 {
    /// Level of detail
    pub lod: u32,
    /// Total number of materials
    pub num_total_materials: u32,
    /// Number of standard materials
    pub num_standard_materials: u32,
    /// Number of FX materials
    pub num_fx_materials: u32,
}

/// Node motion sources for motion mirroring
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACNodeMotionSources {
    /// Number of nodes
    pub num_nodes: u32,
    // Note: Followed by u16[num_nodes] - indices of nodes to extract motion data from
}

/// Attachment nodes list
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XACAttachmentNodes {
    /// Number of attachment nodes
    pub num_nodes: u32,
    // Note: Followed by u16[num_nodes] - attachment node indices
}

/// Transparency types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TransparencyType {
    /// Filter transparency
    Filter = b'F',
    /// Subtractive transparency
    Subtractive = b'S',
    /// Additive transparency
    Additive = b'A',
    /// Unknown transparency type
    Unknown = b'U',
}

impl TryFrom<u8> for TransparencyType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'F' => Ok(TransparencyType::Filter),
            b'S' => Ok(TransparencyType::Subtractive),
            b'A' => Ok(TransparencyType::Additive),
            b'U' => Ok(TransparencyType::Unknown),
            _ => Err("Invalid transparency type"),
        }
    }
}

/// XAC file validation and utility functions
pub mod utils {
    use super::*;

    /// Validates an XAC header
    pub fn validate_header(header: &XACHeader) -> Result<(), &'static str> {
        if !header.is_valid_fourcc() {
            return Err("Invalid XAC fourcc identifier");
        }
        Ok(())
    }

    /// Calculates total size needed for skinning influences
    pub fn calculate_skinning_data_size(num_influences: u32, num_vertices: u32) -> usize {
        (num_influences as usize * std::mem::size_of::<XACSkinInfluence>())
            + (num_vertices as usize * std::mem::size_of::<XACSkinningInfoTableEntry>())
    }

    /// Checks if a node is a valid parent for another node
    pub fn is_valid_parent_relationship(parent_index: u32, child_index: u32) -> bool {
        parent_index != child_index && parent_index != XACNode::ROOT_NODE_INDEX
    }

    /// Calculates the total number of parameters in an FX material
    pub fn total_fx_params(material: &XACFxMaterial2) -> u32 {
        material.num_int_params
            + material.num_float_params
            + material.num_color_params
            + material.num_bool_params
            + material.num_vector3_params
            + material.num_bitmap_params
    }
}

// Type aliases for convenience and backward compatibility
pub type Header = XACHeader;
pub type Info = XACInfo;
pub type Info2 = XACInfo2;
pub type Info3 = XACInfo3;
pub type Info4 = XACInfo4;
pub type Node = XACNode;
pub type Node2 = XACNode2;
pub type Node3 = XACNode3;
pub type Node4 = XACNode4;
pub type MeshLodLevel = XACMeshLodLevel;
pub type Uv = XACUv;
pub type SkinningInfo = XACSkinningInfo;
pub type SkinningInfo2 = XACSkinningInfo2;
pub type SkinningInfo3 = XACSkinningInfo3;
pub type SkinningInfo4 = XACSkinningInfo4;
pub type SkinningInfoTableEntry = XACSkinningInfoTableEntry;
pub type SkinInfluence = XACSkinInfluence;
pub type StandardMaterial = XACStandardMaterial;
pub type StandardMaterial2 = XACStandardMaterial2;
pub type StandardMaterial3 = XACStandardMaterial3;
pub type StandardMaterialLayer = XACStandardMaterialLayer;
pub type StandardMaterialLayer2 = XACStandardMaterialLayer2;
pub type VertexAttributeLayer = XACVertexAttributeLayer;
pub type SubMesh = XACSubMesh;
pub type Mesh = XACMesh;
pub type Mesh2 = XACMesh2;
pub type Limit = XACLimit;
pub type PMorphTarget = XACPMorphTarget;
pub type PMorphTargets = XACPMorphTargets;
pub type PMorphTargetMeshDeltas = XACPMorphTargetMeshDeltas;
pub type PMorphTargetTransform = XACPMorphTargetTransform;
pub type FxMaterial = XACFxMaterial;
pub type FxMaterial2 = XACFxMaterial2;
pub type FxMaterial3 = XACFxMaterial3;
pub type NodeGroup = XACNodeGroup;
pub type Nodes = XACNodes;
pub type MaterialInfo = XACMaterialInfo;
pub type MaterialInfo2 = XACMaterialInfo2;
pub type NodeMotionSources = XACNodeMotionSources;
pub type AttachmentNodes = XACAttachmentNodes;

#[derive(Debug)]
pub enum XACChunk {
    Unknown(FileChunk, Vec<u8>), // raw data
    Node(FileChunk, XACNode),
}

#[derive(Debug)]
pub struct XACRoot {
    pub header: XACHeader,
    pub xac_data: Vec<XACChunk>, // store parsed chunks here
}

impl XACRoot {
    pub fn read_from<R: Read + Seek>(br: &mut BinaryReader<R>) -> io::Result<Self> {
        let header = XACHeader::read_from(br)?;
        let mut xac_data = Vec::new();

        while let Ok(chunk_header) = FileChunk::read_from(br) {
            let bytes_left = br.bytes_left()?;
            let size_to_read =
                std::cmp::min(chunk_header.size_in_bytes as u64, bytes_left) as usize;
            // Parse chunk payload
            let chunk = match (chunk_header.chunk_id, chunk_header.version) {
                (xac_chunk_ids::NODE, 1) => {
                    let node = XACNode::read_from(br, chunk_header.size_in_bytes)?;
                    XACChunk::Node(chunk_header, node)
                }
                _ => XACChunk::Unknown(chunk_header, (&mut *br).read_vec(size_to_read)?),
            };

            xac_data.push(chunk);
        }

        Ok(Self { header, xac_data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_creation() {
        let header = XACHeader::new(2, 34);
        assert_eq!(header.fourcc, *b"XAC ");
        assert_eq!(header.version(), (2, 34));
        assert!(header.is_valid_fourcc());
    }

    #[test]
    fn test_node_root_detection() {
        let root_node = XACNode {
            parent_index: XACNode::ROOT_NODE_INDEX,
            ..unsafe { std::mem::zeroed() }
        };
        assert!(root_node.is_root());

        let child_node = XACNode {
            parent_index: 0,
            ..unsafe { std::mem::zeroed() }
        };
        assert!(!child_node.is_root());
    }

    #[test]
    fn test_node_lod_checking() {
        let node = XACNode {
            skeletal_lods: 0b1010, // Active in LODs 1 and 3
            ..unsafe { std::mem::zeroed() }
        };

        assert!(!node.is_active_in_lod(0));
        assert!(node.is_active_in_lod(1));
        assert!(!node.is_active_in_lod(2));
        assert!(node.is_active_in_lod(3));
    }

    #[test]
    fn test_phoneme_set_operations() {
        let set1 = PhonemeSet::NEUTRAL_POSE;
        let set2 = PhonemeSet::M_B_P_X;
        let combined = set1.union(set2);

        assert!(combined.contains(set1));
        assert!(combined.contains(set2));
        assert!(!set1.contains(set2));
    }

    #[test]
    fn test_transparency_type_conversion() {
        assert_eq!(
            TransparencyType::try_from(b'F').unwrap(),
            TransparencyType::Filter
        );
        assert_eq!(
            TransparencyType::try_from(b'S').unwrap(),
            TransparencyType::Subtractive
        );
        assert_eq!(
            TransparencyType::try_from(b'A').unwrap(),
            TransparencyType::Additive
        );
        assert_eq!(
            TransparencyType::try_from(b'U').unwrap(),
            TransparencyType::Unknown
        );
        assert!(TransparencyType::try_from(b'X').is_err());
    }

    #[test]
    fn test_skin_influence_creation() {
        let influence = XACSkinInfluence::new(0.75, 5);
        assert_eq!(influence.weight, 0.75);
        assert_eq!(influence.node_nr, 5);
    }

    #[test]
    fn test_uv_coordinates() {
        let uv = XACUv::new(0.5, 0.8);
        assert_eq!(uv.u, 0.5);
        assert_eq!(uv.v, 0.8);
    }

    #[test]
    fn test_node4_motion_lod() {
        let node = XACNode4 {
            motion_lods: 0b1100, // Active in motion LODs 2 and 3
            ..unsafe { std::mem::zeroed() }
        };

        assert!(!node.is_active_in_motion_lod(0));
        assert!(!node.is_active_in_motion_lod(1));
        assert!(node.is_active_in_motion_lod(2));
        assert!(node.is_active_in_motion_lod(3));
    }

    #[test]
    fn test_clone_flags() {
        let flags = ActorCloneFlags::DEFAULT;
        assert_eq!(
            flags.0,
            ActorCloneFlags::NODE_ATTRIBUTES.0 | ActorCloneFlags::CONTROLLERS.0
        );

        let all_flags = ActorCloneFlags::ALL;
        assert!(all_flags.0 & ActorCloneFlags::MATERIALS.0 != 0);
        assert!(all_flags.0 & ActorCloneFlags::MESHES.0 != 0);
    }

    #[test]
    fn test_limit_type_flags() {
        let translation_limits = LimitType::TRANSLATION_X.0 | LimitType::TRANSLATION_Y.0;
        assert_eq!(translation_limits, 0b11);

        let rotation_limits =
            LimitType::ROTATION_X.0 | LimitType::ROTATION_Y.0 | LimitType::ROTATION_Z.0;
        assert_eq!(rotation_limits, 0b111000);
    }

    #[test]
    fn test_utils_functions() {
        let header = XACHeader::new(1, 0);
        assert!(utils::validate_header(&header).is_ok());

        let data_size = utils::calculate_skinning_data_size(100, 50);
        let expected_size = 100 * std::mem::size_of::<XACSkinInfluence>()
            + 50 * std::mem::size_of::<XACSkinningInfoTableEntry>();
        assert_eq!(data_size, expected_size);

        assert!(utils::is_valid_parent_relationship(0, 1));
        assert!(!utils::is_valid_parent_relationship(1, 1)); // Self-parent
        assert!(!utils::is_valid_parent_relationship(
            XACNode::ROOT_NODE_INDEX,
            1
        )); // Root as parent
    }
}
