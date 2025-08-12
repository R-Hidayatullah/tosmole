//! XSM (Skeletal Motion) file format definitions.
//!
//! This module provides type definitions and data structures for handling
//! Skeletal Motion files (.xsm), which contain bone animation data with
//! support for both regular keyframe animation and wavelet-compressed motion data.

use std::io::{self, Read, Seek};

use crate::{
    binary::BinaryReader,
    shared_formats::{
        File16BitQuaternion, FileChunk, FileQuaternion, FileVector3, MultiplicationOrder, chunk_ids,
    },
};

/// XSM-specific chunk identifiers
pub mod xsm_chunk_ids {
    use crate::shared_formats::chunk_ids;

    pub const SUBMOTION: u32 = 200;
    pub const INFO: u32 = 201;
    pub const MOTION_EVENT_TABLE: u32 = chunk_ids::MOTION_EVENT_TABLE;
    pub const SUBMOTIONS: u32 = 202;
    pub const WAVELET_INFO: u32 = 203;
}

/// Wavelet types used during compression
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum WaveletType {
    /// Haar wavelet
    Haar = 0,
    /// Daubechies D4 wavelet
    D4 = 1,
    /// Cohen-Daubechies-Feauveau 9/7 wavelet
    Cdf97 = 2,
}

impl TryFrom<u8> for WaveletType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(WaveletType::Haar),
            1 => Ok(WaveletType::D4),
            2 => Ok(WaveletType::Cdf97),
            _ => Err("Invalid wavelet type"),
        }
    }
}

/// Compressor types for quantized data compression
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum CompressorType {
    /// Huffman compression
    Huffman = 0,
    /// Rice compression
    Rice = 1,
}

impl TryFrom<u8> for CompressorType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CompressorType::Huffman),
            1 => Ok(CompressorType::Rice),
            _ => Err("Invalid compressor type"),
        }
    }
}

/// XSM file format header
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XSMHeader {
    /// File format identifier, must be b"XSM "
    pub fourcc: [u8; 4],
    /// High version number (e.g., 2 in version 2.34)
    pub hi_version: u8,
    /// Low version number (e.g., 34 in version 2.34)
    pub lo_version: u8,
    /// Endianness of the data (0 = little endian, 1 = big endian)
    pub endian_type: u8,
    /// Matrix multiplication order (see MultiplicationOrder)
    pub mul_order: u8,
}

impl XSMHeader {
    /// Standard XSM fourcc identifier
    pub const FOURCC: [u8; 4] = *b"XSM ";

    /// Creates a new XSM header with default values
    pub fn new(hi_version: u8, lo_version: u8) -> Self {
        Self {
            fourcc: Self::FOURCC,
            hi_version,
            lo_version,
            endian_type: 0, // Little endian by default
            mul_order: MultiplicationOrder::ScaleRotTrans as u8,
        }
    }

    /// Checks if the fourcc is valid
    pub fn is_valid_fourcc(&self) -> bool {
        self.fourcc == Self::FOURCC
    }

    /// Gets the version as a tuple (major, minor)
    pub fn version(&self) -> (u8, u8) {
        (self.hi_version, self.lo_version)
    }

    /// Checks if data is stored in little endian format
    pub fn is_little_endian(&self) -> bool {
        self.endian_type == 0
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

/// XSM file information chunk (version 1)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XSMInfo {
    /// Motion frame rate in frames per second
    pub motion_fps: u32,
    /// Exporter high version number
    pub exporter_high_version: u8,
    /// Exporter low version number
    pub exporter_low_version: u8,
    padding: [u8; 2],
    // Note: In the actual file format, this is followed by:
    // - String: source application (e.g., "3D Studio MAX 7", "Maya 6.5")
    // - String: original filename of the 3DSMAX/Maya file
    // - String: compilation date of the exporter
    // - String: the name of the motion
}

impl XSMInfo {
    /// Creates a new XSM info structure
    pub fn new(motion_fps: u32, exporter_high_version: u8, exporter_low_version: u8) -> Self {
        Self {
            motion_fps,
            exporter_high_version,
            exporter_low_version,
            padding: [0; 2],
        }
    }
}

/// XSM file information chunk (version 2)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XSMInfo2 {
    /// Motion importance factor for automatic motion LOD
    pub importance_factor: f32,
    /// Maximum acceptable error for LOD system
    pub max_acceptable_error: f32,
    /// Motion frame rate in frames per second
    pub motion_fps: u32,
    /// Exporter high version number
    pub exporter_high_version: u8,
    /// Exporter low version number
    pub exporter_low_version: u8,
    padding: [u8; 2],
    // Note: Followed by the same strings as XSMInfo
}

