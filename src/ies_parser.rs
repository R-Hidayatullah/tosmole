use std::{
    cmp::Ordering,
    fs::File,
    io::{self, BufReader, Cursor, Read, Seek, SeekFrom},
    path::Path,
};

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Ord, PartialOrd, PartialEq, Eq)]
enum IesColumnType {
    Float,
    String,
    StringSecond,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct IesFile {
    header: IesHeader,
    columns: Vec<IesColumn>,
    rows: Vec<Vec<IesRow>>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct IesHeader {
    data_offset: u32,
    resource_offset: u32,
    file_size: u32,
    name: String,
    column_count: u16,
    row_count: u16,
    number_column_count: u16,
    string_column_count: u16,
}

#[derive(Debug, Serialize, Deserialize, Eq)]
struct IesColumn {
    name: String,
    name_second: String,
    column_type: IesColumnType,
    position: u16,
}

impl Default for IesColumn {
    fn default() -> Self {
        IesColumn {
            name: "".to_string(),
            name_second: "".to_string(),
            column_type: IesColumnType::Float,
            position: 0,
        }
    }
}
impl Ord for IesColumn {
    /// Implements ordering for `IesColumn` based on column type and position.
    /// This is used for sorting columns, making it easier to navigate when viewing data in tables.
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.column_type, &other.column_type) {
            (IesColumnType::Float, IesColumnType::Float)
            | (IesColumnType::String, IesColumnType::String)
            | (IesColumnType::StringSecond, IesColumnType::StringSecond) => {
                self.position.cmp(&other.position)
            }
            (IesColumnType::Float, _) => Ordering::Less,
            (_, IesColumnType::Float) => Ordering::Greater,
            (IesColumnType::String, IesColumnType::StringSecond) => Ordering::Less,
            (IesColumnType::StringSecond, IesColumnType::String) => Ordering::Greater,
        }
    }
}

