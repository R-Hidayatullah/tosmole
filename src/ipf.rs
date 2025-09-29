use binrw::{BinReaderExt, binread};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::BTreeMap,
    fs::{File, read_dir},
    io::{self, BufReader, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
    thread,
};

const HEADER_LOCATION: i64 = -24;
const MAGIC_NUMBER: u32 = 0x06054B50;
const CRC32_TABLE: [u32; 256] = [
    0x00000000, 0x77073096, 0xee0e612c, 0x990951ba, 0x076dc419, 0x706af48f, 0xe963a535, 0x9e6495a3,
    0x0edb8832, 0x79dcb8a4, 0xe0d5e91e, 0x97d2d988, 0x09b64c2b, 0x7eb17cbd, 0xe7b82d07, 0x90bf1d91,
    0x1db71064, 0x6ab020f2, 0xf3b97148, 0x84be41de, 0x1adad47d, 0x6ddde4eb, 0xf4d4b551, 0x83d385c7,
    0x136c9856, 0x646ba8c0, 0xfd62f97a, 0x8a65c9ec, 0x14015c4f, 0x63066cd9, 0xfa0f3d63, 0x8d080df5,
    0x3b6e20c8, 0x4c69105e, 0xd56041e4, 0xa2677172, 0x3c03e4d1, 0x4b04d447, 0xd20d85fd, 0xa50ab56b,
    0x35b5a8fa, 0x42b2986c, 0xdbbbc9d6, 0xacbcf940, 0x32d86ce3, 0x45df5c75, 0xdcd60dcf, 0xabd13d59,
    0x26d930ac, 0x51de003a, 0xc8d75180, 0xbfd06116, 0x21b4f4b5, 0x56b3c423, 0xcfba9599, 0xb8bda50f,
    0x2802b89e, 0x5f058808, 0xc60cd9b2, 0xb10be924, 0x2f6f7c87, 0x58684c11, 0xc1611dab, 0xb6662d3d,
    0x76dc4190, 0x01db7106, 0x98d220bc, 0xefd5102a, 0x71b18589, 0x06b6b51f, 0x9fbfe4a5, 0xe8b8d433,
    0x7807c9a2, 0x0f00f934, 0x9609a88e, 0xe10e9818, 0x7f6a0dbb, 0x086d3d2d, 0x91646c97, 0xe6635c01,
    0x6b6b51f4, 0x1c6c6162, 0x856530d8, 0xf262004e, 0x6c0695ed, 0x1b01a57b, 0x8208f4c1, 0xf50fc457,
    0x65b0d9c6, 0x12b7e950, 0x8bbeb8ea, 0xfcb9887c, 0x62dd1ddf, 0x15da2d49, 0x8cd37cf3, 0xfbd44c65,
    0x4db26158, 0x3ab551ce, 0xa3bc0074, 0xd4bb30e2, 0x4adfa541, 0x3dd895d7, 0xa4d1c46d, 0xd3d6f4fb,
    0x4369e96a, 0x346ed9fc, 0xad678846, 0xda60b8d0, 0x44042d73, 0x33031de5, 0xaa0a4c5f, 0xdd0d7cc9,
    0x5005713c, 0x270241aa, 0xbe0b1010, 0xc90c2086, 0x5768b525, 0x206f85b3, 0xb966d409, 0xce61e49f,
    0x5edef90e, 0x29d9c998, 0xb0d09822, 0xc7d7a8b4, 0x59b33d17, 0x2eb40d81, 0xb7bd5c3b, 0xc0ba6cad,
    0xedb88320, 0x9abfb3b6, 0x03b6e20c, 0x74b1d29a, 0xead54739, 0x9dd277af, 0x04db2615, 0x73dc1683,
    0xe3630b12, 0x94643b84, 0x0d6d6a3e, 0x7a6a5aa8, 0xe40ecf0b, 0x9309ff9d, 0x0a00ae27, 0x7d079eb1,
    0xf00f9344, 0x8708a3d2, 0x1e01f268, 0x6906c2fe, 0xf762575d, 0x806567cb, 0x196c3671, 0x6e6b06e7,
    0xfed41b76, 0x89d32be0, 0x10da7a5a, 0x67dd4acc, 0xf9b9df6f, 0x8ebeeff9, 0x17b7be43, 0x60b08ed5,
    0xd6d6a3e8, 0xa1d1937e, 0x38d8c2c4, 0x4fdff252, 0xd1bb67f1, 0xa6bc5767, 0x3fb506dd, 0x48b2364b,
    0xd80d2bda, 0xaf0a1b4c, 0x36034af6, 0x41047a60, 0xdf60efc3, 0xa867df55, 0x316e8eef, 0x4669be79,
    0xcb61b38c, 0xbc66831a, 0x256fd2a0, 0x5268e236, 0xcc0c7795, 0xbb0b4703, 0x220216b9, 0x5505262f,
    0xc5ba3bbe, 0xb2bd0b28, 0x2bb45a92, 0x5cb36a04, 0xc2d7ffa7, 0xb5d0cf31, 0x2cd99e8b, 0x5bdeae1d,
    0x9b64c2b0, 0xec63f226, 0x756aa39c, 0x026d930a, 0x9c0906a9, 0xeb0e363f, 0x72076785, 0x05005713,
    0x95bf4a82, 0xe2b87a14, 0x7bb12bae, 0x0cb61b38, 0x92d28e9b, 0xe5d5be0d, 0x7cdcefb7, 0x0bdbdf21,
    0x86d3d2d4, 0xf1d4e242, 0x68ddb3f8, 0x1fda836e, 0x81be16cd, 0xf6b9265b, 0x6fb077e1, 0x18b74777,
    0x88085ae6, 0xff0f6a70, 0x66063bca, 0x11010b5c, 0x8f659eff, 0xf862ae69, 0x616bffd3, 0x166ccf45,
    0xa00ae278, 0xd70dd2ee, 0x4e048354, 0x3903b3c2, 0xa7672661, 0xd06016f7, 0x4969474d, 0x3e6e77db,
    0xaed16a4a, 0xd9d65adc, 0x40df0b66, 0x37d83bf0, 0xa9bcae53, 0xdebb9ec5, 0x47b2cf7f, 0x30b5ffe9,
    0xbdbdf21c, 0xcabac28a, 0x53b39330, 0x24b4a3a6, 0xbad03605, 0xcdd70693, 0x54de5729, 0x23d967bf,
    0xb3667a2e, 0xc4614ab8, 0x5d681b02, 0x2a6f2b94, 0xb40bbe37, 0xc30c8ea1, 0x5a05df1b, 0x2d02ef8d,
];
const PASSWORD: [u8; 20] = [
    0x6F, 0x66, 0x4F, 0x31, 0x61, 0x30, 0x75, 0x65, 0x58, 0x41, 0x3F, 0x20, 0x5B, 0xFF, 0x73, 0x20,
    0x68, 0x20, 0x25, 0x3F,
];

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct IPFHeader {
    pub file_count: u16,
    pub file_table_pointer: u32,
    pub padding: u16,
    pub header_pointer: u32,
    #[br(assert(magic == MAGIC_NUMBER, "IPFHeader magic mismatch, file may be corrupted"))]
    pub magic: u32,
    pub version_to_patch: u32,
    pub new_version: u32,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct IPFFileTable {
    pub directory_name_length: u16,
    pub crc32: u32,
    pub file_size_compressed: u32,
    pub file_size_uncompressed: u32,
    pub file_pointer: u32,
    pub container_name_length: u16,
    #[br(count = container_name_length, try_map = String::from_utf8)]
    pub container_name: String,

    #[br(count = directory_name_length, try_map = String::from_utf8)]
    pub directory_name: String,

    #[brw(ignore)]
    pub file_path: Option<PathBuf>,
}

impl IPFFileTable {
    /// Check if the file should not be decompressed based on extension
    fn should_skip_decompression(&self) -> bool {
        let ignored_exts = [".fsb", ".jpg", ".mp3"];
        self.directory_name
            .rsplit('.')
            .next()
            .map(|ext| format!(".{}", ext.to_ascii_lowercase()))
            .map_or(false, |ext| ignored_exts.contains(&ext.as_str()))
    }

    pub fn extract_data(&self) -> io::Result<Vec<u8>> {
        let path = self.file_path.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "file_path not set for this IPF entry")
        })?;

        let mut file = File::open(path)?;

        // Seek to the file's data
        file.seek(SeekFrom::Start(self.file_pointer as u64))?;

        // Read the raw compressed/encrypted bytes
        let mut buffer = vec![0u8; self.file_size_compressed as usize];
        file.read_exact(&mut buffer)?;

        // Decrypt and optionally decompress
        if !self.should_skip_decompression() {
            self.decrypt_in_place(&mut buffer);
            buffer = self.decompress_data(&buffer)?;
        }

        Ok(buffer)
    }

    /// Decrypt buffer in place using IPF decryption algorithm
    fn decrypt_in_place(&self, buffer: &mut [u8]) {
        if buffer.is_empty() {
            return;
        }

        let mut keys = self.generate_keys();
        let steps = (buffer.len() - 1) / 2 + 1;

        for i in 0..steps {
            let v = (keys[2] & 0xFFFD) | 2;
            let idx = i * 2;
            if idx < buffer.len() {
                buffer[idx] ^= ((v.wrapping_mul(v ^ 1)) >> 8) as u8;
                self.update_keys(&mut keys, buffer[idx]);
            }
        }
    }

    /// Decompress zlib/deflate data
    fn decompress_data(&self, data: &[u8]) -> io::Result<Vec<u8>> {
        let mut output = Vec::with_capacity(self.file_size_uncompressed as usize);
        flate2::Decompress::new(false)
            .decompress_vec(data, &mut output, flate2::FlushDecompress::Finish)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Failed to decompress"))?;
        Ok(output)
    }

    /// Compute CRC32 for key update
    fn compute_crc32(&self, crc: u32, b: u8) -> u32 {
        CRC32_TABLE[((crc ^ b as u32) & 0xFF) as usize] ^ (crc >> 8)
    }

    /// Extract byte at a given position from u32 value
    fn extract_byte_at(&self, value: u32, byte_index: usize) -> u8 {
        (value >> (byte_index * 8)) as u8
    }

    /// Update decryption keys using a single byte
    fn update_keys(&self, keys: &mut [u32; 3], byte: u8) {
        keys[0] = self.compute_crc32(keys[0], byte);
        keys[1] = 0x8088405u32
            .wrapping_mul((keys[0] as u8 as u32).wrapping_add(keys[1]))
            .wrapping_add(1);
        keys[2] = self.compute_crc32(keys[2], self.extract_byte_at(keys[1], 3));
    }

    /// Generate initial decryption keys from PASSWORD
    fn generate_keys(&self) -> [u32; 3] {
        let mut keys = [0x12345678, 0x23456789, 0x34567890];
        for &b in PASSWORD.iter() {
            self.update_keys(&mut keys, b);
        }
        keys
    }
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct IPFRoot {
    #[br(seek_before = SeekFrom::End(HEADER_LOCATION))]
    pub header: IPFHeader,

    #[br(seek_before = SeekFrom::Start(header.file_table_pointer as u64))]
    #[br(count = header.file_count)]
    pub file_table: Vec<IPFFileTable>,
}

