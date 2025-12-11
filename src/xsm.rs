//! XSM (Skeletal Motion) file format definitions.
//!
//! This module provides type definitions and data structures for handling
//! Skeletal Motion files (.xsm), which contain skeletal animation data
//! with support for wavelet compression.

use binrw::{BinRead, BinReaderExt, binread};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufReader, Cursor, Read, Seek, SeekFrom};
use std::path::Path;

/// XSM-specific chunk identifiers
pub enum XSMChunk {
    Submotion = 200,
    Info = 201,
    MotionEventTable = 50, // SHARED_CHUNK_MOTIONEVENTTABLE
    Submotions = 202,
    WaveletInfo = 203,
}

/// Wavelet types used during compression
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum WaveletType {
    Haar = 0,
    D4 = 1,
    CDF97 = 2,
}

/// Compressor types for quantized data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CompressorType {
    Huffman = 0,
    Rice = 1,
}

/// File chunk header
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct FileChunk {
    pub chunk_id: u32,
    pub size_in_bytes: u32,
    pub version: u32,
}

/// XSM file format header
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XSMHeader {
    /// File format identifier, must be b"XSM "
    pub fourcc: [u8; 4],
    /// High version number (e.g., 2 in version 2.34)
    pub hi_version: u8,
    /// Low version number (e.g., 34 in version 2.34)
    pub lo_version: u8,
    /// Endianness of the data (0 = little endian, 1 = big endian)
    pub endian_type: u8,
    /// Matrix multiplication order
    pub mul_order: u8,
}

/// 3D Vector (from shared_formats.h)
#[binread]
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
#[br(little)]
pub struct FileVector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Quaternion (from shared_formats.h)
#[binread]
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
#[br(little)]
pub struct FileQuaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

/// 16-bit compressed quaternion (from shared_formats.h)
#[binread]
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
#[br(little)]
pub struct File16BitQuaternion {
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub w: i16,
}

