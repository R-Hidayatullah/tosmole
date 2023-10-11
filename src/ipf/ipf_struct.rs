#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct IPFFooter {
    pub(crate) version_to_patch: u32,
    pub(crate) new_version: u32,
    pub(crate) file_count: u16,
    pub(crate) file_table_pointer: u32,
    pub(crate) footer_pointer: u32,
    pub(crate) magic: Vec<u8>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct IPFFileTable {
    pub(crate) idx: i32,
    pub(crate) filename_length: u16,
    pub(crate) container_name_length: u16,
    pub(crate) file_size_compressed: u32,
    pub(crate) file_size_uncompressed: u32,
    pub(crate) file_pointer: u32,
    pub(crate) crc32: u32,
    pub(crate) container_name: String,
    pub(crate) filename: String,
    pub(crate) directory_name: String,
    pub(crate) content: Vec<u8>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct IpfFile {
    pub(crate) footer: IPFFooter,
    pub(crate) file_table: Vec<IPFFileTable>,
}
