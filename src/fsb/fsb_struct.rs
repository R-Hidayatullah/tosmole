use serde::{Deserialize, Serialize};

use crate::fsb::fsb_enum::FsbMode;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FsbFile {
    pub(crate) header: FsbHeader,
    pub(crate) sample_header: Vec<FsbSampleHeader>,
    pub(crate) name_table: FsbNameTable,
    pub(crate) sample_data: Vec<FsbSampleData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FsbNameTable {
    pub(crate) name_start: Vec<u32>,
    pub(crate) name: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FsbHeader {
    pub(crate) magic: String,
    pub(crate) minor_version: u32,
    pub(crate) num_files: u32,
    pub(crate) sample_header_size: u32,
    pub(crate) name_table_size: u32,
    pub(crate) sample_data_compressed_size: u32,
    pub(crate) sample_format: FsbMode,
    pub(crate) zero: Vec<u8>,  // 8 bytes
    pub(crate) hash: Vec<u8>,  // 16 bytes
    pub(crate) dummy: Vec<u8>, // 8 bytes
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FsbSampleHeader {
    pub(crate) extra_params: u32,
    pub(crate) frequency: u32,
    pub(crate) two_channels: u32,
    pub(crate) data_offset: u32,
    pub(crate) samples: u32,
    pub(crate) chunk: FsbChunkType,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FsbChunkType {
    pub(crate) next: u32,
    pub(crate) length: u32,
    pub(crate) mode: FsbMode,
    pub(crate) vorbis: Vorbis,
    pub(crate) loop_data: Loop,
}
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Vorbis {
    pub(crate) crc32: u32,
    pub(crate) packet_data: Vec<PacketData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Loop {
    pub(crate) loop_start: u32,
    pub(crate) loop_end: u32,
}
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PacketData {
    pub(crate) offset: u32,
    pub(crate) granule_position: Option<u32>,
}
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct FsbSampleData {
    pub(crate) length: u16,
    pub(crate) audio: u8,
    pub(crate) r: u8,
    pub(crate) data: Vec<u8>,
}
