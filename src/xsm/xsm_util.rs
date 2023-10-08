#![allow(dead_code)]

use std::io::{Read, Seek};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::xsm::xsm_structs::{XsmQuaternion16, XsmVec3d};

pub(crate) fn xsm_read_string<R: Read + Seek>(file: &mut R) -> String {
    let mut text = String::new();
    let length = file.read_i32::<LittleEndian>().unwrap();
    for _ in 0..length {
        let character = file.read_u8().unwrap();
        text.push(character as char);
    }
    text
}

pub(crate) fn xsm_read_quaternion16<R: Read + Seek>(file: &mut R) -> XsmQuaternion16 {
    let quat = XsmQuaternion16 {
        x: file.read_i16::<LittleEndian>().unwrap(),
        y: file.read_i16::<LittleEndian>().unwrap(),
        z: file.read_i16::<LittleEndian>().unwrap(),
        w: file.read_i16::<LittleEndian>().unwrap(),
    };
    quat
}

pub(crate) fn xsm_read_vec3d<R: Read + Seek>(file: &mut R) -> XsmVec3d {
    let vec3d = XsmVec3d {
        x: file.read_f32::<LittleEndian>().unwrap(),
        y: file.read_f32::<LittleEndian>().unwrap(),
        z: file.read_f32::<LittleEndian>().unwrap(),
    };
    vec3d
}