impl XSMInfo2 {
    /// Creates a new XSM info structure (version 2)
    pub fn new(
        importance_factor: f32,
        max_acceptable_error: f32,
        motion_fps: u32,
        exporter_high_version: u8,
        exporter_low_version: u8,
    ) -> Self {
        Self {
            importance_factor,
            max_acceptable_error,
            motion_fps,
            exporter_high_version,
            exporter_low_version,
            padding: [0; 2],
        }
    }
}

/// XSM file information chunk (version 3) with motion extraction mask
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XSMInfo3 {
    /// Motion importance factor for automatic motion LOD
    pub importance_factor: f32,
    /// Maximum acceptable error for LOD system
    pub max_acceptable_error: f32,
    /// Motion frame rate in frames per second
    pub motion_fps: u32,
    /// Motion extraction mask
    pub motion_extraction_mask: u32,
    /// Exporter high version number
    pub exporter_high_version: u8,
    /// Exporter low version number
    pub exporter_low_version: u8,
    padding: [u8; 2],
    // Note: Followed by the same strings as XSMInfo
}

impl XSMInfo3 {
    /// Creates a new XSM info structure (version 3)
    pub fn new(
        importance_factor: f32,
        max_acceptable_error: f32,
        motion_fps: u32,
        motion_extraction_mask: u32,
        exporter_high_version: u8,
        exporter_low_version: u8,
    ) -> Self {
        Self {
            importance_factor,
            max_acceptable_error,
            motion_fps,
            motion_extraction_mask,
            exporter_high_version,
            exporter_low_version,
            padding: [0; 2],
        }
    }
}

/// Skeletal sub-motion data (version 1)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XSMSkeletalSubMotion {
    /// Initial pose rotation
    pub pose_rot: FileQuaternion,
    /// Bind pose rotation
    pub bind_pose_rot: FileQuaternion,
    /// Pose scale rotation
    pub pose_scale_rot: FileQuaternion,
    /// Bind pose scale rotation
    pub bind_pose_scale_rot: FileQuaternion,
    /// Initial pose position
    pub pose_pos: FileVector3,
    /// Initial pose scale
    pub pose_scale: FileVector3,
    /// Bind pose position
    pub bind_pose_pos: FileVector3,
    /// Bind pose scale
    pub bind_pose_scale: FileVector3,
    /// Number of position keyframes
    pub num_pos_keys: u32,
    /// Number of rotation keyframes
    pub num_rot_keys: u32,
    /// Number of scale keyframes
    pub num_scale_keys: u32,
    /// Number of scale rotation keyframes
    pub num_scale_rot_keys: u32,
    // Note: In the actual file format, this is followed by:
    // - String: motion part name
    // - XSMVector3Key[num_pos_keys]
    // - XSMQuaternionKey[num_rot_keys]
    // - XSMVector3Key[num_scale_keys]
    // - XSMQuaternionKey[num_scale_rot_keys]
}

/// Skeletal sub-motion data (version 2) with max error
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XSMSkeletalSubMotion2 {
    /// Initial pose rotation
    pub pose_rot: FileQuaternion,
    /// Bind pose rotation
    pub bind_pose_rot: FileQuaternion,
    /// Pose scale rotation
    pub pose_scale_rot: FileQuaternion,
    /// Bind pose scale rotation
    pub bind_pose_scale_rot: FileQuaternion,
    /// Initial pose position
    pub pose_pos: FileVector3,
    /// Initial pose scale
    pub pose_scale: FileVector3,
    /// Bind pose position
    pub bind_pose_pos: FileVector3,
    /// Bind pose scale
    pub bind_pose_scale: FileVector3,
    /// Number of position keyframes
    pub num_pos_keys: u32,
    /// Number of rotation keyframes
    pub num_rot_keys: u32,
    /// Number of scale keyframes
    pub num_scale_keys: u32,
    /// Number of scale rotation keyframes
    pub num_scale_rot_keys: u32,
    /// Maximum error for automatic motion LOD system
    pub max_error: f32,
    // Note: Same data layout as version 1 follows
}