impl PartialOrd for IesColumn {
    /// Implements partial ordering for `IesColumn` based on column type and position.
    /// This is used for sorting columns, making it easier to navigate when viewing data in tables.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for IesColumn {
    /// Implements equality comparison for `IesColumn` based on column type and position.
    /// This is used for sorting columns, making it easier to navigate when viewing data in tables.
    fn eq(&self, other: &Self) -> bool {
        self.column_type == other.column_type && self.position == other.position
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct IesRow {
    value_float: Option<f32>,
    value_int: Option<u32>,
    value_string: Option<String>,
}

const HEADER_NAME: usize = 128;
const DATA_NAME: usize = 64;

impl IesFile {
    /// Load data from a file specified by the file path.
    ///
    /// # Arguments
    ///
    /// * `file_path` - A path to the IES file to be loaded.
    ///
    /// # Returns
    ///
    /// A Result containing the parsed `IesFile` or an IO error if the file cannot be read.
    ///
    /// # Examples
    ///
    /// ```
    /// // Example usage of loading an IES file from a specified path
    /// let ies_data = IesFile::load_from_file("path/to/your/file.ies").unwrap();
    /// println!("{:?}", ies_data);
    /// ```
    pub fn load_from_file<P: AsRef<Path>>(file_path: P) -> io::Result<Self> {
        // Open the file and create a buffered reader.
        let file = std::fs::File::open(file_path)?;
        let mut buf_reader = BufReader::new(file);

        // Delegate to load_from_reader for further processing.
        Self::load_from_reader(&mut buf_reader)
    }

    /// Load data from a byte vector.
    ///
    /// # Arguments
    ///
    /// * `bytes` - A vector of bytes containing the IES file data.
    ///
    /// # Returns
    ///
    /// A Result containing the parsed `IesFile` or an IO error if the byte vector is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// // Example usage of loading an IES file from a byte vector
    /// let ies_data = IesFile::load_from_bytes(vec![/* your byte data here */]).unwrap();
    /// println!("{:?}", ies_data);
    /// ```
    pub fn load_from_bytes(mut bytes: Vec<u8>) -> io::Result<Self> {
        // Create a cursor from the byte vector.
        let mut cursor = Cursor::new(&mut bytes);

        // Delegate to load_from_reader for further processing.
        Self::load_from_reader(&mut cursor)
    }

    fn load_from_reader<R: Read + Seek>(reader: &mut R) -> io::Result<Self> {
        let mut ies_data = IesFile::default();
        ies_data.read_header(reader)?;
        ies_data.read_columns(reader)?;
        ies_data.read_rows(reader)?;
        Ok(ies_data)
    }
    fn read_header<R: Read + Seek>(&mut self, file: &mut R) -> io::Result<&mut Self> {
        // Read the header name from the binary data.
        // The header name is stored as a fixed-size array of bytes (HEADER_NAME).
        let mut name = [0; HEADER_NAME];
        file.read_exact(&mut name).unwrap();

        // Convert the byte array to a UTF-8 string, removing trailing null characters,
        // and assign it to the `name` field in the header structure.
        self.header.name = std::str::from_utf8(&name)
            .unwrap()
            .trim_end_matches(char::from(0))
            .to_string();

        file.read_u32::<LittleEndian>().unwrap(); //Padding
        self.header.data_offset = file.read_u32::<LittleEndian>().unwrap();
        self.header.resource_offset = file.read_u32::<LittleEndian>().unwrap();
        self.header.file_size = file.read_u32::<LittleEndian>().unwrap();
        file.read_u16::<LittleEndian>().unwrap(); //Padding
        self.header.row_count = file.read_u16::<LittleEndian>().unwrap();
        self.header.column_count = file.read_u16::<LittleEndian>().unwrap();
        self.header.number_column_count = file.read_u16::<LittleEndian>().unwrap();
        self.header.string_column_count = file.read_u16::<LittleEndian>().unwrap();
        file.read_u16::<LittleEndian>().unwrap(); //Padding
        Ok(self)
    }

    fn read_columns<R: Read + Seek>(&mut self, file: &mut R) -> io::Result<&mut Self> {
        file.seek(SeekFrom::End(
            -((self.header.resource_offset as i64) + (self.header.data_offset as i64)),
        ))
        .unwrap();
        for _ in 0..self.header.column_count {
            let mut column = IesColumn::default();

            // Read the column name from the binary data.
            // The column name is stored as a fixed-size array of bytes (DATA_NAME).
            let mut name = [0u8; DATA_NAME];
            file.read_exact(&mut name).unwrap();

            // Decrypt the byte array representing the column name and assign it to the `name` field in the column structure.
            column.name = Self::decrypt_string(&name)?;

            let mut name_second = [0u8; DATA_NAME];
            file.read_exact(&mut name_second).unwrap();
            column.name_second = Self::decrypt_string(&name_second)?;
            let num = file.read_u16::<LittleEndian>().unwrap();
            column.column_type = match num {
                0 => IesColumnType::Float,
                1 => IesColumnType::String,
                2 => IesColumnType::StringSecond,
                _ => panic!("Invalid column type"),
            };
            file.read_u32::<LittleEndian>().unwrap();
            column.position = file.read_u16::<LittleEndian>().unwrap();
            self.columns.push(column);
        }
        self.columns.sort();
        Ok(self)
    }

    fn read_rows<R: Read + Seek>(&mut self, file: &mut R) -> io::Result<&mut Self> {
        file.seek(SeekFrom::End(-(self.header.resource_offset as i64)))
            .unwrap();

        for _ in 0..self.header.row_count {
            file.read_u32::<LittleEndian>().unwrap(); //Padding

            let count = file.read_u16::<LittleEndian>().unwrap();
            let mut buffer = vec![0; count as usize];
            file.read_exact(&mut buffer).unwrap();

            let mut row = Vec::with_capacity(self.header.row_count as usize);
            for (_, column) in self.columns.iter().enumerate() {
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
                    let string_value = Self::decrypt_string(&string_buffer)?;
                    if !string_value.is_empty() {
                        IesRow {
                            value_float: None,
                            value_int: None,
                            value_string: Some(string_value),
                        }
                    } else {
                        IesRow {
                            value_float: None,
                            value_int: None,
                            value_string: None,
                        }
                    }
                };
                row.push(value);
            }

            self.rows.push(row);
            file.seek(SeekFrom::Current(self.header.string_column_count as i64))
                .unwrap();
        }
        Ok(self)
    }

    /// Decrypts a byte array using a simple XOR operation.
    /// The function applies a XOR operation using a predefined key (xor_key = 1) to each byte in the input data array.
    /// The decrypted byte array is then converted into a UTF-8 string, removing trailing null characters ('\u{1}'),
    /// and returning the resulting string.
    fn decrypt_string(data: &[u8]) -> io::Result<String> {
        let xor_key = 1;

        // Apply XOR operation to each byte in the input data array to decrypt it.
        let decrypted_data: Vec<u8> = data.iter().map(|&byte| byte ^ xor_key).collect();

        // Convert the decrypted byte array into a UTF-8 string.
        // Trim trailing null characters ('\u{1}') and return the resulting string.
        Ok(String::from_utf8(decrypted_data)
            .unwrap()
            .trim_end_matches('\u{1}')
            .to_string())
    }

    pub fn get_columns_length(&self) -> io::Result<usize> {
        Ok(self.columns.len())
    }
    pub fn get_rows_length(&self) -> io::Result<usize> {
        Ok(self.rows.len())
    }

    pub fn get_data_by_column_name_and_index(
        &self,
        column_name: &str,
        row_index: usize,
    ) -> Option<&IesRow> {
        if let Some(column_index) = self.get_column_index_by_name(column_name) {
            if row_index < self.rows.len() {
                Some(&self.rows[row_index][column_index])
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_column_index_by_name(&self, column_name: &str) -> Option<usize> {
        if let Some(index) = self.columns.iter().position(|col| col.name == column_name) {
            Some(index)
        } else {
            self.columns
                .iter()
                .position(|col| col.name_second == column_name)
        }
    }

    pub fn get_column_names(&self) -> Vec<&String> {
        self.columns.iter().map(|col| &col.name).collect()
    }
}

#[test]
fn test_ies_parser() {
    // Provide the path to the test IES file
    let file_path = "tests/cell.ies";

    // Read the content of the file
    let mut file_content = Vec::new();
    let mut file = File::open(&file_path).expect("Failed to open file");
    file.read_to_end(&mut file_content)
        .expect("Failed to read file content");

    // Parse the IES file
    let ies_data = IesFile::load_from_bytes(file_content).expect("Failed to parse IES file");

    // Add assertions based on the structure and content of your IesFile
    assert_eq!(
        ies_data
            .get_columns_length()
            .expect("Failed to get column length"),
        6
    );
    assert_eq!(
        ies_data
            .get_rows_length()
            .expect("Failed to get row length"),
        7
    );
}

#[test]
fn test_get_data_by_column_name_and_index() {
    // Provide the path to the test IES file
    let file_path = "tests/cell.ies";

    // Read the content of the file
    let mut file_content = Vec::new();
    let mut file = File::open(&file_path).expect("Failed to open file");
    file.read_to_end(&mut file_content)
        .expect("Failed to read file content");

    // Parse the IES file
    let ies_data = IesFile::load_from_bytes(file_content).expect("Failed to parse IES file");

    let data = ies_data
        .get_data_by_column_name_and_index("ClassName", 0)
        .expect("Failed to get data");

    assert_eq!(
        data.value_string
            .clone()
            .expect("Failed to get string value"),
        String::from("Flame")
    );
}
