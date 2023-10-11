#![allow(dead_code)]

use std::io::{Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::ies::ies_enum::IesColumnType;
use crate::ies::ies_struct::{IesColumn, IesFile, IesRow};
use crate::ies::ies_util::decrypt_string;

const HEADER_NAME: usize = 128;
const DATA_NAME: usize = 64;
pub fn ies_parse<R: Read + Seek>(ies_file: &mut R) -> IesFile {
    let mut ies_data = IesFile::default();
    read_header(ies_file, &mut ies_data);
    read_columns(ies_file, &mut ies_data);
    read_rows(ies_file, &mut ies_data);
    ies_data
}

fn read_header<'a, R: Read + Seek>(file: &'a mut R, ies: &'a mut IesFile) -> &'a mut IesFile {
    let mut name = [0; HEADER_NAME];
    file.read_exact(&mut name).unwrap();
    ies.header.name = std::str::from_utf8(&name)
        .unwrap()
        .trim_end_matches(char::from(0))
        .to_string();

    file.read_u32::<LittleEndian>().unwrap(); //Padding
    ies.header.data_offset = file.read_u32::<LittleEndian>().unwrap();
    ies.header.resource_offset = file.read_u32::<LittleEndian>().unwrap();
    ies.header.file_size = file.read_u32::<LittleEndian>().unwrap();
    file.read_u16::<LittleEndian>().unwrap(); //Padding
    ies.header.row_count = file.read_u16::<LittleEndian>().unwrap();
    ies.header.column_count = file.read_u16::<LittleEndian>().unwrap();
    ies.header.number_column_count = file.read_u16::<LittleEndian>().unwrap();
    ies.header.string_column_count = file.read_u16::<LittleEndian>().unwrap();
    file.read_u16::<LittleEndian>().unwrap(); //Padding
    ies
}

fn read_columns<'a, R: Read + Seek>(file: &'a mut R, ies: &'a mut IesFile) -> &'a mut IesFile {
    file.seek(SeekFrom::End(
        -((ies.header.resource_offset as i64) + (ies.header.data_offset as i64)),
    ))
    .unwrap();
    for _ in 0..ies.header.column_count {
        let mut column = IesColumn::default();

        let mut name = [0u8; DATA_NAME];
        file.read_exact(&mut name).unwrap();
        column.name = decrypt_string(&name);

        let mut name_second = [0u8; DATA_NAME];
        file.read_exact(&mut name_second).unwrap();
        column.name_second = decrypt_string(&name_second);
        let num = file.read_u16::<LittleEndian>().unwrap();
        column.column_type = match num {
            0 => IesColumnType::Float,
            1 => IesColumnType::String,
            2 => IesColumnType::StringSecond,
            _ => panic!("Invalid column type"),
        };
        file.read_u32::<LittleEndian>().unwrap();
        column.position = file.read_u16::<LittleEndian>().unwrap();
        ies.columns.push(column);
    }
    ies.columns.sort();
    ies
}

fn read_rows<'a, R: Read + Seek>(file: &'a mut R, ies: &'a mut IesFile) -> &'a mut IesFile {
    file.seek(SeekFrom::End(-(ies.header.resource_offset as i64)))
        .unwrap();

    for _ in 0..ies.header.row_count {
        file.read_u32::<LittleEndian>().unwrap(); //Padding

        let count = file.read_u16::<LittleEndian>().unwrap();
        let mut buffer = vec![0; count as usize];
        file.read_exact(&mut buffer).unwrap();

        let mut row = Vec::with_capacity(ies.header.row_count as usize);
        for (_, column) in ies.columns.iter().enumerate() {
            let value = if column.column_type == IesColumnType::Float {
                let nan = file.read_f32::<LittleEndian>().unwrap();
                let max_value = f32::from_bits(u32::MAX);
                if (nan - max_value).abs() < f32::EPSILON {
                    IesRow {
                        value_float: Some(max_value),
                        value_int: None,
                        value_string: None,
                    }
                } else {
                    IesRow {
                        value_float: None,
                        value_int: Some(nan as u32),
                        value_string: None,
                    }
                }
            } else {
                let length = file.read_u16::<LittleEndian>().unwrap();
                let mut string_buffer = vec![0u8; length as usize];
                file.read_exact(&mut string_buffer).unwrap();
                let string_value = decrypt_string(&string_buffer);
                IesRow {
                    value_float: None,
                    value_int: None,
                    value_string: Some(string_value),
                }
            };
            row.push(value);
        }

        ies.rows.push(row);
        file.seek(SeekFrom::Current(ies.header.string_column_count as i64))
            .unwrap();
    }
    ies
}