/// Skeletal sub-motion data (version 3) with compressed quaternions
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XSMSkeletalSubMotion3 {
    /// Initial pose rotation (compressed)
    pub pose_rot: File16BitQuaternion,
    /// Bind pose rotation (compressed)
    pub bind_pose_rot: File16BitQuaternion,
    /// Pose scale rotation (compressed)
    pub pose_scale_rot: File16BitQuaternion,
    /// Bind pose scale rotation (compressed)
    pub bind_pose_scale_rot: File16BitQuaternion,
    /// Initial pose position
    pub pose_pos: FileVector3,
    /// Initial pose scale
    pub pose_scale: FileVector3,
    /// Bind pose position
    pub bind_pose_pos: FileVector3,
    /// Bind pose scale
    pub bind_pose_scale: FileVector3,
    /// Number of position keyframes
    pub num_pos_keys: u32,
    /// Number of rotation keyframes
    pub num_rot_keys: u32,
    /// Number of scale keyframes
    pub num_scale_keys: u32,
    /// Number of scale rotation keyframes
    pub num_scale_rot_keys: u32,
    /// Maximum error for automatic motion LOD system
    pub max_error: f32,
    // Note: In the actual file format, this is followed by:
    // - String: motion part name
    // - XSMVector3Key[num_pos_keys]
    // - XSM16BitQuaternionKey[num_rot_keys]
    // - XSMVector3Key[num_scale_keys]
    // - XSM16BitQuaternionKey[num_scale_rot_keys]
}

/// 3D vector keyframe
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XSMVector3Key {
    /// The vector value
    pub value: FileVector3,
    /// Time in seconds
    pub time: f32,
}

impl XSMVector3Key {
    /// Creates a new vector keyframe
    pub fn new(value: FileVector3, time: f32) -> Self {
        Self { value, time }
    }
}

/// Quaternion keyframe
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XSMQuaternionKey {
    /// The quaternion value
    pub value: FileQuaternion,
    /// Time in seconds
    pub time: f32,
}

impl XSMQuaternionKey {
    /// Creates a new quaternion keyframe
    pub fn new(value: FileQuaternion, time: f32) -> Self {
        Self { value, time }
    }
}

/// 16-bit compressed quaternion keyframe
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XSM16BitQuaternionKey {
    /// The compressed quaternion value
    pub value: File16BitQuaternion,
    /// Time in seconds
    pub time: f32,
}

impl XSM16BitQuaternionKey {
    /// Creates a new compressed quaternion keyframe
    pub fn new(value: File16BitQuaternion, time: f32) -> Self {
        Self { value, time }
    }
}

/// Regular sub-motions container (version 1)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XSMSubMotions {
    /// Number of skeletal motions
    pub num_sub_motions: u32,
    // Note: In the actual file format, this is followed by:
    // - XSMSkeletalSubMotion2[num_sub_motions]
}

/// Regular sub-motions container (version 2)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XSMSubMotions2 {
    /// Number of skeletal motions
    pub num_sub_motions: u32,
    // Note: In the actual file format, this is followed by:
    // - XSMSkeletalSubMotion3[num_sub_motions]
}

/// Wavelet sub-motion mapping entry
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XSMWaveletMapping {
    /// Position track index
    pub pos_index: u16,
    /// Rotation track index
    pub rot_index: u16,
    /// Scale rotation track index
    pub scale_rot_index: u16,
    /// Scale track index
    pub scale_index: u16,
}

impl XSMWaveletMapping {
    /// Creates a new wavelet mapping
    pub fn new(pos_index: u16, rot_index: u16, scale_rot_index: u16, scale_index: u16) -> Self {
        Self {
            pos_index,
            rot_index,
            scale_rot_index,
            scale_index,
        }
    }
}

