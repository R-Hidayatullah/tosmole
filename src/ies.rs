use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader, Cursor},
    path::Path,
};

use binrw::{BinReaderExt, binread};
use serde::{Deserialize, Serialize};

const XOR_KEY: u8 = 1;

fn decrypt_bytes_to_string(encrypted_bytes: &[u8]) -> String {
    let decrypted_bytes: Vec<u8> = encrypted_bytes.iter().map(|&b| b ^ XOR_KEY).collect();

    let s = String::from_utf8_lossy(&decrypted_bytes);

    s.trim_end_matches(|c: char| !c.is_ascii_graphic() && !c.is_ascii_whitespace())
        .to_string()
}

fn trim_padding(padded_bytes: &[u8]) -> String {
    let s = String::from_utf8_lossy(&padded_bytes);

    s.trim_end_matches(|c: char| !c.is_ascii_graphic() && !c.is_ascii_whitespace())
        .to_string()
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct IESColumn {
    #[br(count = 64)]
    #[br(map = |bytes: Vec<u8>| decrypt_bytes_to_string(&bytes))]
    pub column: String,
    #[br(count = 64)]
    #[br(map = |bytes: Vec<u8>| decrypt_bytes_to_string(&bytes))]
    pub name: String,
    pub type_data: u16,
    pub access_data: u16,
    pub sync_data: u16,
    pub decl_idx: u16,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct IESRowText {
    pub text_length: u16,
    #[br(count = text_length)]
    #[br(map = |bytes: Vec<u8>| decrypt_bytes_to_string(&bytes))]
    pub text_data: String,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct IESRowFloat {
    pub float_data: f32,
}

#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(import(num_column_number:u16,num_column_string:u16))]
#[br(little)]
pub struct IESColumnData {
    pub index_data: i32,
    pub row_text: IESRowText,
    #[br(count =  num_column_number)]
    pub floats: Vec<IESRowFloat>,
    #[br(count =  num_column_string)]
    pub texts: Vec<IESRowText>,
    #[br(count =  num_column_string)]
    pub padding: Vec<i8>,
}

/// Metadata section of an IES file (header only, no columns/data)
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct IESHeader {
    #[br(count = 64)]
    #[br(map = |bytes: Vec<u8>| trim_padding(&bytes))]
    pub idspace: String,
    #[br(count = 64)]
    #[br(map = |bytes: Vec<u8>| trim_padding(&bytes))]
    pub keyspace: String,
    pub version: u16,
    pub padding: u16,
    pub info_size: u32,
    pub data_size: u32,
    pub total_size: u32,
    pub use_class_id: u8,
    pub padding2: u8,
    pub num_field: u16,
    pub num_column: u16,
    pub num_column_number: u16,
    pub num_column_string: u16,
    pub padding3: u16,
}

/// Full IES file contents (root structure)
#[binread]
#[derive(Default, Debug, Serialize, Deserialize)]
#[br(little)]
pub struct IESRoot {
    pub header: IESHeader,
    #[br(count = header.num_column)]
    pub columns: Vec<IESColumn>,
    #[br(args { inner: (header.num_column_number,header.num_column_string) })]
    #[br(count = header.num_field)]
    pub data: Vec<IESColumnData>,
}

impl IESRoot {
    /// Read IESRoot from a file path, accepting &str or &Path
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path_ref = path.as_ref();
        let file = File::open(path_ref)?;
        let mut reader = BufReader::new(file);

        let root: IESRoot = reader
            .read_le()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("binrw error: {}", e)))?;

        Ok(root)
    }

    /// Read IESRoot from a byte slice in memory
    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        let mut cursor = Cursor::new(bytes);

        let root: IESRoot = cursor
            .read_le()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("binrw error: {}", e)))?;

        Ok(root)
    }

    /// Extract Mesh -> Path mapping from this IESRoot
    pub fn extract_mesh_path_map(&self) -> HashMap<String, String> {
        // Step 1: Sort columns by decl_idx then type_data
        let mut columns_sorted: Vec<(usize, &IESColumn)> =
            self.columns.iter().enumerate().collect();
        columns_sorted.sort_by(|a, b| {
            a.1.decl_idx
                .cmp(&b.1.decl_idx)
                .then(a.1.type_data.cmp(&b.1.type_data))
        });

        // Step 2: Find indices of "Mesh" and "Path" in sorted columns
        let mut mesh_idx: Option<usize> = None;
        let mut path_idx: Option<usize> = None;

        for (i, (_original_idx, col)) in columns_sorted.iter().enumerate() {
            match col.column.as_str() {
                "Mesh" => mesh_idx = Some(i - 1),
                "Path" => path_idx = Some(i - 1),
                _ => {}
            }
        }

        let mesh_idx = match mesh_idx {
            Some(i) => i,
            None => return HashMap::new(),
        };
        let path_idx = match path_idx {
            Some(i) => i,
            None => return HashMap::new(),
        };

        // Step 3: Extract data using these indices
        let mut map = HashMap::new();
        for row in &self.data {
            let mesh_name = row
                .texts
                .get(mesh_idx)
                .map(|t| t.text_data.replace('\\', "/"))
                .unwrap_or_default();

            let path = row
                .texts
                .get(path_idx)
                .map(|t| t.text_data.replace('\\', "/"))
                .unwrap_or_default();

            if !mesh_name.is_empty() && !path.is_empty() {
                map.insert(mesh_name.to_lowercase(), path);
            }
        }

        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_read_ies_root() -> io::Result<()> {
        // Path to your test IES file
        let path = "tests/cell.ies";

        // Read IESRoot from file
        let root = IESRoot::from_file(path)?;

        // Print for debugging (optional)
        println!("Header: {:#?}", root.header);
        println!("Columns: {:#?}", root.columns);
        println!("Data: {:#?}", root.data);

        // Basic assertions
        assert!(!root.columns.is_empty(), "Columns should not be empty");
        assert!(!root.data.is_empty(), "Data rows should not be empty");
        assert!(
            !root.header.idspace.is_empty(),
            "Header idspace should not be empty"
        );

        Ok(())
    }

    #[test]
    fn test_read_ies_from_memory() -> io::Result<()> {
        // Load file into memory first
        let data = std::fs::read("tests/cell.ies")?;

        // Parse from memory instead of directly from file
        let root = IESRoot::from_bytes(&data)?;

        println!("Header: {:?}", root.header);
        assert!(!root.columns.is_empty());
        assert!(!root.data.is_empty());

        Ok(())
    }
}