impl IPFRoot {
    /// Read IPFRoot from a file path, accepting &str or &Path
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path_ref = path.as_ref();
        let file = File::open(path_ref)?;
        let mut reader = BufReader::new(file);

        let mut root: IPFRoot = reader
            .read_le()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("binrw error: {}", e)))?;

        for f in &mut root.file_table {
            f.file_path = Some(path_ref.to_path_buf());

            // Prepend container_name to directory_name if not already present
            let container_stem = Path::new(&f.container_name)
                .file_stem()
                .unwrap()
                .to_string_lossy();

            f.directory_name = format!("{}/{}", container_stem, f.directory_name);
        }

        Ok(root)
    }
}

pub fn parse_all_ipf_files_limited_threads(
    dir: &Path,
    max_threads: usize,
) -> io::Result<Vec<IPFRoot>> {
    let ipf_paths: Vec<PathBuf> = read_dir(dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "ipf"))
        .collect();

    let (tx_paths, rx_paths) = mpsc::channel::<PathBuf>();
    let rx_paths = Arc::new(Mutex::new(rx_paths));
    let (tx_results, rx_results) = mpsc::channel::<io::Result<IPFRoot>>();

    for path in ipf_paths {
        tx_paths.send(path).unwrap();
    }
    drop(tx_paths); // signal no more tasks

    let mut handles = Vec::new();
    for _ in 0..max_threads {
        let rx_paths = Arc::clone(&rx_paths);
        let tx_results = tx_results.clone();

        let handle = thread::spawn(move || {
            while let Ok(path) = rx_paths.lock().unwrap().recv() {
                let res = IPFRoot::from_file(&path).map(|root| root);
                let _ = tx_results.send(res);
            }
        });
        handles.push(handle);
    }

    drop(tx_results); // optional: close sender so iterator will end

    // Collect all results
    let mut results = Vec::new();
    for res in rx_results.iter() {
        results.push(res?);
    }

    for handle in handles {
        handle.join().expect("Worker thread panicked");
    }

    Ok(results)
}