/// Wavelet skeletal sub-motions header
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XSMWaveletInfo {
    /// Number of wavelet chunks
    pub num_chunks: u32,
    /// Samples per chunk
    pub samples_per_chunk: u32,
    /// Decompressed rotation data size in bytes
    pub decompressed_rot_num_bytes: u32,
    /// Decompressed position data size in bytes
    pub decompressed_pos_num_bytes: u32,
    /// Decompressed scale data size in bytes
    pub decompressed_scale_num_bytes: u32,
    /// Number of rotation tracks
    pub num_rot_tracks: u32,
    /// Number of scale rotation tracks
    pub num_scale_rot_tracks: u32,
    /// Number of scale tracks
    pub num_scale_tracks: u32,
    /// Number of position tracks
    pub num_pos_tracks: u32,
    /// Chunk overhead in bytes
    pub chunk_overhead: u32,
    /// Total compressed size
    pub compressed_size: u32,
    /// Optimized size
    pub optimized_size: u32,
    /// Uncompressed size
    pub uncompressed_size: u32,
    /// Scale rotation data offset
    pub scale_rot_offset: u32,
    /// Number of sub-motions
    pub num_sub_motions: u32,
    /// Position quantization factor
    pub pos_quant_factor: f32,
    /// Rotation quantization factor
    pub rot_quant_factor: f32,
    /// Scale quantization factor
    pub scale_quant_factor: f32,
    /// Sample spacing
    pub sample_spacing: f32,
    /// Seconds per chunk
    pub seconds_per_chunk: f32,
    /// Maximum time value
    pub max_time: f32,
    /// Wavelet type identifier
    pub wavelet_id: u8,
    /// Compressor type identifier
    pub compressor_id: u8,
    padding: [u8; 2],
    // Note: In the actual file format, this is followed by:
    // - XSMWaveletMapping[num_sub_motions]
    // - XSMWaveletSkeletalSubMotion[num_sub_motions]
    // - XSMWaveletChunk[num_chunks]
}

impl XSMWaveletInfo {
    /// Gets the wavelet type
    pub fn wavelet_type(&self) -> Result<WaveletType, &'static str> {
        WaveletType::try_from(self.wavelet_id)
    }

    /// Gets the compressor type
    pub fn compressor_type(&self) -> Result<CompressorType, &'static str> {
        CompressorType::try_from(self.compressor_id)
    }

    /// Calculates the total decompressed data size
    pub fn total_decompressed_size(&self) -> u32 {
        self.decompressed_rot_num_bytes
            + self.decompressed_pos_num_bytes
            + self.decompressed_scale_num_bytes
    }

    /// Calculates compression ratio
    pub fn compression_ratio(&self) -> f32 {
        if self.compressed_size > 0 {
            self.uncompressed_size as f32 / self.compressed_size as f32
        } else {
            0.0
        }
    }
}

/// Wavelet skeletal sub-motion data
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XSMWaveletSkeletalSubMotion {
    /// Initial pose rotation (compressed)
    pub pose_rot: File16BitQuaternion,
    /// Bind pose rotation (compressed)
    pub bind_pose_rot: File16BitQuaternion,
    /// Pose scale rotation (compressed)
    pub pose_scale_rot: File16BitQuaternion,
    /// Bind pose scale rotation (compressed)
    pub bind_pose_scale_rot: File16BitQuaternion,
    /// Initial pose position
    pub pose_pos: FileVector3,
    /// Initial pose scale
    pub pose_scale: FileVector3,
    /// Bind pose position
    pub bind_pose_pos: FileVector3,
    /// Bind pose scale
    pub bind_pose_scale: FileVector3,
    /// Maximum error for automatic motion LOD system
    pub max_error: f32,
    // Note: In the actual file format, this is followed by:
    // - String: motion part name
}

/// Wavelet compressed chunk
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XSMWaveletChunk {
    /// Rotation quantization scale
    pub rot_quant_scale: f32,
    /// Position quantization scale
    pub pos_quant_scale: f32,
    /// Scale quantization scale
    pub scale_quant_scale: f32,
    /// Start time for this chunk
    pub start_time: f32,
    /// Compressed rotation data size in bytes
    pub compressed_rot_num_bytes: u32,
    /// Compressed position data size in bytes
    pub compressed_pos_num_bytes: u32,
    /// Compressed scale data size in bytes
    pub compressed_scale_num_bytes: u32,
    /// Compressed position data size in bits
    pub compressed_pos_num_bits: u32,
    /// Compressed rotation data size in bits
    pub compressed_rot_num_bits: u32,
    /// Compressed scale data size in bits
    pub compressed_scale_num_bits: u32,
    // Note: In the actual file format, this is followed by:
    // - u8 compressed_rot_data[compressed_rot_num_bytes]
    // - u8 compressed_pos_data[compressed_pos_num_bytes]
    // - u8 compressed_scale_data[compressed_scale_num_bytes]
}

impl XSMWaveletChunk {
    /// Calculates total compressed data size
    pub fn total_compressed_size(&self) -> u32 {
        self.compressed_rot_num_bytes
            + self.compressed_pos_num_bytes
            + self.compressed_scale_num_bytes
    }

    /// Calculates total compressed data size in bits
    pub fn total_compressed_bits(&self) -> u32 {
        self.compressed_rot_num_bits + self.compressed_pos_num_bits + self.compressed_scale_num_bits
    }
}

/// XSM file validation and utility functions
pub mod utils {
    use super::*;

