//! XPM (Progressive Morph Motion) file format definitions.
//!
//! This module provides type definitions and data structures for handling
//! Progressive Morph Motion files (.xpm), which contain facial animation
//! and morph target data with phoneme sets for speech animation.

use std::io::{self, Read, Seek};

use crate::{
    binary::BinaryReader,
    shared_formats::{FileChunk, MultiplicationOrder, chunk_ids},
};

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

impl XPMHeader {
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

/// XPM file information chunk
#[derive(Debug, Clone)]
#[repr(C)]
pub struct XPMInfo {
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
    pub source_app: String,
    pub original_filename: String,
    pub compilation_date: String,
    pub motion_name: String,
}

impl XPMInfo {
    /// Creates a new XPM info structure
    pub fn new(motion_fps: u32, exporter_high_version: u8, exporter_low_version: u8) -> Self {
        Self {
            motion_fps,
            exporter_high_version,
            exporter_low_version,
            padding: [0; 2],
            source_app: String::new(),
            original_filename: String::new(),
            compilation_date: String::new(),
            motion_name: String::new(),
        }
    }

    /// Gets the exporter version as a tuple (major, minor)
    pub fn exporter_version(&self) -> (u8, u8) {
        (self.exporter_high_version, self.exporter_low_version)
    }

    pub fn read_from<R: Read + Seek>(br: &mut BinaryReader<R>, size: u32) -> io::Result<Self> {
        let start_pos = br.position()?;

        let motion_fps = br.read_u32()?;
        let exporter_high_version = br.read_u8()?;
        let exporter_low_version = br.read_u8()?;
        let padding = br.read_exact::<2>()?;
        let motion_fps = br.read_u32()?;

        let source_app = br.read_string_u32()?;
        let original_filename = br.read_string_u32()?;
        let compilation_date = br.read_string_u32()?;
        let motion_name = br.read_string_u32()?;

        let end_pos = br.position()?;
        let parsed_bytes = (end_pos - start_pos) as u32;

        if parsed_bytes != size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "XPMInfo chunk size mismatch: expected {}, parsed {}",
                    size, parsed_bytes
                ),
            ));
        }

        Ok(Self {
            motion_fps,
            exporter_high_version,
            exporter_low_version,
            padding,
            source_app,
            original_filename,
            compilation_date,
            motion_name,
        })
    }
}

/// Progressive morph sub-motion data
#[derive(Debug, Clone)]
#[repr(C)]
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
    // Note: In the actual file format, this is followed by:
    // - String: name (the name of this motion part)
    // - XPMUnsignedShortKey[num_keys]
    pub name: String,
    pub xpm_key: Vec<XPMUnsignedShortKey>,
}

