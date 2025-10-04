use binrw::{
    BinReaderExt, Endian, binread,
    io::{Read, Seek},
};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Cursor};

// MODE enum
#[binread]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[br(repr = u32)]
pub enum Mode {
    None = 0,
    PCM8 = 1,
    PCM16 = 2,
    PCM24 = 3,
    PCM32 = 4,
    PCMFLOAT = 5,
    GCADPCM = 6,
    IMAADPCM = 7,
    VAG = 8,
    HEVAG = 9,
    XMA = 10,
    MPEG = 11,
    CELT = 12,
    AT9 = 13,
    XWMA = 14,
    VORBIS = 15,
}

// CHUNK_TYPE enum
#[binread]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[br(repr = u32)]
pub enum ChunkType {
    Channels = 1,
    Frequency = 2,
    Loop = 3,
    XmaSeek = 6,
    DspCoeff = 7,
    XwmaData = 10,
    VorbisData = 11,
    UnknownData = 255,
}

// FSB5 file header
#[binread]
#[derive(Debug, Serialize, Deserialize)]
#[br(little)]
pub struct FSB5Header {
    pub id: [u8; 4],
    pub version: i32,
    pub num_samples: i32,
    pub sample_header_size: i32,
    pub name_table_size: i32,
    pub data_size: i32,
    pub mode: Mode,

    pub zero: [u8; 8],
    pub hash: [u8; 16],
    pub dummy: [u8; 8],

    #[br(if(version == 0))]
    pub unknown: Option<u32>,
}

// Bitfield sample header
#[binread]
#[derive(Debug, Serialize, Deserialize)]
#[br(little)]
pub struct SampleHeaderBitfield {
    #[br(temp)]
    raw_bitfield: u64,
    // read the whole 64 bits first
    #[br(calc=(raw_bitfield & 0x1) != 0)]
    pub extra_params: bool,
    #[br(calc=((raw_bitfield >> 1) & 0xF) as u8)]
    pub frequency: u8,
    // 1-9 -> map to Hz
    #[br(calc=((raw_bitfield >> 5) & 0x1) != 0)]
    pub two_channels: bool,
    #[br(calc = ((raw_bitfield >> 6) & 0x0FFF_FFFF) as u32)]
    pub data_offset: u32,
    // multiply by 16 to get actual offset
    #[br(calc = ((raw_bitfield >> 34) & 0x3FFF_FFFF) as u32)]
    pub samples: u32,
}

// Loop struct
#[binread]
#[derive(Debug, Serialize, Deserialize)]
#[br(little)]
pub struct Loop {
    pub loop_start: u32,
    pub loop_end: u32,
}

#[binread]
#[derive(Debug, Serialize, Deserialize)]
#[br(little)]
pub struct VorbisPacketData {
    pub offset: u32,

    #[br(parse_with = parse_granule_position)]
    pub granule_position: Option<u32>,
}

fn parse_granule_position<R: Read + Seek>(
    reader: &mut R,
    endian: Endian,
    _: (),
) -> binrw::BinResult<Option<u32>> {
    use binrw::io::Read;

    // Peek or determine remaining bytes to decide if granule_position exists
    // Here we try reading u32, if EOF, return None
    let mut buf = [0u8; 4];
    let pos = reader.stream_position()?;
    match reader.read(&mut buf)? {
        4 => Ok(Some(u32::from_le_bytes(buf))),
        0 => Ok(None),
        _ => {
            // Partial read -> restore position and return None
            reader.seek(std::io::SeekFrom::Start(pos))?;
            Ok(None)
        }
    }
}

// Vorbis chunk
#[derive(Debug, Serialize, Deserialize)]
pub struct VorbisChunk {
    pub crc32: u32,
    pub packets: Vec<VorbisPacketData>,
}

// Extra chunk
#[derive(Debug, Serialize, Deserialize)]
pub enum ExtraChunk {
    Channels(u8),
    Frequency(u32),
    Loop(Loop),
    XmaSeek(Vec<u8>),
    DspCoeff(Vec<u8>),
    XwmaData(Vec<u8>),
    VorbisData(VorbisChunk),
    Unknown(Vec<u8>),
}

// Full FSOUND_FSB_SAMPLE_HEADER
#[derive(Debug, Serialize, Deserialize)]
pub struct FSBSampleHeader {
    pub bitfield: SampleHeaderBitfield,
    pub extra_chunks: Vec<ExtraChunk>,
}

// Name table entry
#[derive(Debug, Serialize, Deserialize)]
pub struct NameTableEntry {
    pub name_start: u32,
    pub name: String,
}