    /// Validates an XSM header
    pub fn validate_header(header: &XSMHeader) -> Result<(), &'static str> {
        if !header.is_valid_fourcc() {
            return Err("Invalid XSM fourcc identifier");
        }

        Ok(())
    }

    /// Calculates the total number of keyframes in a sub-motion
    pub fn total_keyframes(submotion: &XSMSkeletalSubMotion) -> u32 {
        submotion.num_pos_keys
            + submotion.num_rot_keys
            + submotion.num_scale_keys
            + submotion.num_scale_rot_keys
    }
}

// Type aliases for convenience
pub type Header = XSMHeader;
pub type Info = XSMInfo;
pub type Info2 = XSMInfo2;
pub type Info3 = XSMInfo3;
pub type SkeletalSubMotion = XSMSkeletalSubMotion;
pub type SkeletalSubMotion2 = XSMSkeletalSubMotion2;
pub type SkeletalSubMotion3 = XSMSkeletalSubMotion3;
pub type Vector3Key = XSMVector3Key;
pub type QuaternionKey = XSMQuaternionKey;
pub type SubMotions = XSMSubMotions;
pub type SubMotions2 = XSMSubMotions2;
pub type WaveletMapping = XSMWaveletMapping;
pub type WaveletInfo = XSMWaveletInfo;
pub type WaveletSkeletalSubMotion = XSMWaveletSkeletalSubMotion;
pub type WaveletChunk = XSMWaveletChunk;

#[derive(Debug)]
pub enum XSMChunk {
    Unknown(FileChunk, Vec<u8>), // raw data
}

#[derive(Debug)]
pub struct XSMRoot {
    pub header: XSMHeader,
    pub xsm_data: Vec<XSMChunk>, // store parsed chunks here
}

impl XSMRoot {
    pub fn read_from<R: Read + Seek>(br: &mut BinaryReader<R>) -> io::Result<Self> {
        let header = XSMHeader::read_from(br)?;
        let mut xsm_data = Vec::new();

        while let Ok(chunk_header) = FileChunk::read_from(br) {
            let bytes_left = br.bytes_left()?;
            let size_to_read =
                std::cmp::min(chunk_header.size_in_bytes as u64, bytes_left) as usize;
            // Parse chunk payload
            let chunk = match (chunk_header.chunk_id, chunk_header.version) {
                _ => XSMChunk::Unknown(chunk_header, br.read_vec(size_to_read)?),
            };

            xsm_data.push(chunk);
        }

        Ok(Self { header, xsm_data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_creation() {
        let header = XSMHeader::new(2, 34);
        assert_eq!(header.fourcc, *b"XSM ");
        assert_eq!(header.version(), (2, 34));
        assert!(header.is_valid_fourcc());
        assert!(header.is_little_endian());
    }

    #[test]
    fn test_wavelet_type_conversion() {
        assert_eq!(WaveletType::try_from(0).unwrap(), WaveletType::Haar);
        assert_eq!(WaveletType::try_from(1).unwrap(), WaveletType::D4);
        assert_eq!(WaveletType::try_from(2).unwrap(), WaveletType::Cdf97);
        assert!(WaveletType::try_from(3).is_err());
    }

    #[test]
    fn test_compressor_type_conversion() {
        assert_eq!(
            CompressorType::try_from(0).unwrap(),
            CompressorType::Huffman
        );
        assert_eq!(CompressorType::try_from(1).unwrap(), CompressorType::Rice);
        assert!(CompressorType::try_from(2).is_err());
    }

    #[test]
    fn test_wavelet_info_compression_ratio() {
        let mut info = XSMWaveletInfo {
            compressed_size: 1000,
            uncompressed_size: 5000,
            ..unsafe { std::mem::zeroed() }
        };
        assert_eq!(info.compression_ratio(), 5.0);

        info.compressed_size = 0;
        assert_eq!(info.compression_ratio(), 0.0);
    }

    #[test]
    fn test_wavelet_chunk_sizes() {
        let chunk = XSMWaveletChunk {
            compressed_rot_num_bytes: 100,
            compressed_pos_num_bytes: 200,
            compressed_scale_num_bytes: 50,
            compressed_rot_num_bits: 800,
            compressed_pos_num_bits: 1600,
            compressed_scale_num_bits: 400,
            ..unsafe { std::mem::zeroed() }
        };

        assert_eq!(chunk.total_compressed_size(), 350);
        assert_eq!(chunk.total_compressed_bits(), 2800);
    }
}