pub fn parse_game_folders_multithread_limited(
    game_root: &Path,
    max_threads: usize,
) -> io::Result<Vec<IPFRoot>> {
    let data_dir = game_root.join("data");
    let patch_dir = game_root.join("patch");

    let handle_data = {
        let data_dir = data_dir.clone();
        thread::spawn(move || parse_all_ipf_files_limited_threads(&data_dir, max_threads))
    };
    let handle_patch = {
        let patch_dir = patch_dir.clone();
        thread::spawn(move || parse_all_ipf_files_limited_threads(&patch_dir, max_threads))
    };

    let mut all_parsed = handle_data.join().unwrap()?;
    all_parsed.extend(handle_patch.join().unwrap()?);

    Ok(all_parsed)
}

pub fn parse_game_ipfs(game_root: &Path) -> io::Result<Vec<IPFRoot>> {
    parse_game_folders_multithread_limited(game_root, 4)
}

pub fn collect_file_tables_from_parsed(parsed_ipfs: &mut Vec<IPFRoot>) -> Vec<IPFFileTable> {
    let mut all_file_table = Vec::new();

    for ipf_root in parsed_ipfs.iter_mut() {
        // Move file_table out of each IPFRoot, leaving it empty
        let file_table = std::mem::take(&mut ipf_root.file_table);
        all_file_table.extend(file_table);
    }

    all_file_table
}