// VorbisPacket struct
#[derive(Debug, Serialize, Deserialize)]
pub struct VorbisPacket {
    pub audio: bool,
    pub r: u8,
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SampleData {
    Vorbis(Vec<VorbisPacket>),
    Raw(Vec<u8>),
}

// Main FSB5 struct
#[derive(Debug, Serialize, Deserialize)]
pub struct FSB5File {
    pub header: FSB5Header,
    pub sample_headers: Vec<FSBSampleHeader>,
    pub name_table: Option<Vec<NameTableEntry>>,
    pub sample_data: Vec<SampleData>, // Each sample's raw data
}

// ---------------- Example parser ----------------
impl FSB5File {
    pub fn read<R: Read + Seek>(reader: &mut R) -> binrw::BinResult<Self> {
        use binrw::io::SeekFrom;

        let header: FSB5Header = reader.read_le()?;

        // Sample headers
        let mut sample_headers = Vec::new();
        for _ in 0..header.num_samples {
            let bitfield: SampleHeaderBitfield = reader.read_le()?;

            let mut extra_chunks = Vec::new();
            if bitfield.extra_params {
                let mut next = true;
                while next {
                    // Read the 32-bit chunk header
                    let raw_header: u32 = reader.read_le()?;

                    // Extract the fields
                    next = (raw_header & 0x1) != 0;
                    let size = (raw_header >> 1) & 0x00FF_FFFF; // 24 bits
                    let chunk_type = ((raw_header >> 25) & 0x7F) as u8;

                    // Convert chunk_type to enum
                    let chunk_type = match chunk_type {
                        1 => ChunkType::Channels,
                        2 => ChunkType::Frequency,
                        3 => ChunkType::Loop,
                        6 => ChunkType::XmaSeek,
                        7 => ChunkType::DspCoeff,
                        10 => ChunkType::XwmaData,
                        11 => ChunkType::VorbisData,
                        _ => ChunkType::UnknownData,
                    };

                    let chunk = match chunk_type {
                        ChunkType::Channels => {
                            assert_eq!(size, 1, "Channels chunk should be 1 byte");
                            let mut b = [0u8; 1];
                            reader.read_exact(&mut b)?;
                            ExtraChunk::Channels(b[0])
                        }
                        ChunkType::Frequency => {
                            assert_eq!(size, 4, "Frequency chunk should be 4 bytes");
                            let val: u32 = reader.read_le()?;
                            ExtraChunk::Frequency(val)
                        }
                        ChunkType::Loop => {
                            assert_eq!(size, 8, "Loop chunk should be 8 bytes");
                            let val: Loop = reader.read_le()?;
                            ExtraChunk::Loop(val)
                        }
                        ChunkType::XmaSeek => {
                            let mut buf = vec![0u8; size as usize];
                            reader.read_exact(&mut buf)?;
                            ExtraChunk::XmaSeek(buf)
                        }
                        ChunkType::DspCoeff => {
                            let mut buf = vec![0u8; size as usize];
                            reader.read_exact(&mut buf)?;
                            ExtraChunk::DspCoeff(buf)
                        }
                        ChunkType::XwmaData => {
                            let mut buf = vec![0u8; size as usize];
                            reader.read_exact(&mut buf)?;
                            ExtraChunk::XwmaData(buf)
                        }
                        ChunkType::VorbisData => {
                            let crc32: u32 = reader.read_le()?;
                            let mut packets = Vec::new();
                            let mut remain = size as i64 - 4;

                            while remain > 0 {
                                let offset: u32 = reader.read_le()?;
                                let granule_position = if remain > 4 {
                                    Some(reader.read_le()?)
                                } else {
                                    None
                                };

                                packets.push(VorbisPacketData {
                                    offset,
                                    granule_position,
                                });

                                // Always subtract 8, like the 010 template
                                remain -= 8;
                            }

                            ExtraChunk::VorbisData(VorbisChunk { crc32, packets })
                        }

                        ChunkType::UnknownData => {
                            let mut buf = vec![0u8; size as usize];
                            reader.read_exact(&mut buf)?;
                            ExtraChunk::Unknown(buf)
                        }
                        _ => {
                            let mut buf = vec![0u8; size as usize];
                            reader.read_exact(&mut buf)?;
                            ExtraChunk::Unknown(buf)
                        }
                    };

                    extra_chunks.push(chunk);
                }
            }

            sample_headers.push(FSBSampleHeader {
                bitfield,
                extra_chunks,
            });
        }

        // Name table
        let name_table = if header.name_table_size > 0 {
            let name_table_start = reader.stream_position()?;
            let mut name_start_vec = Vec::new();
            for _ in 0..header.num_samples {
                let start: u32 = reader.read_le()?;
                name_start_vec.push(start);
            }

            let mut names = Vec::new();
            for start in name_start_vec {
                reader.seek(SeekFrom::Start(name_table_start + start as u64))?;
                let mut buf = Vec::new();
                loop {
                    let mut byte = [0u8; 1];
                    reader.read_exact(&mut byte)?;
                    if byte[0] == 0 {
                        break;
                    }
                    buf.push(byte[0]);
                }
                let name = String::from_utf8_lossy(&buf).to_string();
                names.push(NameTableEntry {
                    name_start: start,
                    name,
                });
            }
            Some(names)
        } else {
            None
        };
        let current_pos = reader.stream_position()?; // current file pointer
        let padding_len =
            (60u64 + header.sample_header_size as u64 + header.name_table_size as u64)
                .saturating_sub(current_pos); // avoid underflow

        if padding_len > 0 {
            let mut _pad = vec![0u8; padding_len as usize];
            reader.read_exact(&mut _pad)?; // consume padding
        }

        let sample_data_start = reader.stream_position()?;

        // Sample data
        let mut sample_data: Vec<SampleData> = Vec::new();
        for index in 0..header.num_samples {
            let start = sample_data_start
                + (sample_headers[index as usize].bitfield.data_offset as u64 * 16);

            let mut end = sample_data_start + header.data_size as u64;
            if index + 1 < header.num_samples {
                end = sample_data_start
                    + (sample_headers[index as usize + 1].bitfield.data_offset as u64 * 16);
            }

            let size = (end - start) as usize;

            reader.seek(std::io::SeekFrom::Start(start))?;

            let sample = if header.mode == Mode::VORBIS {
                // Read VORBIS packets
                let mut remaining = size;
                let mut packets = Vec::new();

                while remaining > 0 {
                    // Read packet size (ushort = 2 bytes)
                    let packet_size: u16 = match reader.read_le() {
                        Ok(sz) => sz,
                        Err(_) => break,
                    };
                    if packet_size == 0 {
                        break;
                    }
                    remaining = remaining.saturating_sub(2);

                    // Read 1 byte: audio (bit 0) + r (bits 1..7)
                    let byte: u8 = reader.read_le()?;
                    remaining = remaining.saturating_sub(1);

                    let audio = (byte & 0x01) != 0; // bit0
                    let r = (byte >> 1) & 0x7F; // bits1..7

                    // Read the rest of the packet
                    let data_len = (packet_size as usize).saturating_sub(1);
                    let mut data = vec![0u8; data_len];
                    reader.read_exact(&mut data)?;
                    remaining = remaining.saturating_sub(data_len);

                    packets.push(VorbisPacket { audio, r, data });
                }

                SampleData::Vorbis(packets)
            } else {
                // Non-VORBIS: read raw bytes
                let mut buf = vec![0u8; size];
                reader.read_exact(&mut buf)?;
                SampleData::Raw(buf)
            };

            sample_data.push(sample);
        }

        Ok(Self {
            header,
            sample_headers,
            name_table,
            sample_data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;

    #[test]
    fn read_skilvoice_fsb5() {
        // Open file
        let f = File::open("tests/skilvoice_jap.fsb").expect("Failed to open skilvoice_jap.fsb");
        let mut reader = BufReader::new(f);

        // Parse FSB5
        let fsb = FSB5File::read(&mut reader).expect("Failed to parse FSB5 file");

        // --- Basic header checks ---
        println!("ID: {:?}", std::str::from_utf8(&fsb.header.id).unwrap());
        println!("Version: {}", fsb.header.version);
        println!("Num samples: {}", fsb.header.num_samples);
        println!("Num samples Actual: {}", fsb.sample_headers.len());

        println!("Mode: {:?}", fsb.header.mode);
        println!("Data size: {}", fsb.header.data_size);
        println!("Data Actual: {}", fsb.sample_data.len());

        // --- Sample headers ---
        for (i, sh) in fsb.sample_headers.iter().rev().enumerate() {
            println!("Sample {}:", i);
            println!("  Extra params: {}", sh.bitfield.extra_params);
            println!(
                "  Frequency: {} ({})",
                sh.bitfield.frequency,
                frequency_to_hz(sh.bitfield.frequency)
            );
            println!("  Two channels: {}", sh.bitfield.two_channels);
            println!("  Data offset: {}", sh.bitfield.data_offset * 16);
            println!("  Samples: {}", sh.bitfield.samples);

            for (j, chunk) in sh.extra_chunks.iter().enumerate() {
                println!("    Chunk {}: {:?}", j, chunk);
                break;
            }

            break;
        }

        // --- Name table ---
        if let Some(name_table) = &fsb.name_table {
            for entry in name_table.iter().rev() {
                println!("Name start: {}, Name: {}", entry.name_start, entry.name);
                break;
            }
        }

        // --- Sample data ---
        println!("Sample data:");
        for (i, sd) in fsb.sample_data.iter().enumerate() {
            if i == 1529 {
                match sd {
                    SampleData::Vorbis(packets) => {
                        for (j, p) in packets.iter().enumerate() {
                            println!(
                                "Sample {} Packet {} | audio: {} | r: {} | data length: {}",
                                i,
                                j,
                                p.audio,
                                p.r,
                                p.data.len()
                            );
                        }
                    }
                    SampleData::Raw(buf) => {
                        println!("Sample {} Raw data length: {}", i, buf.len());
                    }
                }
                break;
            }
        }

        // --- Assertions (optional) ---
        assert_eq!(&fsb.header.id, b"FSB5");
        assert_eq!(fsb.sample_headers.len() as i32, fsb.header.num_samples);
    }

    // Optional helper to map frequency codes to Hz
    fn frequency_to_hz(code: u8) -> &'static str {
        match code {
            1 => "8000Hz",
            2 => "11000Hz",
            3 => "11025Hz",
            4 => "16000Hz",
            5 => "22050Hz",
            6 => "24000Hz",
            7 => "32000Hz",
            8 => "44100Hz",
            9 => "48000Hz",
            _ => "Unknown",
        }
    }
}