/// XSM Info chunk (version 1)
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XSMInfo {
    pub motion_fps: u32,
    pub exporter_high_version: u8,
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

/// XSM Info chunk (version 2)
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XSMInfo2 {
    pub importance_factor: f32,
    pub max_acceptable_error: f32,
    pub motion_fps: u32,
    pub exporter_high_version: u8,
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

/// XSM Info chunk (version 3 - adds motion extraction mask)
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XSMInfo3 {
    pub importance_factor: f32,
    pub max_acceptable_error: f32,
    pub motion_fps: u32,
    pub motion_extraction_mask: u32,
    pub exporter_high_version: u8,
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

/// 3D Vector keyframe
#[binread]
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[br(little)]
pub struct XSMVector3Key {
    pub value: FileVector3,
    pub time: f32,
}

/// Quaternion keyframe
#[binread]
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[br(little)]
pub struct XSMQuaternionKey {
    pub value: FileQuaternion,
    pub time: f32,
}

/// 16-bit compressed quaternion keyframe
#[binread]
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[br(little)]
pub struct XSM16BitQuaternionKey {
    pub value: File16BitQuaternion,
    pub time: f32,
}

/// Skeletal submotion (version 1)
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XSMSkeletalSubMotion {
    pub pose_rot: FileQuaternion,
    pub bind_pose_rot: FileQuaternion,
    pub pose_scale_rot: FileQuaternion,
    pub bind_pose_scale_rot: FileQuaternion,
    pub pose_pos: FileVector3,
    pub pose_scale: FileVector3,
    pub bind_pose_pos: FileVector3,
    pub bind_pose_scale: FileVector3,
    pub num_pos_keys: u32,
    pub num_rot_keys: u32,
    pub num_scale_keys: u32,
    pub num_scale_rot_keys: u32,

    #[br(temp)]
    name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub name: String,

    #[br(count = num_pos_keys)]
    pub pos_keys: Vec<XSMVector3Key>,
    #[br(count = num_rot_keys)]
    pub rot_keys: Vec<XSMQuaternionKey>,
    #[br(count = num_scale_keys)]
    pub scale_keys: Vec<XSMVector3Key>,
    #[br(count = num_scale_rot_keys)]
    pub scale_rot_keys: Vec<XSMQuaternionKey>,
}

/// Skeletal submotion (version 2 - adds max_error)
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XSMSkeletalSubMotion2 {
    pub pose_rot: FileQuaternion,
    pub bind_pose_rot: FileQuaternion,
    pub pose_scale_rot: FileQuaternion,
    pub bind_pose_scale_rot: FileQuaternion,
    pub pose_pos: FileVector3,
    pub pose_scale: FileVector3,
    pub bind_pose_pos: FileVector3,
    pub bind_pose_scale: FileVector3,
    pub num_pos_keys: u32,
    pub num_rot_keys: u32,
    pub num_scale_keys: u32,
    pub num_scale_rot_keys: u32,
    pub max_error: f32,

    #[br(temp)]
    name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub name: String,

    #[br(count = num_pos_keys)]
    pub pos_keys: Vec<XSMVector3Key>,
    #[br(count = num_rot_keys)]
    pub rot_keys: Vec<XSMQuaternionKey>,
    #[br(count = num_scale_keys)]
    pub scale_keys: Vec<XSMVector3Key>,
    #[br(count = num_scale_rot_keys)]
    pub scale_rot_keys: Vec<XSMQuaternionKey>,
}

/// Skeletal submotion (version 3 - uses 16-bit quaternions)
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XSMSkeletalSubMotion3 {
    pub pose_rot: File16BitQuaternion,
    pub bind_pose_rot: File16BitQuaternion,
    pub pose_scale_rot: File16BitQuaternion,
    pub bind_pose_scale_rot: File16BitQuaternion,
    pub pose_pos: FileVector3,
    pub pose_scale: FileVector3,
    pub bind_pose_pos: FileVector3,
    pub bind_pose_scale: FileVector3,
    pub num_pos_keys: u32,
    pub num_rot_keys: u32,
    pub num_scale_keys: u32,
    pub num_scale_rot_keys: u32,
    pub max_error: f32,

    #[br(temp)]
    name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub name: String,

    #[br(count = num_pos_keys)]
    pub pos_keys: Vec<XSMVector3Key>,
    #[br(count = num_rot_keys)]
    pub rot_keys: Vec<XSM16BitQuaternionKey>,
    #[br(count = num_scale_keys)]
    pub scale_keys: Vec<XSMVector3Key>,
    #[br(count = num_scale_rot_keys)]
    pub scale_rot_keys: Vec<XSM16BitQuaternionKey>,
}

/// Container for submotions (version 1)
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XSMSubMotions {
    pub num_sub_motions: u32,
    #[br(count = num_sub_motions)]
    pub sub_motions: Vec<XSMSkeletalSubMotion2>,
}

/// Container for submotions (version 2)
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XSMSubMotions2 {
    pub num_sub_motions: u32,
    #[br(count = num_sub_motions)]
    pub sub_motions: Vec<XSMSkeletalSubMotion3>,
}

/// Wavelet submotion mapping entry
#[binread]
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
#[br(little)]
pub struct XSMWaveletMapping {
    pub pos_index: u16,
    pub rot_index: u16,
    pub scale_rot_index: u16,
    pub scale_index: u16,
}

/// Wavelet skeletal submotion
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XSMWaveletSkeletalSubMotion {
    pub pose_rot: File16BitQuaternion,
    pub bind_pose_rot: File16BitQuaternion,
    pub pose_scale_rot: File16BitQuaternion,
    pub bind_pose_scale_rot: File16BitQuaternion,
    pub pose_pos: FileVector3,
    pub pose_scale: FileVector3,
    pub bind_pose_pos: FileVector3,
    pub bind_pose_scale: FileVector3,
    pub max_error: f32,

    #[br(temp)]
    name_length: u32,
    #[br(count = name_length, map = |s: Vec<u8>| String::from_utf8_lossy(&s).to_string())]
    pub name: String,
}

/// Wavelet compressed chunk
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XSMWaveletChunk {
    pub rot_quant_scale: f32,
    pub pos_quant_scale: f32,
    pub scale_quant_scale: f32,
    pub start_time: f32,
    pub compressed_rot_num_bytes: u32,
    pub compressed_pos_num_bytes: u32,
    pub compressed_scale_num_bytes: u32,
    pub compressed_pos_num_bits: u32,
    pub compressed_rot_num_bits: u32,
    pub compressed_scale_num_bits: u32,

    #[br(count = compressed_rot_num_bytes)]
    pub compressed_rot_data: Vec<u8>,
    #[br(count = compressed_pos_num_bytes)]
    pub compressed_pos_data: Vec<u8>,
    #[br(count = compressed_scale_num_bytes)]
    pub compressed_scale_data: Vec<u8>,
}

/// Wavelet skeletal submotions header
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct XSMWaveletInfo {
    pub num_chunks: u32,
    pub samples_per_chunk: u32,
    pub decompressed_rot_num_bytes: u32,
    pub decompressed_pos_num_bytes: u32,
    pub decompressed_scale_num_bytes: u32,
    pub num_rot_tracks: u32,
    pub num_scale_rot_tracks: u32,
    pub num_scale_tracks: u32,
    pub num_pos_tracks: u32,
    pub chunk_overhead: u32,
    pub compressed_size: u32,
    pub optimized_size: u32,
    pub uncompressed_size: u32,
    pub scale_rot_offset: u32,
    pub num_sub_motions: u32,
    pub pos_quant_factor: f32,
    pub rot_quant_factor: f32,
    pub scale_quant_factor: f32,
    pub sample_spacing: f32,
    pub seconds_per_chunk: f32,
    pub max_time: f32,
    pub wavelet_id: u8,
    pub compressor_id: u8,
    padding: [u8; 2],

    #[br(count = num_sub_motions)]
    pub mappings: Vec<XSMWaveletMapping>,
    #[br(count = num_sub_motions)]
    pub sub_motions: Vec<XSMWaveletSkeletalSubMotion>,
    #[br(count = num_chunks)]
    pub chunks: Vec<XSMWaveletChunk>,
}

/// Chunk data variants
#[derive(Debug, Serialize, Deserialize)]
pub enum XSMChunkData {
    Info(XSMInfo),
    Info2(XSMInfo2),
    Info3(XSMInfo3),
    SubMotions(XSMSubMotions),
    SubMotions2(XSMSubMotions2),
    WaveletInfo(XSMWaveletInfo),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct XSMChunkEntry {
    pub chunk: FileChunk,
    pub chunk_data: XSMChunkData,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct XSMRoot {
    pub header: XSMHeader,
    pub chunks: Vec<XSMChunkEntry>,
}

impl XSMRoot {
    /// Read XSMRoot from a file path
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let root = XSMRoot {
            header: reader
                .read_le()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("binrw error: {}", e)))?,
            chunks: Self::read_chunks(&mut reader)?,
        };

        Ok(root)
    }

    /// Read XSMRoot from a byte slice in memory
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let root = XSMRoot {
            header: cursor
                .read_le()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("binrw error: {}", e)))?,
            chunks: Self::read_chunks(&mut cursor)?,
        };

        Ok(root)
    }

    fn read_chunks<R: Read + Seek>(reader: &mut R) -> io::Result<Vec<XSMChunkEntry>> {
        let mut chunks = Vec::new();

        while let Ok(chunk) = FileChunk::read(reader) {
            let start_pos = reader.seek(SeekFrom::Current(0))?;

            let chunk_data = match Self::parse_chunk_data(&chunk, reader) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Failed to parse chunk {}: {:?}", chunk.chunk_id, e);
                    if let Err(seek_err) =
                        reader.seek(SeekFrom::Start(start_pos + chunk.size_in_bytes as u64))
                    {
                        eprintln!("Failed to skip chunk {}: {:?}", chunk.chunk_id, seek_err);
                    }
                    continue;
                }
            };

            let fallback_end = start_pos + chunk.size_in_bytes as u64;
            let current_pos = reader.seek(SeekFrom::Current(0))?;
            if current_pos < fallback_end {
                reader.seek(SeekFrom::Start(fallback_end))?;
            }

            chunks.push(XSMChunkEntry { chunk, chunk_data });
        }

        Ok(chunks)
    }

    fn parse_chunk_data<R: Read + Seek>(
        chunk: &FileChunk,
        reader: &mut R,
    ) -> Result<XSMChunkData, binrw::Error> {
        match chunk.chunk_id {
            x if x == XSMChunk::Info as u32 => match chunk.version {
                1 => Ok(XSMChunkData::Info(reader.read_le()?)),
                2 => Ok(XSMChunkData::Info2(reader.read_le()?)),
                3 => Ok(XSMChunkData::Info3(reader.read_le()?)),
                _ => Self::unsupported(chunk, reader),
            },

            x if x == XSMChunk::Submotions as u32 => match chunk.version {
                1 => Ok(XSMChunkData::SubMotions(reader.read_le()?)),
                2 => Ok(XSMChunkData::SubMotions2(reader.read_le()?)),
                _ => Self::unsupported(chunk, reader),
            },

            x if x == XSMChunk::WaveletInfo as u32 => {
                Ok(XSMChunkData::WaveletInfo(reader.read_le()?))
            }

            _ => Self::unsupported(chunk, reader),
        }
    }

    fn unsupported<R: Read + Seek>(
        chunk: &FileChunk,
        reader: &mut R,
    ) -> Result<XSMChunkData, binrw::Error> {
        let pos = reader.seek(SeekFrom::Current(0)).unwrap_or(0);

        Err(binrw::Error::AssertFail {
            pos,
            message: format!(
                "Unknown or unsupported chunk_id {} with version {}",
                chunk.chunk_id, chunk.version
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_xsm_root() -> io::Result<()> {
        let path = "tests/npc_lecifer_run.xsm";
        let root = XSMRoot::from_file(path)?;
        println!("{:?}", root.header);
        for (i, entry) in root.chunks.iter().enumerate() {
            println!(
                "Chunk {}: id={}, version={}, size={}",
                i, entry.chunk.chunk_id, entry.chunk.version, entry.chunk.size_in_bytes
            );
        }
        Ok(())
    }

    #[test]
    fn test_read_xsm_from_memory() -> io::Result<()> {
        let data = std::fs::read("tests/npc_Catapult_wlk.xsm")?;
        let root = XSMRoot::from_bytes(&data)?;
        println!("{:?}", root.header);
        for (i, entry) in root.chunks.iter().enumerate() {
            println!(
                "Chunk {}: id={}, version={}, size={}",
                i, entry.chunk.chunk_id, entry.chunk.version, entry.chunk.size_in_bytes
            );
        }
        Ok(())
    }
}
