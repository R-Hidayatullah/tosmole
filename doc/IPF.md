---

# IPF Archive File Parser and Extractor (Rust)

This document explains the Rust implementation for parsing and extracting files from an **IPF archive** format, which includes header reading, file table parsing, decryption, and decompression of files.

---

## Constants

```rust
const HEADER_LOCATION: i64 = -24;
const MAGIC_NUMBER: u32 = 0x06054B50;
// Standard CRC32 lookup table used for key updates in decryption
const CRC32_TABLE: [u32; 256] = [
    0x00000000, 0x77073096, 0xEE0E612C, 0x990951BA, 0x076DC419, 0x706AF48F, 0xE963A535, 0x9E6495A3,
    0x0EDB8832, 0x79DCB8A4, 0xE0D5E91E, 0x97D2D988, 0x09B64C2B, 0x7EB17CBD, 0xE7B82D07, 0x90BF1D91,
    // ... fill the remaining 240 values here ...
    // You can find the full table online or generate it with a CRC32 function.
];
const PASSWORD: [u8; 20] = [
    0x6F, 0x66, 0x4F, 0x31, 0x61, 0x30, 0x75, 0x65, 0x58, 0x41,
    0x3F, 0x20, 0x5B, 0xFF, 0x73, 0x20, 0x68, 0x20, 0x25, 0x3F,
];
```

* **HEADER\_LOCATION**: Offset from the end of the file where the IPF header starts (24 bytes backward).
* **CRC32_TABLE** is a precomputed lookup table for CRC32 checksum calculation.
* **MAGIC\_NUMBER**: Expected magic number for IPF file validation.
* **PASSWORD**: 20-byte key used in decryption algorithm of file contents.

---

## Data Structures

### `IPFHeader`

Metadata for the IPF archive.

| Field                | Type | Description                      |
| -------------------- | ---- | -------------------------------- |
| `file_count`         | u16  | Number of files in archive       |
| `file_table_pointer` | u32  | Offset to the file table section |
| `padding`            | u16  | Padding                          |
| `header_pointer`     | u32  | Offset to header (redundant?)    |
| `magic`              | u32  | Magic number to validate format  |
| `version_to_patch`   | u32  | Version info                     |
| `new_version`        | u32  | New version info                 |

* Read from file by seeking 24 bytes before EOF.
* Validates magic number; errors out if invalid.

---

### `IPFFileTable`

Represents an entry in the IPF archive's file table describing each contained file.

| Field                    | Type   | Description                              |
| ------------------------ | ------ | ---------------------------------------- |
| `directory_name_length`  | u16    | Length of the directory name string      |
| `crc32`                  | u32    | CRC32 checksum of the file               |
| `file_size_compressed`   | u32    | Compressed file size in bytes            |
| `file_size_uncompressed` | u32    | Uncompressed file size in bytes          |
| `file_pointer`           | u32    | Offset in archive where file data starts |
| `container_name_length`  | u16    | Length of the container name string      |
| `container_name`         | String | Container filename                       |
| `directory_name`         | String | Directory path or name                   |

* Contains metadata needed to locate and extract each file.
* Strings (`container_name`, `directory_name`) are read as UTF-8 from the archive.

---

### `IPFRoot`

Root structure representing the entire IPF archive contents.

| Field        | Type                         | Description                                |
| ------------ | ---------------------------- | ------------------------------------------ |
| `header`     | `IPFHeader`                  | Archive header metadata                    |
| `file_table` | `Vec<IPFFileTable>`          | Vector of file table entries               |
| `filepath`   | `Option<PathBuf>`            | Optional path to source archive file       |
| `reader`     | `Option<BinaryReader<File>>` | Internal file reader to support extraction |

* Manages archive state and file extraction.
* Keeps an internal reader for direct file extraction if opened from a file.

---

## Implementation Details

### Reading Header: `IPFHeader::read_from`

* Seeks to 24 bytes before the end of the file.
* Reads the header fields in order.
* Validates the magic number against `MAGIC_NUMBER`.
* Seeks to the file table offset for subsequent reading.

### Reading File Table Entry: `IPFFileTable::read_from`

* Reads fixed-size fields describing the file.
* Reads variable length container and directory names as UTF-8 strings.
* Returns fully populated `IPFFileTable` struct.

### Extracting File Contents: `IPFFileTable::extract`

1. Seeks to the file data pointer.
2. Reads compressed and encrypted file bytes.
3. Decrypts the bytes using a custom XOR-based scheme with rolling keys derived from `PASSWORD`.
4. Decompresses data using DEFLATE (via `flate2` crate).
5. Returns decompressed raw file data bytes.

### Decryption Algorithm

* Uses three rolling keys initialized with constants.
* Updates keys byte-wise using CRC32 computation and multiplication.
* Decrypts every other byte of the file data buffer by XORing with a derived key value.
* The `PASSWORD` byte array is used to update keys initially.

---

## IPFRoot Methods

### `open`

* Opens an IPF archive file from a path.
* Reads the header and file table.
* Stores internal `BinaryReader<File>` for extraction support.

### `from_reader`

* Reads IPF archive from any generic `Read + Seek` source (e.g., in-memory buffer).
* Reads header and file table.
* Does **not** keep internal reader, so extraction is not supported unless handled externally.

### `extract_file`

* Private method that extracts the file data for a given index.
* Requires internal `reader` to seek and read file contents.
* Returns decompressed raw bytes.

### `extract_file_if_available`

* Public extraction method returning an `Option`.
* Returns `Some(io::Result<Vec<u8>>)` if internal reader is available.
* Returns `None` if no reader is present (e.g., archive loaded from memory).

---

## Usage Summary

```rust
// Open IPF archive from file
let mut ipf = IPFRoot::open("archive.ipf")?;

// Extract first file data if possible
if let Some(Ok(file_data)) = ipf.extract_file_if_available(0) {
    // Use `file_data` bytes here...
}
```

---

## Notes

* File extraction depends on the presence of an internal file reader (`BinaryReader<File>`).
* If you load IPF archive from memory buffers (`from_reader`), extraction is not directly available.
* Decompression uses the `flate2` crate's DEFLATE algorithm.
* CRC32 table `CRC32_TABLE` is used internally in key updates (not shown here but assumed present).
* Careful with indexing and bounds checks to avoid invalid extraction.

---