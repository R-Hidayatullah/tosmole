#![allow(dead_code)]

use std::io::{Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::xsm::xsm_enums::XsmChunkType::{XsmBoneAnimationId, XsmMetadataId};
use crate::xsm::xsm_structs::{
    Xsm, XsmChunk, XsmPosKey, XsmRotKey, XsmScaleKey, XsmScaleRotKey, XsmSubMotion,
};
use crate::xsm::xsm_util::{xsm_read_quaternion16, xsm_read_string, xsm_read_vec3d};

const MAGIC_NUMBER: usize = 4;

pub fn xsm_parse<R: Read + Seek>(xsm_file: &mut R) -> Xsm {
    let mut xsm_new = Xsm::default();
    read_header(xsm_file, &mut xsm_new);
    read_chunk(xsm_file, &mut xsm_new);
    xsm_new
}

fn read_header<'a, R: Read + Seek>(file: &'a mut R, xsm: &'a mut Xsm) -> &'a mut Xsm {
    let mut magic = [0; MAGIC_NUMBER];
    file.read_exact(&mut magic).unwrap();
    xsm.header.magic = std::str::from_utf8(&magic).unwrap().to_string();
    xsm.header.major_version = file.read_u8().unwrap();
    xsm.header.minor_version = file.read_u8().unwrap();
    xsm.header.big_endian = file.read_u8().unwrap() != 0;
    file.read_u8().unwrap(); // Padding
    xsm
}

fn read_chunk<'a, R: Read + Seek>(file: &'a mut R, xsm: &'a mut Xsm) -> &'a mut Xsm {
    while file.stream_position().unwrap() < file.stream_len().unwrap() {
        let chunk = XsmChunk {
            chunk_type: file.read_i32::<LittleEndian>().unwrap(),
            length: file.read_i32::<LittleEndian>().unwrap(),
            version: file.read_i32::<LittleEndian>().unwrap(),
        };
        let position = file.stream_position().unwrap();
        if chunk.chunk_type == XsmMetadataId as i32 {
            read_metadata(file, xsm);
        }
        if chunk.chunk_type == XsmBoneAnimationId as i32 {
            read_bone_animation(file, xsm);
        }
        file.seek(SeekFrom::Start(position + chunk.length as u64))
            .unwrap();
    }
    xsm
}

fn read_metadata<'a, R: Read + Seek>(file: &'a mut R, xsm: &'a mut Xsm) -> &'a mut Xsm {
    xsm.metadata.unused = file.read_f32::<LittleEndian>().unwrap();
    xsm.metadata.max_acceptable_error = file.read_f32::<LittleEndian>().unwrap();
    xsm.metadata.fps = file.read_i32::<LittleEndian>().unwrap();
    xsm.metadata.exporter_major_version = file.read_u8().unwrap();
    xsm.metadata.exporter_minor_version = file.read_u8().unwrap();
    file.read_u8().unwrap(); //Padding
    file.read_u8().unwrap(); //Padding
    xsm.metadata.source_app = xsm_read_string(file);
    xsm.metadata.original_filename = xsm_read_string(file);
    xsm.metadata.export_date = xsm_read_string(file);
    xsm.metadata.motion_name = xsm_read_string(file);
    xsm
}

fn read_bone_animation<'a, R: Read + Seek>(file: &'a mut R, xsm: &'a mut Xsm) -> &'a mut Xsm {
    xsm.bone_animation.num_submotion = file.read_i32::<LittleEndian>().unwrap();
    for _ in 0..xsm.bone_animation.num_submotion {
        xsm.bone_animation.skeletal_submotion.push({
            let mut submotion = XsmSubMotion {
                pose_rot: xsm_read_quaternion16(file),
                bind_pose_rot: xsm_read_quaternion16(file),
                pose_scale_rot: xsm_read_quaternion16(file),
                bind_pose_scale_rot: xsm_read_quaternion16(file),
                pose_pos: xsm_read_vec3d(file),
                pose_scale: xsm_read_vec3d(file),
                bind_pose_pos: xsm_read_vec3d(file),
                bind_pose_scale_pos: xsm_read_vec3d(file),
                num_pos_keys: file.read_i32::<LittleEndian>().unwrap(),
                num_rot_keys: file.read_i32::<LittleEndian>().unwrap(),
                num_scale_keys: file.read_i32::<LittleEndian>().unwrap(),
                num_scale_rot_keys: file.read_i32::<LittleEndian>().unwrap(),
                max_error: file.read_f32::<LittleEndian>().unwrap(),
                node_name: xsm_read_string(file),
                pos_key: vec![],
                rot_key: vec![],
                scale_key: vec![],
                scale_rot_key: vec![],
            };

            for _ in 0..submotion.num_pos_keys {
                submotion.pos_key.push(XsmPosKey {
                    pos: xsm_read_vec3d(file),
                    time: file.read_f32::<LittleEndian>().unwrap(),
                })
            }

            for _ in 0..submotion.num_rot_keys {
                submotion.rot_key.push(XsmRotKey {
                    rot: xsm_read_quaternion16(file),
                    time: file.read_f32::<LittleEndian>().unwrap(),
                })
            }
            for _ in 0..submotion.num_scale_keys {
                submotion.scale_key.push(XsmScaleKey {
                    scale: xsm_read_vec3d(file),
                    time: file.read_f32::<LittleEndian>().unwrap(),
                })
            }

            for _ in 0..submotion.num_scale_rot_keys {
                submotion.scale_rot_key.push(XsmScaleRotKey {
                    rot: xsm_read_quaternion16(file),
                    time: file.read_f32::<LittleEndian>().unwrap(),
                })
            }
            submotion
        });
    }

    xsm
}
