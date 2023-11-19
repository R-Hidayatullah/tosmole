use std::{
    fs::File,
    io::{self, BufReader, Cursor, Read, Seek},
    path::Path,
    time::Instant,
};

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

#[link(name = "ipf_utility")]
extern "C" {
    pub fn ipf_decrypt(buffer: *mut u8, size: usize);
    pub fn ipf_encrypt(buffer: *mut u8, size: usize);
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct IPFFooter {
    version_to_patch: u32,
    new_version: u32,
    file_count: u16,
    file_table_pointer: u32,
    footer_pointer: u32,
    magic: Vec<u8>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct IPFFileTable {
    idx: i32,
    filename_length: u16,
    container_name_length: u16,
    file_size_compressed: u32,
    file_size_uncompressed: u32,
    file_pointer: u32,
    crc32: u32,
    container_name: String,
    filename: String,
    directory_name: String,
    content: Vec<u8>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct IpfFile {
    footer: IPFFooter,
    file_table: Vec<IPFFileTable>,
}

const HEADER_LOCATION: i64 = -24;
const MAGIC_NUMBER: usize = 4;

impl IpfFile {
    /// Load IPF data from a file specified by the file path.
    ///
    /// # Arguments
    ///
    /// * `file_path` - A path to the IPF file to be loaded.
    ///
    /// # Returns
    ///
    /// A Result containing the parsed `IpfFile` or an IO error if the file cannot be read.
    pub fn load_from_file<P: AsRef<Path>>(file_path: P) -> io::Result<Self> {
        // Check if the file extension is '.ipf'
        let file_path_str = file_path.as_ref().to_str().unwrap_or("");
        if !file_path_str.to_lowercase().ends_with(".ipf") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid file extension. Expected '.ipf'.",
            ));
        }

        // Open the file and create a buffered reader.
        let file = std::fs::File::open(file_path)?;
        let mut buf_reader = BufReader::new(file);

        // Delegate to load_from_reader for further processing.
        Self::load_from_reader(&mut buf_reader)
    }

    fn load_from_reader<R: Read + Seek>(reader: &mut R) -> io::Result<Self> {
        let mut ipf_data = IpfFile::default();
        ipf_data.read_header(reader)?;
        ipf_data.read_data_tables(reader)?;
        Ok(ipf_data)
    }

    fn read_header<R: Read + Seek>(&mut self, file: &mut R) -> io::Result<&mut Self> {
        // Move to the end of the file to read the header
        file.seek(std::io::SeekFrom::End(HEADER_LOCATION))?;

        // Read and set footer information
        self.footer.file_count = file.read_u16::<LittleEndian>()?;
        self.footer.file_table_pointer = file.read_u32::<LittleEndian>()?;
        file.read_u16::<LittleEndian>()?; // Padding
        self.footer.footer_pointer = file.read_u32::<LittleEndian>()?;

        // Read magic number
        let mut magic = [0; MAGIC_NUMBER];
        file.read_exact(&mut magic)?;
        self.footer.magic = Vec::from(magic);

        // Read version information
        self.footer.version_to_patch = file.read_u32::<LittleEndian>()?;
        self.footer.new_version = file.read_u32::<LittleEndian>()?;

        // Check the magic number to verify if it's a valid IPF file
        let check_magic = [0x50, 0x4B, 0x05, 0x06];
        if self.footer.magic != check_magic {
            panic!("Not an IPF file: invalid header magic");
        }

        Ok(self)
    }
    fn read_data_tables<R: Read + Seek>(&mut self, file: &mut R) -> io::Result<&mut Self> {
        file.seek(std::io::SeekFrom::Start(
            self.footer.file_table_pointer as u64,
        ))?;

        for i in 0..self.footer.file_count {
            let mut ipf_file_table = IPFFileTable::default();
            ipf_file_table.idx = i as i32;
            ipf_file_table.filename_length = file.read_u16::<LittleEndian>()?;
            ipf_file_table.crc32 = file.read_u32::<LittleEndian>()?; // Should be hexadecimal instead u32
            ipf_file_table.file_size_compressed = file.read_u32::<LittleEndian>()?;
            ipf_file_table.file_size_uncompressed = file.read_u32::<LittleEndian>()?;
            ipf_file_table.file_pointer = file.read_u32::<LittleEndian>()?;
            ipf_file_table.container_name_length = file.read_u16::<LittleEndian>()?;
            ipf_file_table.container_name =
                Self::ipf_read_string(file, ipf_file_table.container_name_length)?;
            ipf_file_table.directory_name =
                Self::ipf_read_string(file, ipf_file_table.filename_length)?;
            ipf_file_table.filename = std::path::Path::new(ipf_file_table.directory_name.as_str())
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            let current_position = file.stream_position()?;
            self.file_table.push(ipf_file_table);
            file.seek(std::io::SeekFrom::Start(current_position))?;
        }

        Ok(self)
    }

    fn ipf_decompress(data: &Vec<u8>, version: u32, uncompressed_size: u32) -> Vec<u8> {
        if version > 11000 || version == 0 {
            let size = data.len();
            unsafe {
                ipf_decrypt(data.as_ptr() as *mut u8, size);
            }
        }

        if uncompressed_size as usize <= data.len() {
            data.to_owned()
        } else {
            Self::ipf_decompress_data(data, uncompressed_size)
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

    fn ipf_read_string<R: Read + Seek>(file: &mut R, length: u16) -> io::Result<String> {
        let mut text = String::new();
        for _ in 0..length {
            let character = file.read_u8()?;
            text.push(character as char);
        }
        Ok(text)
    }

    pub fn get_data<P: AsRef<Path>>(
        &mut self,
        file_path: P,
        index: usize,
    ) -> io::Result<&mut Self> {
        // Check if the file extension is '.ipf'
        let file_path_str = file_path.as_ref().to_str().unwrap_or("");
        if !file_path_str.to_lowercase().ends_with(".ipf") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid file extension. Expected '.ipf'.",
            ));
        }

        // Open the file and create a buffered reader.
        let file = std::fs::File::open(file_path)?;
        let mut buf_reader = BufReader::new(file);

        let ipf_table = &self.file_table[index];

        // Call ipf_read_data to read the compressed data
        let mut data = Self::ipf_read_data(
            &mut buf_reader,
            ipf_table.file_pointer,
            ipf_table.file_size_compressed,
        );

        let extension = std::path::Path::new(&ipf_table.filename)
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("");

        if extension.ne("fsb") {
            // Call ipf_decompress to decompress the data
            data = Self::ipf_decompress(
                &mut data,
                self.footer.new_version,
                ipf_table.file_size_uncompressed,
            );
        }

        let _text_filename = [
            "xml", "effect", "skn", "3deffect", "3dworld", "3drender", "3dprop", "3dzone", "fx",
            "fxh", "h", "lst", "export", "skn", "fdp", "txt", "sani", "xsd", "sprbin", "fdp",
            "lua", "h",
        ];

        let _image_filename = ["png", "jpg", "dds", "gif", "jpeg", "bmp", "tga"];
        if extension.eq("xac") {
            let mut _data = Cursor::new(data);
        } else if extension.eq("xsm") {
            let mut _data = Cursor::new(data);
            println!();
        } else if extension.eq("ies") {
            let mut _data = Cursor::new(data);
            println!();
        } else if extension.eq("fsb") {
            let _data = Cursor::new(data);
            println!();
        } else if _text_filename.contains(&extension) {
            let xml_file = String::from_utf8(data).unwrap();
            println!();
            println!("{}", xml_file);
        } else if _image_filename.contains(&extension) {
            println!("Image data.");
        } else {
            println!("Unknown file type : {}", &extension);
        }
        Ok(self)
    }

    fn ipf_read_data(file: &mut BufReader<File>, offset: u32, length: u32) -> Vec<u8> {
        file.seek(std::io::SeekFrom::Start(offset as u64)).unwrap();
        let mut data = vec![0; length as usize];
        file.read_exact(&mut data).unwrap();
        data
    }
}
