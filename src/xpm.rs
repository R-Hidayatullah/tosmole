//! XPM (Progressive Morph Motion) file format definitions.
//!
//! This module provides type definitions and data structures for handling
//! Progressive Morph Motion files (.xpm), which contain facial animation
//! and morph target data with phoneme sets for speech animation.

use binrw::{BinRead, BinReaderExt, BinResult, binread};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;

/// XPM-specific chunk identifiers
pub enum XPMChunk {
    SUBMOTION = 100,
    INFO = 101,
    MotionEventTable = 50,
    SUBMOTIONS = 102,
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

#[derive(Debug, Serialize, Deserialize)]
pub enum XPMChunkData {
    XPMInfo(XPMInfo),
    XPMSubMotions(XPMSubMotions),
}

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
    padding: u16,

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
    padding: u16,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct XPMChunkEntry {
    pub chunk: FileChunk,
    pub chunk_data: XPMChunkData,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct XPMRoot {
    pub header: XPMHeader,
    pub chunks: Vec<XPMChunkEntry>,
}

impl XPMRoot {
    /// Read XPMRoot from a file path, accepting &str or &Path
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path_ref = path.as_ref();
        let file = File::open(path_ref)?;
        let mut reader = BufReader::new(file);
        let root = XPMRoot {
            header: reader
                .read_le()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("binrw error: {}", e)))?,
            chunks: Self::read_chunks(&mut reader)?,
        };

        Ok(root)
    }

    /// Read XPMRoot from a byte slice in memory
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);
        let root = XPMRoot {
            header: cursor
                .read_le()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("binrw error: {}", e)))?,
            chunks: Self::read_chunks(&mut cursor)?,
        };

        Ok(root)
    }

    fn read_chunks<R: Read + Seek>(reader: &mut R) -> io::Result<Vec<XPMChunkEntry>> {
        let mut chunks = Vec::new();

        while let Ok(chunk) = FileChunk::read(reader) {
            let start_pos = reader.seek(SeekFrom::Current(0))?;

            // Attempt to parse directly from the reader
            let chunk_data = match Self::parse_chunk_data(&chunk, reader) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Failed to parse chunk {}: {:?}", chunk.chunk_id, e);

                    // Fallback: skip chunk based on size_in_bytes
                    if let Err(seek_err) =
                        reader.seek(SeekFrom::Start(start_pos + chunk.size_in_bytes as u64))
                    {
                        eprintln!("Failed to skip chunk {}: {:?}", chunk.chunk_id, seek_err);
                    }

                    continue;
                }
            };

            // Ensure the reader is at least at the end of the chunk according to size_in_bytes
            let fallback_end = start_pos + chunk.size_in_bytes as u64;
            let current_pos = reader.seek(SeekFrom::Current(0))?;
            if current_pos < fallback_end {
                reader.seek(SeekFrom::Start(fallback_end))?;
            }

            chunks.push(XPMChunkEntry { chunk, chunk_data });
        }

        Ok(chunks)
    }

    fn parse_chunk_data<R: Read + Seek>(
        chunk: &FileChunk,
        reader: &mut R,
    ) -> Result<XPMChunkData, binrw::Error> {
        match chunk.chunk_id {
            x if x == XPMChunk::INFO as u32 => Ok(XPMChunkData::XPMInfo(reader.read_le()?)),

            x if x == XPMChunk::SUBMOTIONS as u32 => {
                Ok(XPMChunkData::XPMSubMotions(reader.read_le()?))
            }

            _ => Self::unsupported(chunk, reader),
        }
    }

    /// helper for unsupported chunk/version
    fn unsupported<R: Read + Seek>(
        chunk: &FileChunk,
        reader: &mut R,
    ) -> Result<XPMChunkData, binrw::Error> {
        let pos = reader.seek(SeekFrom::Current(0)).unwrap_or(0); // current position, fallback to 0 if error

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
    use std::io;

    #[test]
    fn test_read_xpm_root() -> io::Result<()> {
        // Path to your test IES file
        let path = "tests/npc_diana_head_std.xpm";

        // Read XPMRoot from file
        let root = XPMRoot::from_file(path)?;

        // Print for debugging (optional)
        println!("{:#?}", root.header);
        for (i, entry) in root.chunks.iter().enumerate() {
            println!(
                "Chunk {}: id={}, version={}, size={}",
                i, entry.chunk.chunk_id, entry.chunk.version, entry.chunk.size_in_bytes
            );
        }
        Ok(())
    }

    #[test]
    fn test_read_xpm_from_memory_stats() -> io::Result<()> {
        let data = std::fs::read("tests/GM1_hair_skl_summon_demon2.xpm")?;
        let root = XPMRoot::from_bytes(&data)?;

        println!("{:#?}", root.header);
        for (i, entry) in root.chunks.iter().enumerate() {
            println!(
                "Chunk {}: id={}, version={}, size={}",
                i, entry.chunk.chunk_id, entry.chunk.version, entry.chunk.size_in_bytes
            );
        }
        Ok(())
    }
}
