---

# IES File Format Rust Parser Documentation

This document explains the Rust implementation of an IES file parser, including data structures, decryption logic, and parsing flow.

---

## Overview

The IES file format is a binary structured file containing metadata and tabular data with text and floating-point values. This Rust code reads and decrypts the file contents into structured Rust types.

* Uses XOR-based simple encryption/decryption with key `1` on text fields.
* Reads a fixed-size encrypted header, followed by column definitions and data rows.
* Supports a mixture of text columns and floating-point columns.
* Uses a `BinaryReader` with endian awareness for binary data reading.

---

## Constants

```rust
const XOR_KEY: u8 = 1;
```

* Key used for XOR decryption/encryption of string bytes.

---

## Decryption and Trimming Utilities

### `decrypt_bytes_to_string(encrypted_bytes: &[u8]) -> String`

* Decrypts a byte slice by XORing each byte with `XOR_KEY`.
* Converts the decrypted bytes into a UTF-8 string (lossy).
* Trims trailing characters that are **not** ASCII graphic or whitespace.
* Returns the cleaned string.

### `trim_padding(padded_bytes: &[u8]) -> String`

* Converts a byte slice directly to UTF-8 string (lossy).
* Trims trailing non-printable characters similar to `decrypt_bytes_to_string`.
* Used mainly for fixed-length header fields.

---

## Data Structures

### `IESHeader`

Represents the fixed-length metadata header of the IES file.

| Field               | Type   | Description                           |
| ------------------- | ------ | ------------------------------------- |
| `idspace`           | String | Encrypted string ID space (64 bytes)  |
| `keyspace`          | String | Encrypted string key space (64 bytes) |
| `version`           | u16    | Version number                        |
| `padding`           | u16    | Padding field                         |
| `info_size`         | u32    | Size of the info section              |
| `data_size`         | u32    | Size of the data section              |
| `total_size`        | u32    | Total size of the file                |
| `use_class_id`      | u8     | Flag or identifier byte               |
| `padding2`          | u8     | Padding byte                          |
| `num_field`         | u16    | Number of data rows (fields)          |
| `num_column`        | u16    | Number of columns                     |
| `num_column_number` | u16    | Number of floating-point columns      |
| `num_column_string` | u16    | Number of string columns              |
| `padding3`          | u16    | Additional padding                    |

* Read by `IESHeader::read_from()` method from the binary stream.
* Strings are read as fixed-length byte arrays, trimmed after decryption.

---

### `IESColumn`

Defines metadata about each column in the IES table.

| Field         | Type   | Description                   |
| ------------- | ------ | ----------------------------- |
| `column`      | String | Column identifier (encrypted) |
| `name`        | String | Column name (encrypted)       |
| `type_data`   | u16    | Data type code                |
| `access_data` | u16    | Access permissions or flags   |
| `sync_data`   | u16    | Synchronization flags         |
| `decl_idx`    | u16    | Declaration index             |

* Each column is read with two 64-byte encrypted strings, then 4 fields of `u16`.

---

### `IESRowText`

Represents a string text data cell in a row.

| Field         | Type   | Description             |
| ------------- | ------ | ----------------------- |
| `text_length` | u16    | Length of the text data |
| `text_data`   | String | Decrypted text content  |

---

### `IESRowFloat`

Represents a floating-point data cell in a row.

| Field        | Type | Description          |
| ------------ | ---- | -------------------- |
| `float_data` | f32  | Floating-point value |

---

### `IESColumnData`

Represents a data row containing mixed data types.

| Field        | Type             | Description                              |
| ------------ | ---------------- | ---------------------------------------- |
| `index_data` | i32              | Row index or ID                          |
| `row_text`   | IESRowText       | Primary text field for the row           |
| `floats`     | Vec<IESRowFloat> | List of floating-point values in the row |
| `texts`      | Vec<IESRowText>  | List of string columns in the row        |
| `padding`    | Vec<i8>          | Padding bytes (signed)                   |

---

### `IESRoot`

The root structure containing the full parsed file.

| Field     | Type               | Description        |
| --------- | ------------------ | ------------------ |
| `header`  | IESHeader          | Parsed file header |
| `columns` | Vec<IESColumn>     | List of columns    |
| `data`    | Vec<IESColumnData> | List of data rows  |

---

## Reading Process (`IESRoot::read_from`)

1. Read the header metadata (`IESHeader`).
2. Read each column:

   * Decrypt two 64-byte strings (`column` and `name`).
   * Read 4 `u16` fields.
3. For each data row (based on `num_field`):

   * Read 4 bytes as `i32` index.
   * Read first text field (length + encrypted bytes).
   * Read floating-point columns (`num_column_number` times).
   * Read string columns (`num_column_string` times: length + encrypted bytes).
   * Read padding bytes (`num_column_string` times).
4. Collect all rows into `data` vector.
5. Return the fully populated `IESRoot`.

---

## Testing

* Two test examples show reading an IES file:

  * Directly from file (`File` + `BinaryReader`).
  * From a memory buffer (`Cursor` + `BinaryReader`).
* Tests assert the `idspace` field is non-empty after trimming.

---

## Notes

* All string fields are XOR-encrypted with key `1` and require decryption.
* Strings are stored as fixed-length byte arrays (usually 64 bytes).
* Trailing padding or null bytes are trimmed after decryption.
* Floats and integers are read using little endian order.
* Padding bytes accompany string columns in data rows for alignment or metadata.

---