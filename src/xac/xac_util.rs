#![allow(dead_code)]

use std::io::{Read, Seek};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::xac::xac_struct::{
    XacColor, XacColor8, XacMatrix44, XacQuaternion, XacVec2d, XacVec3d, XacVec4d,
};

pub(crate) fn xac_read_string<R: Read + Seek>(reader: &mut R) -> String {
    let mut text = String::new();
    let length = reader.read_i32::<LittleEndian>().unwrap();
    for _ in 0..length {
        let character = reader.read_u8().unwrap();
        text.push(character as char);
    }
    text
}
pub(crate) fn xac_read_boolean<R: Read + Seek>(file: &mut R) -> bool {
    file.read_u8().unwrap() != 0
}
pub(crate) fn xac_read_color8<R: Read + Seek>(file: &mut R) -> XacColor8 {
    XacColor8 {
        x: file.read_u8().unwrap(),
        y: file.read_u8().unwrap(),
        z: file.read_u8().unwrap(),
    }
}

pub(crate) fn xac_read_color<R: Read + Seek>(file: &mut R) -> XacColor {
    XacColor {
        x: file.read_f32::<LittleEndian>().unwrap(),
        y: file.read_f32::<LittleEndian>().unwrap(),
        z: file.read_f32::<LittleEndian>().unwrap(),
    }
}

pub(crate) fn xac_read_vec2d<R: Read + Seek>(file: &mut R) -> XacVec2d {
    XacVec2d {
        x: file.read_f32::<LittleEndian>().unwrap(),
        y: file.read_f32::<LittleEndian>().unwrap(),
    }
}
pub(crate) fn xac_read_vec3d<R: Read + Seek>(file: &mut R) -> XacVec3d {
    XacVec3d {
        x: file.read_f32::<LittleEndian>().unwrap(),
        y: file.read_f32::<LittleEndian>().unwrap(),
        z: file.read_f32::<LittleEndian>().unwrap(),
    }
}
pub(crate) fn xac_read_vec4d<R: Read + Seek>(file: &mut R) -> XacVec4d {
    XacVec4d {
        x: file.read_f32::<LittleEndian>().unwrap(),
        y: file.read_f32::<LittleEndian>().unwrap(),
        z: file.read_f32::<LittleEndian>().unwrap(),
        w: file.read_f32::<LittleEndian>().unwrap(),
    }
}

pub(crate) fn xac_read_quaternion<R: Read + Seek>(file: &mut R) -> XacQuaternion {
    XacQuaternion {
        x: file.read_f32::<LittleEndian>().unwrap(),
        y: file.read_f32::<LittleEndian>().unwrap(),
        z: file.read_f32::<LittleEndian>().unwrap(),
        w: file.read_f32::<LittleEndian>().unwrap(),
    }
}

pub(crate) fn xac_read_matrix44<R: Read + Seek>(file: &mut R) -> XacMatrix44 {
    XacMatrix44 {
        axis_1: xac_read_vec4d(file),
        axis_2: xac_read_vec4d(file),
        axis_3: xac_read_vec4d(file),
        pos: xac_read_vec4d(file),
    }
}