impl XPMProgressiveSubMotion {
    /// Creates a new progressive sub-motion
    pub fn new(
        pose_weight: f32,
        min_weight: f32,
        max_weight: f32,
        phoneme_set: u32,
        num_keys: u32,
        name: String,
        xpm_key: Vec<XPMUnsignedShortKey>,
    ) -> Self {
        Self {
            pose_weight,
            min_weight,
            max_weight,
            phoneme_set,
            num_keys,
            name,
            xpm_key,
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

    pub fn read_from<R: Read + Seek>(br: &mut BinaryReader<R>) -> io::Result<Self> {
        let pose_weight = br.read_f32()?;
        let min_weight = br.read_f32()?;
        let max_weight = br.read_f32()?;
        let phoneme_set = br.read_u32()?;
        let num_keys = br.read_u32()?;
        let name = br.read_string_u32()?;
        let xpm_key = Vec::new();

        Ok(Self {
            pose_weight,
            min_weight,
            max_weight,
            phoneme_set,
            num_keys,
            name,
            xpm_key,
        })
    }
}

/// Floating-point keyframe data
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct XPMFloatKey {
    /// Time in seconds
    pub time: f32,
    /// Keyframe value
    pub value: f32,
}

impl XPMFloatKey {
    /// Creates a new float key
    pub fn new(time: f32, value: f32) -> Self {
        Self { time, value }
    }
}

impl PartialOrd for XPMFloatKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

/// Compressed 16-bit unsigned integer keyframe data
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct XPMUnsignedShortKey {
    /// Time in seconds
    pub time: f32,
    /// Compressed keyframe value (16-bit unsigned integer)
    pub value: u16,
    padding: [u8; 2],
}

impl XPMUnsignedShortKey {
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

impl PartialOrd for XPMUnsignedShortKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

/// Container for multiple sub-motions
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct XPMSubMotions {
    /// Number of sub-motions in this container
    pub num_sub_motions: u32,
    // Note: In the actual file format, this is followed by:
    // - XPMProgressiveSubMotion[num_sub_motions]
}

impl XPMSubMotions {
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
    pub fn validate_header(header: &XPMHeader) -> Result<(), &'static str> {
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
        num_keys as usize * std::mem::size_of::<XPMFloatKey>()
    }

    /// Calculates the packed size for a compressed key sequence
    pub fn calculate_packed_size(num_keys: u32) -> usize {
        num_keys as usize * std::mem::size_of::<XPMUnsignedShortKey>()
    }
}

// Type aliases for convenience
pub type FloatKey = XPMFloatKey;
pub type UShortKey = XPMUnsignedShortKey;
pub type ProgressiveSubMotion = XPMProgressiveSubMotion;
pub type SubMotions = XPMSubMotions;
pub type Info = XPMInfo;
pub type Header = XPMHeader;

#[derive(Debug)]
pub enum XPMChunk {
    Unknown(FileChunk, Vec<u8>), // raw data
    Info(FileChunk, XPMInfo),
}

#[derive(Debug)]
pub struct XPMRoot {
    pub header: XPMHeader,
    pub xpm_data: Vec<XPMChunk>, // store parsed chunks here
}

impl XPMRoot {
    pub fn read_from<R: Read + Seek>(br: &mut BinaryReader<R>) -> io::Result<Self> {
        let header = XPMHeader::read_from(br)?;
        let mut xpm_data = Vec::new();

        while let Ok(chunk_header) = FileChunk::read_from(br) {
            // Deduce type from chunk_id + version
            let chunk = match (chunk_header.chunk_id, chunk_header.version) {
                (xpm_chunk_ids::INFO, 1) => {
                    let info = XPMInfo::read_from(br, chunk_header.size_in_bytes)?;
                    XPMChunk::Info(chunk_header, info)
                }

                _ => XPMChunk::Unknown(
                    chunk_header,
                    br.read_vec(chunk_header.size_in_bytes as usize)?,
                ),
            };
            xpm_data.push(chunk);
        }

        Ok(Self { header, xpm_data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_creation() {
        let header = XPMHeader::new(2, 34);
        assert_eq!(header.fourcc, *b"XPM ");
        assert_eq!(header.version(), (2, 34));
        assert!(header.is_valid_fourcc());
        assert!(header.is_little_endian());
    }

    #[test]
    fn test_float_key_ordering() {
        let key1 = XPMFloatKey::new(1.0, 0.5);
        let key2 = XPMFloatKey::new(2.0, 0.8);
        assert!(key1 < key2);
    }

    #[test]
    fn test_progressive_submotion_types() {
        let morph_target =
            XPMProgressiveSubMotion::new(1.0, 0.0, 1.0, 0, 10, "".to_string(), Vec::new());
        assert!(morph_target.is_morph_target());
        assert!(!morph_target.is_phoneme_motion());

        let phoneme_motion =
            XPMProgressiveSubMotion::new(1.0, 0.0, 1.0, 1, 10, "".to_string(), Vec::new());
        assert!(!phoneme_motion.is_morph_target());
        assert!(phoneme_motion.is_phoneme_motion());
    }

    #[test]
    fn test_header_validation() {
        let valid_header = XPMHeader::new(2, 34);
        assert!(utils::validate_header(&valid_header).is_ok());

        let mut invalid_header = valid_header;
        invalid_header.fourcc = *b"XXXX";
        assert!(utils::validate_header(&invalid_header).is_err());
    }
}
