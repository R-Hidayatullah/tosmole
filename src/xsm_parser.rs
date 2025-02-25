use std::{
    fs::File,
    io::{self, BufReader, Cursor, Read, Seek, SeekFrom},
    path::Path,
};

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

enum XsmChunkType {
    XsmMetadataId = 201,
    XsmBoneAnimationId = 202,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XsmVec3d {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XsmQuaternion16 {
    x: i16,
    y: i16,
    z: i16,
    w: i16,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct XsmFile {
    header: XsmHeader,
    metadata: XsmMetadata,
    bone_animation: XsmBoneAnimation,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XsmHeader {
    magic: String,
    major_version: u8,
    minor_version: u8,
    big_endian: bool,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XsmChunk {
    chunk_type: i32,
    length: i32,
    version: i32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XsmMetadata {
    unused: f32,
    max_acceptable_error: f32,
    fps: i32,
    exporter_major_version: u8,
    exporter_minor_version: u8,
    source_app: String,
    original_filename: String,
    export_date: String,
    motion_name: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XsmBoneAnimation {
    num_submotion: i32,
    skeletal_submotion: Vec<XsmSubMotion>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XsmSubMotion {
    pose_rot: XsmQuaternion16,
    bind_pose_rot: XsmQuaternion16,
    pose_scale_rot: XsmQuaternion16,
    bind_pose_scale_rot: XsmQuaternion16,
    pose_pos: XsmVec3d,
    pose_scale: XsmVec3d,
    bind_pose_pos: XsmVec3d,
    bind_pose_scale_pos: XsmVec3d,
    num_pos_keys: i32,
    num_rot_keys: i32,
    num_scale_keys: i32,
    num_scale_rot_keys: i32,
    max_error: f32,
    node_name: String,
    pos_key: Vec<XsmPosKey>,
    rot_key: Vec<XsmRotKey>,
    scale_key: Vec<XsmScaleKey>,
    scale_rot_key: Vec<XsmScaleRotKey>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XsmPosKey {
    pos: XsmVec3d,
    time: f32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XsmRotKey {
    rot: XsmQuaternion16,
    time: f32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XsmScaleKey {
    scale: XsmVec3d,
    time: f32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XsmScaleRotKey {
    rot: XsmQuaternion16,
    time: f32,
}

const MAGIC_NUMBER: usize = 4;

impl XsmFile {
    /// Load XSM data from a file specified by the file path.
    ///
    /// # Arguments
    ///
    /// * `file_path` - A path to the XSM file to be loaded.
    ///
    /// # Returns
    ///
    /// A Result containing the parsed `XsmFile` or an IO error if the file cannot be read.
    pub fn load_from_file<P: AsRef<Path>>(file_path: P) -> io::Result<Self> {
        // Check if the file extension is '.xsm'
        let file_path_str = file_path.as_ref().to_str().unwrap_or("");
        if !file_path_str.to_lowercase().ends_with(".xsm") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid file extension. Expected '.xsm'.",
            ));
        }

        // Open the file and create a buffered reader.
        let file = std::fs::File::open(file_path)?;
        let mut buf_reader = BufReader::new(file);

        // Delegate to load_from_reader for further processing.
        Self::load_from_reader(&mut buf_reader)
    }

    /// Load XSM data from a byte vector.
    ///
    /// # Arguments
    ///
    /// * `bytes` - A vector of bytes containing the XSM file data.
    ///
    /// # Returns
    ///
    /// A Result containing the parsed `XsmFile` or an IO error if the byte vector is invalid.
    pub fn load_from_bytes(mut bytes: Vec<u8>) -> io::Result<Self> {
        // Create a cursor from the byte vector.
        let mut cursor = Cursor::new(&mut bytes);

        // Delegate to load_from_reader for further processing.
        Self::load_from_reader(&mut cursor)
    }

    fn load_from_reader<R: Read + Seek>(reader: &mut R) -> io::Result<Self> {
        let mut xsm_data = XsmFile::default();
        xsm_data.read_header(reader)?;
        xsm_data.read_chunk(reader)?;
        Ok(xsm_data)
    }

    fn read_header<R: Read + Seek>(&mut self, file: &mut R) -> io::Result<&mut Self> {
        let mut magic = [0; MAGIC_NUMBER];
        file.read_exact(&mut magic).unwrap();
        self.header.magic = std::str::from_utf8(&magic).unwrap().to_string();
        self.header.major_version = file.read_u8().unwrap();
        self.header.minor_version = file.read_u8().unwrap();
        self.header.big_endian = file.read_u8().unwrap() != 0;
        file.read_u8().unwrap(); // Padding
        Ok(self)
    }

    fn read_chunk<R: Read + Seek>(&mut self, file: &mut R) -> io::Result<&mut Self> {
        let current_pos = file.stream_position()?; // Get current position
        let file_len = file.seek(SeekFrom::End(0))?; // Get file length
        file.seek(SeekFrom::Start(current_pos))?; // Restore original position

        while file.stream_position()? < file_len {
            let chunk = XsmChunk {
                chunk_type: file.read_i32::<LittleEndian>().unwrap(),
                length: file.read_i32::<LittleEndian>().unwrap(),
                version: file.read_i32::<LittleEndian>().unwrap(),
            };
            let position = file.stream_position().unwrap();
            if chunk.chunk_type == XsmChunkType::XsmMetadataId as i32 {
                self.read_metadata(file)?;
            }
            if chunk.chunk_type == XsmChunkType::XsmBoneAnimationId as i32 {
                self.read_bone_animation(file)?;
            }
            file.seek(SeekFrom::Start(position + chunk.length as u64))
                .unwrap();
        }
        Ok(self)
    }

    fn read_metadata<R: Read + Seek>(&mut self, file: &mut R) -> io::Result<&mut Self> {
        self.metadata.unused = file.read_f32::<LittleEndian>().unwrap();
        self.metadata.max_acceptable_error = file.read_f32::<LittleEndian>().unwrap();
        self.metadata.fps = file.read_i32::<LittleEndian>().unwrap();
        self.metadata.exporter_major_version = file.read_u8().unwrap();
        self.metadata.exporter_minor_version = file.read_u8().unwrap();
        file.read_u8().unwrap(); // Padding
        file.read_u8().unwrap(); // Padding
        self.metadata.source_app = self.read_string(file)?;
        self.metadata.original_filename = self.read_string(file)?;
        self.metadata.export_date = self.read_string(file)?;
        self.metadata.motion_name = self.read_string(file)?;
        Ok(self)
    }
    fn read_bone_animation<R: Read + Seek>(&mut self, file: &mut R) -> io::Result<&mut Self> {
        self.bone_animation.num_submotion = file.read_i32::<LittleEndian>().unwrap();
        for _ in 0..self.bone_animation.num_submotion {
            let mut submotion = XsmSubMotion {
                pose_rot: Self::xsm_read_quaternion16(file),
                bind_pose_rot: Self::xsm_read_quaternion16(file),
                pose_scale_rot: Self::xsm_read_quaternion16(file),
                bind_pose_scale_rot: Self::xsm_read_quaternion16(file),
                pose_pos: Self::xsm_read_vec3d(file),
                pose_scale: Self::xsm_read_vec3d(file),
                bind_pose_pos: Self::xsm_read_vec3d(file),
                bind_pose_scale_pos: Self::xsm_read_vec3d(file),
                num_pos_keys: file.read_i32::<LittleEndian>().unwrap(),
                num_rot_keys: file.read_i32::<LittleEndian>().unwrap(),
                num_scale_keys: file.read_i32::<LittleEndian>().unwrap(),
                num_scale_rot_keys: file.read_i32::<LittleEndian>().unwrap(),
                max_error: file.read_f32::<LittleEndian>().unwrap(),
                node_name: self.read_string(file)?,
                pos_key: vec![],
                rot_key: vec![],
                scale_key: vec![],
                scale_rot_key: vec![],
            };

            for _ in 0..submotion.num_pos_keys {
                submotion.pos_key.push(XsmPosKey {
                    pos: Self::xsm_read_vec3d(file),
                    time: file.read_f32::<LittleEndian>().unwrap(),
                });
            }

            for _ in 0..submotion.num_rot_keys {
                submotion.rot_key.push(XsmRotKey {
                    rot: Self::xsm_read_quaternion16(file),
                    time: file.read_f32::<LittleEndian>().unwrap(),
                });
            }

            for _ in 0..submotion.num_scale_keys {
                submotion.scale_key.push(XsmScaleKey {
                    scale: Self::xsm_read_vec3d(file),
                    time: file.read_f32::<LittleEndian>().unwrap(),
                });
            }

            for _ in 0..submotion.num_scale_rot_keys {
                submotion.scale_rot_key.push(XsmScaleRotKey {
                    rot: Self::xsm_read_quaternion16(file),
                    time: file.read_f32::<LittleEndian>().unwrap(),
                });
            }

            self.bone_animation.skeletal_submotion.push(submotion);
        }
        Ok(self)
    }
    fn read_string<R: Read + Seek>(&mut self, file: &mut R) -> io::Result<String> {
        let mut text = String::new();
        let length = file.read_i32::<LittleEndian>().unwrap();
        for _ in 0..length {
            let character = file.read_u8().unwrap();
            text.push(character as char);
        }
        Ok(text)
    }
    fn xsm_read_quaternion16<R: Read + Seek>(file: &mut R) -> XsmQuaternion16 {
        XsmQuaternion16 {
            x: file.read_i16::<LittleEndian>().unwrap(),
            y: file.read_i16::<LittleEndian>().unwrap(),
            z: file.read_i16::<LittleEndian>().unwrap(),
            w: file.read_i16::<LittleEndian>().unwrap(),
        }
    }

    fn xsm_read_vec3d<R: Read + Seek>(file: &mut R) -> XsmVec3d {
        XsmVec3d {
            x: file.read_f32::<LittleEndian>().unwrap(),
            y: file.read_f32::<LittleEndian>().unwrap(),
            z: file.read_f32::<LittleEndian>().unwrap(),
        }
    }
}

#[test]
fn test_xsm_parser() {
    // Provide the path to the test XSM file
    let file_path = "tests/npc_Catapult_wlk.xsm";

    // Read the content of the file
    let mut file_content = Vec::new();
    let mut file = File::open(&file_path).expect("Failed to open file");
    file.read_to_end(&mut file_content)
        .expect("Failed to read file content");

    // Parse the XSM file
    let _ = XsmFile::load_from_bytes(file_content).expect("Failed to parse XSM file");
}
