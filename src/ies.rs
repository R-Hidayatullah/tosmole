use crate::binary::{BinaryReader, Endian};
use std::{
    fs::File,
    io::{self, Cursor, Read, Seek},
};

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

#[derive(Debug)]
pub struct IESColumn {
    pub column: String,
    pub name: String,
    pub type_data: u16,
    pub access_data: u16,
    pub sync_data: u16,
    pub decl_idx: u16,
}

#[derive(Debug)]
pub struct IESRowText {
    pub text_length: u16,
    pub text_data: String,
}

#[derive(Debug)]
pub struct IESRowFloat {
    pub float_data: f32,
}

#[derive(Debug)]
pub struct IESColumnData {
    pub index_data: i32,
    pub row_text: IESRowText,
    pub floats: Vec<IESRowFloat>,
    pub texts: Vec<IESRowText>,
    pub padding: Vec<i8>,
}

/// Metadata section of an IES file (header only, no columns/data)
#[derive(Debug)]
pub struct IESHeader {
    pub idspace: String,
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

impl IESHeader {
    pub fn read_from<R: Read + Seek>(br: &mut BinaryReader<R>) -> io::Result<Self> {
        let idspace_encrypted = br.read_vec(64 as usize)?;
        let idspace = trim_padding(&idspace_encrypted);
        let keyspace_encrypted = br.read_vec(64 as usize)?;
        let keyspace = trim_padding(&keyspace_encrypted);

        Ok(Self {
            idspace: idspace,
            keyspace: keyspace,
            version: br.read_u16()?,
            padding: br.read_u16()?,
            info_size: br.read_u32()?,
            data_size: br.read_u32()?,
            total_size: br.read_u32()?,
            use_class_id: br.read_u8()?,
            padding2: br.read_u8()?,
            num_field: br.read_u16()?,
            num_column: br.read_u16()?,
            num_column_number: br.read_u16()?,
            num_column_string: br.read_u16()?,
            padding3: br.read_u16()?,
        })
    }
}

/// Full IES file contents (root structure)
#[derive(Debug)]
pub struct IESRoot {
    pub header: IESHeader,
    pub columns: Vec<IESColumn>,
    pub data: Vec<IESColumnData>,
}

impl IESRoot {
    pub fn read_from<R: Read + Seek>(br: &mut BinaryReader<R>) -> io::Result<Self> {
        let header = IESHeader::read_from(br)?;

        // Read columns
        let mut columns = Vec::with_capacity(header.num_column as usize);
        for _ in 0..header.num_column {
            let column_encrypted = br.read_vec(64 as usize)?;
            let column = decrypt_bytes_to_string(&column_encrypted);
            let name_encrypted = br.read_vec(64 as usize)?;
            let name = decrypt_bytes_to_string(&name_encrypted);

            columns.push(IESColumn {
                column,
                name,
                type_data: br.read_u16()?,
                access_data: br.read_u16()?,
                sync_data: br.read_u16()?,
                decl_idx: br.read_u16()?,
            });
        }

        // Read data rows
        let mut data = Vec::with_capacity(header.num_field as usize);
        for _ in 0..header.num_field {
            let index_data = {
                let buf = br.read_with_endian::<4>(Endian::Little)?;
                i32::from_le_bytes(buf)
            };

            // First text field
            let text_length = br.read_u16()?;
            let text_data_encrypted = br.read_vec(text_length as usize)?;
            let text_data = decrypt_bytes_to_string(&text_data_encrypted);
            let row_text = IESRowText {
                text_length,
                text_data,
            };

            // Floats
            let mut floats = Vec::with_capacity(header.num_column_number as usize);
            for _ in 0..header.num_column_number {
                floats.push(IESRowFloat {
                    float_data: br.read_f32()?,
                });
            }

            // Text columns
            let mut texts = Vec::with_capacity(header.num_column_string as usize);
            for _ in 0..header.num_column_string {
                let tl = br.read_u16()?;
                let td = br.read_vec(tl as usize)?;
                let text_data = decrypt_bytes_to_string(&td);

                texts.push(IESRowText {
                    text_length: tl,
                    text_data,
                });
            }

            // Padding bytes
            let mut padding_vec = Vec::with_capacity(header.num_column_string as usize);
            for _ in 0..header.num_column_string {
                padding_vec.push(br.read_u8()? as i8);
            }

            data.push(IESColumnData {
                index_data,
                row_text,
                floats,
                texts,
                padding: padding_vec,
            });
        }

        Ok(Self {
            header,
            columns,
            data,
        })
    }
}

#[test]
fn test_read_cell_ies() -> io::Result<()> {
    let path = "tests/cell.ies";

    // Example 1 — Read directly from file
    {
        let file = File::open(path)?;
        let mut reader = BinaryReader::new(file, Endian::Little);
        let root = IESRoot::read_from(&mut reader)?;
        println!(
            "File IDSpace: {}",
            root.header.idspace.trim_end_matches('\0')
        );

        // You can add real assertions here
        assert!(!root.header.idspace.trim_end_matches('\0').is_empty());
    }

    // Example 2 — Read from memory buffer
    {
        let bytes = std::fs::read(path)?;
        let cursor = Cursor::new(bytes);
        let mut reader = BinaryReader::new(cursor, Endian::Little);
        let root = IESRoot::read_from(&mut reader)?;
        println!("Memory IES: {:?}", root.header);

        // Same assertion
        assert!(!root.header.idspace.trim_end_matches('\0').is_empty());
    }

    Ok(())
}
