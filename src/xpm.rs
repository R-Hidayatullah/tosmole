//! XPM (Progressive Morph Motion) file format definitions.
//!
//! This module provides type definitions and data structures for handling
//! Progressive Morph Motion files (.xpm), which contain facial animation
//! and morph target data with phoneme sets for speech animation.

use crate::shared_formats::{MultiplicationOrder, chunk_ids};

/// XPM-specific chunk identifiers
pub mod xpm_chunk_ids {
    use crate::shared_formats::chunk_ids;

    pub const SUBMOTION: u32 = 100;
    pub const INFO: u32 = 101;
    pub const MOTION_EVENT_TABLE: u32 = chunk_ids::MOTION_EVENT_TABLE;
    pub const SUBMOTIONS: u32 = 102;
}

/// XPM file format header
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XpmHeader {
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

impl XpmHeader {
    /// Standard XPM fourcc identifier
    pub const FOURCC: [u8; 4] = *b"XPM ";

    /// Creates a new XPM header with default values
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

    /// Gets the matrix multiplication order
    pub fn multiplication_order(&self) -> Option<MultiplicationOrder> {
        match self.mul_order {
            0 => Some(MultiplicationOrder::ScaleRotTrans),
            1 => Some(MultiplicationOrder::RotScaleTrans),
            _ => None,
        }
    }
}

/// XPM file information chunk
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XpmInfo {
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

impl XpmInfo {
    /// Creates a new XPM info structure
    pub fn new(motion_fps: u32, exporter_high_version: u8, exporter_low_version: u8) -> Self {
        Self {
            motion_fps,
            exporter_high_version,
            exporter_low_version,
            padding: [0; 2],
        }
    }

    /// Gets the exporter version as a tuple (major, minor)
    pub fn exporter_version(&self) -> (u8, u8) {
        (self.exporter_high_version, self.exporter_low_version)
    }
}

/// Progressive morph sub-motion data
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XpmProgressiveSubMotion {
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
    // Note: In the actual file format, this is followed by:
    // - String: name (the name of this motion part)
    // - XpmUnsignedShortKey[num_keys]
}

impl XpmProgressiveSubMotion {
    /// Creates a new progressive sub-motion
    pub fn new(
        pose_weight: f32,
        min_weight: f32,
        max_weight: f32,
        phoneme_set: u32,
        num_keys: u32,
    ) -> Self {
        Self {
            pose_weight,
            min_weight,
            max_weight,
            phoneme_set,
            num_keys,
        }
    }

    /// Checks if this is a phoneme-based sub-motion
    pub fn is_phoneme_motion(&self) -> bool {
        self.phoneme_set != 0
    }

    /// Checks if this is a normal morph target sub-motion
    pub fn is_morph_target(&self) -> bool {
        self.phoneme_set == 0
    }

    /// Gets the weight range for this sub-motion
    pub fn weight_range(&self) -> (f32, f32) {
        (self.min_weight, self.max_weight)
    }
}

/// Floating-point keyframe data
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct XpmFloatKey {
    /// Time in seconds
    pub time: f32,
    /// Keyframe value
    pub value: f32,
}

impl XpmFloatKey {
    /// Creates a new float key
    pub fn new(time: f32, value: f32) -> Self {
        Self { time, value }
    }
}

impl PartialOrd for XpmFloatKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

/// Compressed 16-bit unsigned integer keyframe data
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct XpmUnsignedShortKey {
    /// Time in seconds
    pub time: f32,
    /// Compressed keyframe value (16-bit unsigned integer)
    pub value: u16,
    padding: [u8; 2],
}

impl XpmUnsignedShortKey {
    /// Creates a new unsigned short key
    pub fn new(time: f32, value: u16) -> Self {
        Self {
            time,
            value,
            padding: [0; 2],
        }
    }

    /// Unpacks the compressed value to a float using the given range
    pub fn unpack_value(&self, min_weight: f32, max_weight: f32) -> f32 {
        let normalized = self.value as f32 / u16::MAX as f32;
        min_weight + normalized * (max_weight - min_weight)
    }

    /// Packs a float value into the compressed format using the given range
    pub fn pack_value(time: f32, value: f32, min_weight: f32, max_weight: f32) -> Self {
        let normalized = if max_weight != min_weight {
            ((value - min_weight) / (max_weight - min_weight)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let packed_value = (normalized * u16::MAX as f32).round() as u16;
        Self::new(time, packed_value)
    }
}

impl PartialOrd for XpmUnsignedShortKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

/// Container for multiple sub-motions
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XpmSubMotions {
    /// Number of sub-motions in this container
    pub num_sub_motions: u32,
    // Note: In the actual file format, this is followed by:
    // - XpmProgressiveSubMotion[num_sub_motions]
}

impl XpmSubMotions {
    /// Creates a new sub-motions container
    pub fn new(num_sub_motions: u32) -> Self {
        Self { num_sub_motions }
    }
}

/// Endianness type for XPM files
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    /// Little endian byte order
    Little = 0,
    /// Big endian byte order
    Big = 1,
}

impl From<u8> for Endianness {
    fn from(value: u8) -> Self {
        match value {
            0 => Endianness::Little,
            _ => Endianness::Big,
        }
    }
}

impl From<Endianness> for u8 {
    fn from(endianness: Endianness) -> Self {
        endianness as u8
    }
}

/// XPM file validation and utility functions
pub mod utils {
    use super::*;

    /// Validates an XPM header
    pub fn validate_header(header: &XpmHeader) -> Result<(), &'static str> {
        if !header.is_valid_fourcc() {
            return Err("Invalid XPM fourcc identifier");
        }

        if header.multiplication_order().is_none() {
            return Err("Invalid matrix multiplication order");
        }

        Ok(())
    }

    /// Calculates the unpacked size for a compressed key sequence
    pub fn calculate_unpacked_size(num_keys: u32) -> usize {
        num_keys as usize * std::mem::size_of::<XpmFloatKey>()
    }

    /// Calculates the packed size for a compressed key sequence
    pub fn calculate_packed_size(num_keys: u32) -> usize {
        num_keys as usize * std::mem::size_of::<XpmUnsignedShortKey>()
    }
}

// Type aliases for convenience
pub type FloatKey = XpmFloatKey;
pub type UShortKey = XpmUnsignedShortKey;
pub type ProgressiveSubMotion = XpmProgressiveSubMotion;
pub type SubMotions = XpmSubMotions;
pub type Info = XpmInfo;
pub type Header = XpmHeader;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_creation() {
        let header = XpmHeader::new(2, 34);
        assert_eq!(header.fourcc, *b"XPM ");
        assert_eq!(header.version(), (2, 34));
        assert!(header.is_valid_fourcc());
        assert!(header.is_little_endian());
    }

    #[test]
    fn test_float_key_ordering() {
        let key1 = XpmFloatKey::new(1.0, 0.5);
        let key2 = XpmFloatKey::new(2.0, 0.8);
        assert!(key1 < key2);
    }

    #[test]
    fn test_progressive_submotion_types() {
        let morph_target = XpmProgressiveSubMotion::new(1.0, 0.0, 1.0, 0, 10);
        assert!(morph_target.is_morph_target());
        assert!(!morph_target.is_phoneme_motion());

        let phoneme_motion = XpmProgressiveSubMotion::new(1.0, 0.0, 1.0, 1, 10);
        assert!(!phoneme_motion.is_morph_target());
        assert!(phoneme_motion.is_phoneme_motion());
    }

    #[test]
    fn test_header_validation() {
        let valid_header = XpmHeader::new(2, 34);
        assert!(utils::validate_header(&valid_header).is_ok());

        let mut invalid_header = valid_header;
        invalid_header.fourcc = *b"XXXX";
        assert!(utils::validate_header(&invalid_header).is_err());
    }
}