/// Sorts IPF files: folder first, then human-friendly filename order
pub fn sort_file_tables_by_folder_then_name(file_tables: &mut Vec<IPFFileTable>) {
    file_tables.sort_by(|a, b| {
        let path_a = a.file_path.as_ref().unwrap();
        let path_b = b.file_path.as_ref().unwrap();

        // Compare folders first
        let folder_ord = path_a.parent().unwrap().cmp(path_b.parent().unwrap());
        if folder_ord != Ordering::Equal {
            return folder_ord;
        }

        // Then compare normalized filenames
        let name_a = path_a.file_name().unwrap().to_str().unwrap();
        let name_b = path_b.file_name().unwrap().to_str().unwrap();

        name_a.cmp(&name_b)
    });
}

/// Group IPFFileTable by directory name, moving the items
pub fn group_file_tables_by_directory(
    file_tables: Vec<IPFFileTable>,
) -> BTreeMap<String, Vec<IPFFileTable>> {
    let mut map: BTreeMap<String, Vec<IPFFileTable>> = BTreeMap::new();

    for file in file_tables {
        map.entry(file.directory_name.clone()) // use directory_name as key
            .or_default()
            .push(file); // move file into the Vec
    }

    map
}

pub fn print_hex_viewer(data: &[u8]) {
    const BYTES_PER_LINE: usize = 16;

    for (i, chunk) in data.chunks(BYTES_PER_LINE).enumerate() {
        // Offset decimal (8 digits padded)
        print!("{:08}  ", i * BYTES_PER_LINE);

        // Hex bytes uppercase
        for b in chunk.iter() {
            print!("{:02X} ", b);
        }

        // Pad hex if last line shorter
        let pad_spaces = (BYTES_PER_LINE - chunk.len()) * 3;
        for _ in 0..pad_spaces {
            print!(" ");
        }

        // ASCII chars or '.' if non-printable
        print!(" ");
        for &b in chunk.iter() {
            let c = if b.is_ascii_graphic() || b == b' ' {
                b as char
            } else {
                '.'
            };
            print!("{}", c);
        }

        println!();
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FileSizeStats {
    pub count_duplicated: u32,
    pub count_unique: u32,
    pub compressed_lowest: u32,
    pub compressed_highest: u32,
    pub compressed_avg: u32,
    pub uncompressed_lowest: u32,
    pub uncompressed_highest: u32,
    pub uncompressed_avg: u32,
}

pub fn compute_ipf_file_stats(ipfs: &[IPFRoot]) -> FileSizeStats {
    let mut count_duplicated = 0u32;
    let mut count_unique = 0u32;
    let mut compressed_sum = 0u64;
    let mut uncompressed_sum = 0u64;
    let mut compressed_lowest = u32::MAX;
    let mut compressed_highest = 0u32;
    let mut uncompressed_lowest = u32::MAX;
    let mut uncompressed_highest = 0u32;

    for ipf in ipfs {
        for file in &ipf.file_table {
            count_duplicated += 1;
            compressed_sum += file.file_size_compressed as u64;
            uncompressed_sum += file.file_size_uncompressed as u64;

            compressed_lowest = compressed_lowest.min(file.file_size_compressed);
            compressed_highest = compressed_highest.max(file.file_size_compressed);

            uncompressed_lowest = uncompressed_lowest.min(file.file_size_uncompressed);
            uncompressed_highest = uncompressed_highest.max(file.file_size_uncompressed);
        }
    }

    if count_duplicated == 0 {
        return FileSizeStats {
            count_duplicated: 0,
            count_unique: 0,
            compressed_lowest: 0,
            compressed_highest: 0,
            compressed_avg: 0,
            uncompressed_lowest: 0,
            uncompressed_highest: 0,
            uncompressed_avg: 0,
        };
    }

    FileSizeStats {
        count_duplicated,
        count_unique,
        compressed_lowest,
        compressed_highest,
        compressed_avg: (compressed_sum / count_duplicated as u64) as u32,
        uncompressed_lowest,
        uncompressed_highest,
        uncompressed_avg: (uncompressed_sum / count_duplicated as u64) as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_read_ipf_root() -> io::Result<()> {
        // Path to your test IPF file
        let path = "tests/379124_001001.ipf";

        // Read IPFRoot from file
        let root = IPFRoot::from_file(path)?;

        // Print for debugging (optional)
        println!("Header: {:#?}", root.header);

        // Basic assertions
        assert!(
            !root.file_table.is_empty(),
            "File table should not be empty"
        );

        Ok(())
    }

    #[test]
    fn test_ipf_file_index_37_is_valid_utf8() -> io::Result<()> {
        // Read IPFRoot from file
        let root = IPFRoot::from_file("tests/379124_001001.ipf")?;

        // Ensure file table is not empty
        assert!(
            !root.file_table.is_empty(),
            "File table should not be empty"
        );

        // Choose the index to test
        let index = 37;

        if let Some(file_entry) = root.file_table.get(index) {
            // Extract data from the file
            let result_data = file_entry.extract_data()?;

            // Assert that the extracted data is valid UTF-8
            assert!(
                String::from_utf8(result_data).is_ok(),
                "File at index {} is not valid UTF-8",
                index
            );
        } else {
            panic!("File table index {} does not exist", index);
        }

        Ok(())
    }
}
