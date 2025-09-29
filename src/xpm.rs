//! XPM (Progressive Morph Motion) file format definitions.
//!
//! This module provides type definitions and data structures for handling
//! Progressive Morph Motion files (.xpm), which contain facial animation
//! and morph target data with phoneme sets for speech animation.

use binrw::{BinRead, BinReaderExt, BinResult, binread};
use serde::{Deserialize, Serialize};

/// XPM-specific chunk identifiers
pub enum XPMChunk {
    SUBMOTION = 100,
    INFO = 101,
    MotionEventTable = 50,
    SUBMOTIONS = 102,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum XPMChunkData {}

/// XPM file format header
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XPMHeader {
    /// File format identifier, must be b"XPM "
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

/// XPM file information chunk
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XPMInfo {
    /// Motion frame rate in frames per second
    pub motion_fps: u32,
    /// Exporter high version number
    pub exporter_high_version: u8,
    /// Exporter low version number
    pub exporter_low_version: u8,
    padding: [u8; 2],

    #[br(temp)]
    source_app_length: u32,
    #[br(count = source_app_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub source_app: String,

    #[br(temp)]
    original_filename_length: u32,
    #[br(count = original_filename_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub original_filename: String,

    #[br(temp)]
    compilation_date_length: u32,
    #[br(count = compilation_date_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub compilation_date: String,

    #[br(temp)]
    motion_name_length: u32,
    #[br(count = motion_name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub motion_name: String,
}

/// Progressive morph sub-motion data
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XPMProgressiveSubMotion {
    /// Pose weight to use when no animation data is present
    pub pose_weight: f32,
    /// Minimum allowed weight value (used for unpacking keyframe weights)
    pub min_weight: f32,
    /// Maximum allowed weight value (used for unpacking keyframe weights)
    pub max_weight: f32,
    /// Phoneme set identifier (0 if this is a normal progressive morph target)
    pub phoneme_set: u32,
    /// Number of keyframes that follow this structure
    pub num_keys: u32,
    #[br(temp)]
    name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub name: String,
    #[br(count = num_keys)]
    pub xpm_key: Vec<XPMUnsignedShortKey>,
}

/// Floating-point keyframe data
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XPMFloatKey {
    /// Time in seconds
    pub time: f32,
    /// Keyframe value
    pub value: f32,
}

/// Compressed 16-bit unsigned integer keyframe data
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XPMUnsignedShortKey {
    /// Time in seconds
    pub time: f32,
    /// Compressed keyframe value (16-bit unsigned integer)
    pub value: u16,
    padding: [u8; 2],
}

/// Container for multiple sub-motions
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XPMSubMotions {
    /// Number of sub-motions in this container
    pub num_sub_motions: u32,
    // Note: In the actual file format, this is followed by:
    #[br(count = num_sub_motions)]
    pub progressive_sub_motions: Vec<XPMProgressiveSubMotion>,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XPMRoot {
    pub header: XPMHeader,
}
