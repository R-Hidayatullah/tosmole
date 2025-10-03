use binrw::{
    BinReaderExt, Endian, binread,
    io::{Read, Seek},
};
use hound::{SampleFormat, WavSpec, WavWriter};
use lewton::inside_ogg::OggStreamReader;
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
    raw_bitfield: u64, // read the whole 64 bits first

    #[br(calc=(raw_bitfield & 0x1) != 0)]
    pub extra_params: bool,
    #[br(calc=((raw_bitfield >> 1) & 0xF) as u8)]
    pub frequency: u8, // 1-9 -> map to Hz
    #[br(calc=((raw_bitfield >> 5) & 0x1) != 0)]
    pub two_channels: bool,
    #[br(calc = ((raw_bitfield >> 6) & 0x0FFF_FFFF) as u32)]
    pub data_offset: u32, // multiply by 16 to get actual offset
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
                        _ => {
                            // unknown type
                            // you can store as ExtraChunk::Unknown later
                            ChunkType::VorbisData // placeholder, or handle specially
                        }
                    };

                    let chunk = match chunk_type {
                        ChunkType::Channels => {
                            let val: u8 = reader.read_le()?;
                            ExtraChunk::Channels(val)
                        }
                        ChunkType::Frequency => {
                            let val: u32 = reader.read_le()?;
                            ExtraChunk::Frequency(val)
                        }
                        ChunkType::Loop => {
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
                                remain -= if granule_position.is_some() { 8 } else { 4 };
                            }
                            ExtraChunk::VorbisData(VorbisChunk { crc32, packets })
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

                    // Read 1 byte: audio (1 bit) + r (7 bits)
                    let byte: u8 = reader.read_le()?;
                    remaining = remaining.saturating_sub(1);

                    let audio = (byte & 0x80) != 0;
                    let _r = byte & 0x7F;

                    // Read the rest of the packet
                    let data_len = (packet_size as usize).saturating_sub(1);
                    let mut data = vec![0u8; data_len];
                    reader.read_exact(&mut data)?;
                    remaining = remaining.saturating_sub(data_len);

                    packets.push(VorbisPacket { audio, data });
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

/// Export a raw PCM sample as WAV
fn export_pcm_sample(
    filename: &str,
    data: &[u8],
    channels: u16,
    sample_rate: u32,
) -> std::io::Result<()> {
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16, // assuming PCM16 for most FSB5
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(filename, spec).unwrap();

    for chunk in data.chunks_exact(2) {
        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
        writer.write_sample(sample).unwrap();
    }

    writer.finalize().unwrap();
    Ok(())
}

/// Export Vorbis sample as WAV using lewton
fn export_vorbis_sample(
    filename: &str,
    packets: &[crate::fsb::VorbisPacket],
    channels: u16,
    sample_rate: u32,
) -> std::io::Result<()> {
    // Reassemble packets into single OGG stream in memory
    let mut ogg_data = Vec::new();
    for packet in packets {
        ogg_data.extend_from_slice(&packet.data);
    }

    let cursor = Cursor::new(ogg_data);
    let mut ogg = OggStreamReader::new(cursor).expect("Failed to parse Vorbis");

    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(filename, spec).unwrap();

    while let Some(packet) = ogg.read_dec_packet_itl().unwrap() {
        for sample in packet {
            writer.write_sample(sample).unwrap();
        }
    }

    writer.finalize().unwrap();
    Ok(())
}

/// Map frequency code to Hz
fn frequency_to_hz(code: u8) -> u32 {
    match code {
        1 => 8000,
        2 => 11000,
        3 => 11025,
        4 => 16000,
        5 => 22050,
        6 => 24000,
        7 => 32000,
        8 => 44100,
        9 => 48000,
        _ => 44100,
    }
}

// Convert FSB5 Vorbis packets into playable WAV
pub fn export_vorbis_sample_to_wav(
    sample_name: &str,
    packets: &[crate::fsb::VorbisPacket],
    sample_rate: u32,
    channels: u16,
) -> std::io::Result<()> {
    // Create a buffer for the minimal Ogg stream
    let mut ogg_data = Vec::new();

    // ----------------------------
    // Step 1: Write Ogg identification header
    // ----------------------------
    // For a minimal wrapper, we can use the "Vorbis identification header" structure:
    // [0x01][“vorbis”][version][channels][sample rate][bitrate fields][blocksize][framing]
    // This is 30 bytes normally; we keep it minimal
    ogg_data.extend_from_slice(&[
        0x01,
        b'v',
        b'o',
        b'r',
        b'b',
        b'i',
        b's', // packet type + "vorbis"
        0x00,
        0x00,
        0x00,
        0x00,           // version
        channels as u8, // channels
        (sample_rate & 0xFF) as u8,
        ((sample_rate >> 8) & 0xFF) as u8,
        ((sample_rate >> 16) & 0xFF) as u8,
        ((sample_rate >> 24) & 0xFF) as u8,
        0x00,
        0x00,
        0x00,
        0x00, // bitrate fields (nominal, max, min)
        0xB0, // blocksize + framing flag (just a placeholder)
        0x01, // framing
    ]);

    // ----------------------------
    // Step 2: Write comment header (minimal)
    // ----------------------------
    ogg_data.extend_from_slice(&[
        0x03, b'v', b'o', b'r', b'b', b'i', b's', // packet type + "vorbis"
        0x00, 0x00, 0x00, 0x00, // vendor length placeholder
        0x00, 0x00, 0x00, 0x00, // user comment list length placeholder
        0x01, // framing
    ]);

    // ----------------------------
    // Step 3: Append FSB5 Vorbis packets
    // ----------------------------
    for pkt in packets {
        ogg_data.extend_from_slice(&pkt.data);
    }

    // ----------------------------
    // Step 4: Decode Ogg Vorbis via Lewton
    // ----------------------------
    let cursor = Cursor::new(ogg_data);
    let mut reader = OggStreamReader::new(cursor).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Lewton error: {:?}", e),
        )
    })?;

    // Prepare WAV writer
    let spec = WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let out_path = format!("{}.wav", sample_name);
    let mut writer = WavWriter::create(&out_path, spec).unwrap();

    while let Some(pck) = reader.read_dec_packet_itl().unwrap() {
        for sample in pck {
            let val = (sample as f32 * i16::MAX as f32) as i16;
            writer.write_sample(val).unwrap();
        }
    }

    writer.finalize().unwrap();
    Ok(())
}

