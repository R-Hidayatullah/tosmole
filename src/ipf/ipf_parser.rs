use std::io::{Cursor, Read, Seek};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::ies::ies_parser::ies_parse;
use crate::ipf::ipf_struct::{IPFFileTable, IpfFile};
use crate::ipf::ipf_util::ipf_read_string;
use crate::xac::xac_parser::xac_parse;
use crate::xsm::xsm_parser::xsm_parse;

#[link(name = "ipf_utility")]
extern "C" {
    pub fn ipf_decrypt(buffer: *mut u8, size: usize);
    pub fn ipf_encrypt(buffer: *mut u8, size: usize);
}

const HEADER_LOCATION: i64 = -24;
const MAGIC_NUMBER: usize = 4;

pub(crate) fn ipf_parse(ipf_file: &mut std::fs::File) -> IpfFile {
    let mut ipf_data = IpfFile::default();
    ipf_file
        .seek(std::io::SeekFrom::End(HEADER_LOCATION))
        .unwrap();
    ipf_data.footer.file_count = ipf_file.read_u16::<LittleEndian>().unwrap();
    ipf_data.footer.file_table_pointer = ipf_file.read_u32::<LittleEndian>().unwrap();
    ipf_file.read_u16::<LittleEndian>().unwrap(); //Padding
    ipf_data.footer.footer_pointer = ipf_file.read_u32::<LittleEndian>().unwrap();
    let mut magic = [0; MAGIC_NUMBER];
    ipf_file.read_exact(&mut magic).unwrap();
    ipf_data.footer.magic = Vec::from(magic);
    ipf_data.footer.version_to_patch = ipf_file.read_u32::<LittleEndian>().unwrap();
    ipf_data.footer.new_version = ipf_file.read_u32::<LittleEndian>().unwrap();
    let check_magic = [0x50, 0x4B, 0x05, 0x06];
    if ipf_data.footer.magic != check_magic {
        panic!("Not an IPF file: invalid header magic");
    }

    ipf_file
        .seek(std::io::SeekFrom::Start(
            ipf_data.footer.file_table_pointer as u64,
        ))
        .unwrap();
    let mut pb = pbr::ProgressBar::new(ipf_data.footer.file_count as u64);
    pb.format("╢▌▌░╟");
    println!("Start parsing IPF data.");
    for i in 0..ipf_data.footer.file_count {
        let mut ipf_file_table = IPFFileTable::default();
        ipf_file_table.idx = i as i32;
        ipf_file_table.filename_length = ipf_file.read_u16::<LittleEndian>().unwrap();
        ipf_file_table.crc32 = ipf_file.read_u32::<LittleEndian>().unwrap(); //Should be hexadecimal instead u32
        ipf_file_table.file_size_compressed = ipf_file.read_u32::<LittleEndian>().unwrap();
        ipf_file_table.file_size_uncompressed = ipf_file.read_u32::<LittleEndian>().unwrap();
        ipf_file_table.file_pointer = ipf_file.read_u32::<LittleEndian>().unwrap();
        ipf_file_table.container_name_length = ipf_file.read_u16::<LittleEndian>().unwrap();
        ipf_file_table.container_name =
            ipf_read_string(ipf_file, ipf_file_table.container_name_length);
        ipf_file_table.directory_name = ipf_read_string(ipf_file, ipf_file_table.filename_length);
        ipf_file_table.filename = std::path::Path::new(ipf_file_table.directory_name.as_str())
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let current_position = ipf_file.stream_position().unwrap();
        ipf_data.file_table.push(ipf_file_table);
        ipf_file
            .seek(std::io::SeekFrom::Start(current_position))
            .unwrap();
        pb.inc();
    }
    println!("\nFinish!");
    ipf_data
}
pub(crate) fn ipf_get_data(ipf_file: &mut std::fs::File, ipf_data: &IpfFile, index_num: usize) {
    let _default_decompressed = [
        "jpg", "fsb", "mp3", "fdp", "fev", "xml", "ies", "png", "tga", "lua",
    ];
    let _not_encrypted = [".mp3", ".fsb", ".jpg"];
    let ipf_table = &ipf_data.file_table[index_num];
    let mut data = ipf_read_data(
        ipf_file,
        ipf_table.file_pointer,
        ipf_table.file_size_compressed,
    );

    let extension = std::path::Path::new(ipf_table.filename.as_str())
        .extension()
        .unwrap_or("".as_ref())
        .to_str()
        .unwrap();

    if extension.ne("fsb") {
        data = ipf_decompress(
            &mut data,
            ipf_data.footer.new_version,
            ipf_table.file_size_uncompressed,
        );
    }

    //let output = data.clone();

    let _text_filename = [
        "xml", "effect", "skn", "3deffect", "3dworld", "3drender", "3dprop", "3dzone", "fx", "fxh",
        "h", "lst", "export", "skn", "fdp", "txt", "sani", "xsd", "sprbin", "fdp", "lua", "h",
    ];

    let _image_filename = ["png", "jpg", "dds", "gif", "jpeg", "bmp", "tga"];
    println!("{:?}", ipf_data.footer);
    println!("{:?}", ipf_table);
    if extension.eq("xac") {
        //let mut berkas = File::create("berkas").unwrap();
        //berkas.write_all(&data).unwrap();
        let mut _data = Cursor::new(data);
        let xac_file = xac_parse(&mut _data);
        println!();
        println!("{:?}", xac_file.header);
        println!("{:?}", xac_file.metadata);
        //let mut my_file = File::create(ipf_table.filename.clone() + "22.json").unwrap();
        //serde_json::to_writer_pretty(my_file, &xac_file).unwrap();
    } else if extension.eq("xsm") {
        let mut _data = Cursor::new(data);
        let xsm_file = xsm_parse(&mut _data);
        println!();
        println!("{:?}", xsm_file.header);
        println!("{:?}", xsm_file.metadata);
    } else if extension.eq("ies") {
        let mut _data = Cursor::new(data);
        let ies_file = ies_parse(&mut _data);
        println!();
        println!("{:?}", ies_file);
    } else if extension.eq("fsb") {
        let _data = Cursor::new(data);
        println!();
        println!("Parse FSB File!");
    } else if _text_filename.contains(&extension) {
        let xml_file = String::from_utf8(data).unwrap();
        println!();
        println!("{}", xml_file);
    } else if _image_filename.contains(&extension) {
        println!("Image data.");
    } else {
        println!("Unknown file type : {}", &extension);
    }
}

fn ipf_read_data(file: &mut std::fs::File, offset: u32, length: u32) -> Vec<u8> {
    file.seek(std::io::SeekFrom::Start(offset as u64)).unwrap();
    let mut data = vec![0; length as usize];
    file.read_exact(&mut data).unwrap();
    data
}

fn ipf_decompress(data: &mut Vec<u8>, version: u32, uncompressed_size: u32) -> Vec<u8> {
    if version > 11000 || version == 0 {
        let size = data.len();
        unsafe {
            ipf_decrypt(data.as_mut_ptr(), size);
        }
    }

    if uncompressed_size as usize <= data.len() {
        data.to_owned()
    } else {
        ipf_decompress_data(data, uncompressed_size)
    }
}

fn ipf_decompress_data(data: &Vec<u8>, uncompressed_size: u32) -> Vec<u8> {
    let input_data = data.as_slice();

    let mut output_data = Vec::with_capacity(uncompressed_size as usize);
    flate2::Decompress::new(false)
        .decompress_vec(
            input_data,
            &mut output_data,
            flate2::FlushDecompress::Finish,
        )
        .expect("Decompressing buffer!");
    output_data
}
