# IPF Structure and Parsing

IPF is a file format used for packaging various types of data in games. This document explains the binary
structure of IPF files and how they are parsed using the provided Rust code.

## IPF Header

An IPF file begins with a header containing metadata and information about the file's structure.

### Header Fields

- **File Count (2 bytes):** The total number of files or entries contained within the IPF file.

- **File Table Pointer (4 bytes):** The offset to the start of the file table in the IPF file.

- **Padding (2 bytes):** Reserved for alignment.

- **Footer Pointer (4 bytes):** The offset to the start of the footer within the IPF file.

- **Magic (4 bytes):** A unique identifier or magic number that indicates the file type. Typically, `PK\x05\x06` for IPF
  files.

- **Version to Patch (4 bytes):** A version number to identify the file format.

- **New Version (4 bytes):** Another version number to indicate the IPF file format.

### Header Validation

The magic number "PK\x05\x06" is a common identifier for IPF files. It is used to confirm that a file is indeed an IPF
file.

## File Table

The file table follows the header and contains information about individual files or entries within the IPF file.

### File Table Entry Components

Each file table entry consists of:

- **Filename Length (2 bytes):** The length of the filename for this entry.

- **CRC32 (4 bytes):** A checksum value (usually in hexadecimal) to verify the integrity of the file data.

- **File Size Compressed (4 bytes):** The size of the file when it's compressed.

- **File Size Uncompressed (4 bytes):** The size of the file when it's uncompressed.

- **File Pointer (4 bytes):** The offset to the start of the file data within the IPF file.

- **Container Name Length (2 bytes):** The length of the container name.

- **Container Name (string):** The name of the container where the file is located.

- **Directory Name (string):** The name of the directory containing the file.

- **Filename (string):** The name of the file.

## IPF Decryption and Decompression

The provided Rust code includes functions to decrypt and decompress the data within the IPF file. Decryption is
performed if the version number is greater than 11000 or equal to 0.

- **IPF Decryption:** The code uses an external C library to decrypt the data.

- **IPF Decompression:** The code checks if the file size after decompression is smaller than or equal to the size of
  the compressed data. If not, it decompresses the data using the zlib-based decompression algorithm.

## Data Parsing

The code also includes a function to parse and handle the data based on its file extension. Depending on the file type,
different actions are taken:

- **XAC and XSM Files:** The data is parsed and printed as structured data.

- **IES Files:** The data is parsed and printed as structured data.

- **FSB Files:** These are handled differently based on the code's logic.

- **Text Files:** The data is printed as a string if the file extension indicates a text file.

- **Image Files:** Image data is detected but not processed further.

- **Unknown Files:** Files with unrecognized extensions are identified as unknown.

This document provides an overview of the binary structure of IPF files and how they are processed in the provided Rust
code. Specific details and the structure of the file may vary based on the application or game that uses them.

Please note that additional details about specific file types and their structures can be found in the documentation for
the relevant software or application.