/// Export sample by index
pub fn export_sample_by_index(fsb: &crate::fsb::FSB5File, index: usize) -> std::io::Result<()> {
    if index >= fsb.sample_data.len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Index out of bounds",
        ));
    }

    let sample = &fsb.sample_data[index];
    let header = &fsb.sample_headers[index];

    let channels = header
        .extra_chunks
        .iter()
        .find_map(|c| {
            if let crate::fsb::ExtraChunk::Channels(ch) = c {
                Some(*ch)
            } else {
                None
            }
        })
        .unwrap_or(1);

    let sample_rate = frequency_to_hz(header.bitfield.frequency);
    let filename = format!("sample_{}.wav", index);

    match sample {
        crate::fsb::SampleData::Raw(data) => {
            export_pcm_sample(&filename, data, channels as u16, sample_rate)?
        }
        crate::fsb::SampleData::Vorbis(packets) => {
            export_vorbis_sample(&filename, packets, channels as u16, sample_rate)?
        }
    }

    println!("Exported sample {} -> {}", index, filename);
    Ok(())
}

/// Export sample by name from name table
pub fn export_sample_by_name(fsb: &crate::fsb::FSB5File, name: &str) -> std::io::Result<()> {
    if let Some(name_table) = &fsb.name_table {
        if let Some((index, _entry)) = name_table.iter().enumerate().find(|(_, e)| e.name == name) {
            return export_sample_by_index(fsb, index);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Sample name not found",
    ))
}

pub fn export_sample(fsb: &FSB5File, sample_index: usize) {
    let sample_name = if let Some(names) = &fsb.name_table {
        names
            .get(sample_index)
            .map(|n| n.name.clone())
            .unwrap_or("sample".to_string())
    } else {
        format!("sample_{}", sample_index)
    };

    if let SampleData::Vorbis(packets) = &fsb.sample_data[sample_index] {
        // Use the frequency mapping from your parser
        let freq_code = fsb.sample_headers[sample_index].bitfield.frequency;
        let sample_rate = match freq_code {
            1 => 8000,
            2 => 11000,
            3 => 11025,
            4 => 16000,
            5 => 22050,
            6 => 24000,
            7 => 32000,
            8 => 44100,
            9 => 48000,
            _ => 44100,
        };

        let channels = if fsb.sample_headers[sample_index].bitfield.two_channels {
            2
        } else {
            1
        };

        export_vorbis_sample_to_wav(&sample_name, packets, sample_rate, channels)
            .expect("Failed to export WAV");

        println!("Exported WAV: {}", sample_name);
    } else {
        println!("Sample {} is not Vorbis", sample_index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;

    const TEST_FSB_PATH: &str = "tests/skilvoice_jap.fsb";

    fn load_test_fsb() -> crate::fsb::FSB5File {
        let f = File::open(TEST_FSB_PATH).expect("Failed to open test FSB5");
        let mut reader = BufReader::new(f);
        crate::fsb::FSB5File::read(&mut reader).expect("Failed to parse FSB5")
    }

    #[test]
    fn export_by_index_creates_wav() {
        let fsb = load_test_fsb();
        let out_path = "test_sample_index.wav";

        // Remove if exists
        if Path::new(out_path).exists() {
            fs::remove_file(out_path).unwrap();
        }

        export_sample(&fsb, 0);
        assert!(Path::new(out_path).exists(), "WAV file was not created");

        // Cleanup
        fs::remove_file(out_path).unwrap();
    }

    #[test]
    fn export_by_name_creates_wav() {
        let fsb = load_test_fsb();
        let sample_name = fsb.name_table.as_ref().unwrap()[0].name.clone();
        let out_path = format!("test_sample_name.wav");

        if Path::new(&out_path).exists() {
            fs::remove_file(&out_path).unwrap();
        }

        export_sample_by_name(&fsb, &sample_name).expect("Failed to export sample by name");
        assert!(Path::new(&out_path).exists(), "WAV file was not created");

        // Cleanup
        fs::remove_file(&out_path).unwrap();
    }

    #[test]
    fn export_by_name_not_found() {
        let fsb = load_test_fsb();
        let err = export_sample_by_name(&fsb, "NON_EXISTENT_SAMPLE").unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn export_by_index_out_of_bounds() {
        let fsb = load_test_fsb();
        let err = export_sample_by_index(&fsb, fsb.sample_data.len()).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }

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
        println!("Mode: {:?}", fsb.header.mode);
        println!("Data size: {}", fsb.header.data_size);

        // --- Sample headers ---
        for (i, sh) in fsb.sample_headers.iter().enumerate() {
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
            for entry in name_table.iter() {
                println!("Name start: {}, Name: {}", entry.name_start, entry.name);
                break;
            }
        }

        // --- Sample data ---
        println!("Sample data lengths:");
        for (i, data) in fsb.sample_data.iter().enumerate() {
            println!("  Sample {}: {:?}", i, data);
            break;
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
