use binrw::{BinRead, BinReaderExt, BinWrite, BinWriterExt, binrw};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::path::PathBuf;
use std::process;

const XOR_KEY: u8 = 1;

fn xor_encrypt_decrypt_string(s: &str) -> String {
    let encrypted_bytes: Vec<u8> = s.bytes().map(|b| b ^ XOR_KEY).collect();
    // Convert back to string, handling potential invalid UTF-8
    String::from_utf8_lossy(&encrypted_bytes).into_owned()
}

fn decrypt_bytes_to_string(encrypted_bytes: &[u8]) -> String {
    let decrypted_bytes: Vec<u8> = encrypted_bytes.iter().map(|&b| b ^ XOR_KEY).collect();

    let s = String::from_utf8_lossy(&decrypted_bytes);

    s.trim_end_matches(|c: char| !c.is_ascii_graphic() && !c.is_ascii_whitespace())
     .to_string()
}


#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct IESColumn {
    #[br(count = 64)]
    #[bw(pad_size_to = 64)]
    pub column: Vec<u8>,

    #[br(count = 64)]
    #[bw(pad_size_to = 64)]
    pub name: Vec<u8>,

    pub type_data: u16,
    pub access_data: u16,
    pub sync_data: u16,
    pub decl_idx: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IESColumnSerializable {
    pub column_name: String,
    pub name: String,
    pub type_data: u16,
    pub access_data: u16,
    pub sync_data: u16,
    pub decl_idx: u16,
}

impl IESColumn {
    pub fn column_str(&self) -> String {
        decrypt_bytes_to_string(&self.column)
    }

    pub fn name_str(&self) -> String {
        decrypt_bytes_to_string(&self.name)
    }

    pub fn new(
        column: &str,
        name: &str,
        type_data: u16,
        access_data: u16,
        sync_data: u16,
        decl_idx: u16,
    ) -> Self {
        let mut column_bytes = vec![0u8; 64];
        let encrypted_column = xor_encrypt_decrypt_string(column);
        let column_data = encrypted_column.as_bytes();
        let copy_len = std::cmp::min(column_data.len(), 63);
        column_bytes[..copy_len].copy_from_slice(&column_data[..copy_len]);

        let mut name_bytes = vec![0u8; 64];
        let encrypted_name = xor_encrypt_decrypt_string(name);
        let name_data = encrypted_name.as_bytes();
        let copy_len = std::cmp::min(name_data.len(), 63);
        name_bytes[..copy_len].copy_from_slice(&name_data[..copy_len]);

        Self {
            column: column_bytes,
            name: name_bytes,
            type_data,
            access_data,
            sync_data,
            decl_idx,
        }
    }

    pub fn to_serializable(&self) -> IESColumnSerializable {
        IESColumnSerializable {
            column_name: self.column_str(),
            name: self.name_str(),
            type_data: self.type_data,
            access_data: self.access_data,
            sync_data: self.sync_data,
            decl_idx: self.decl_idx,
        }
    }

    pub fn from_serializable(serializable: &IESColumnSerializable) -> Self {
        Self::new(
            &serializable.column_name,
            &serializable.name,
            serializable.type_data,
            serializable.access_data,
            serializable.sync_data,
            serializable.decl_idx,
        )
    }
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct IESRowText {
    pub text_length: u16,

    #[br(count = text_length)]
    pub text_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IESRowTextSerializable {
    pub text: String,
}

impl IESRowText {
    pub fn text_str(&self) -> String {
        let decrypted_bytes: Vec<u8> = self.text_data.iter().map(|&b| b ^ XOR_KEY).collect();
        String::from_utf8_lossy(&decrypted_bytes).to_string()
    }

    pub fn new(text: &str) -> Self {
        let encrypted_bytes: Vec<u8> = text.bytes().map(|b| b ^ XOR_KEY).collect();
        Self {
            text_length: encrypted_bytes.len() as u16,
            text_data: encrypted_bytes,
        }
    }

    pub fn to_serializable(&self) -> IESRowTextSerializable {
        IESRowTextSerializable {
            text: self.text_str(),
        }
    }

    pub fn from_serializable(serializable: &IESRowTextSerializable) -> Self {
        Self::new(&serializable.text)
    }
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IESRowFloat {
    pub float_data: f32,
}

#[derive(Debug, Clone)]
pub struct IESColumnData {
    pub index_data: i32,
    pub text_data: IESRowText,
    pub float_data: Vec<IESRowFloat>,
    pub string_data: Vec<IESRowText>,
    pub padding: Vec<i8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IESColumnDataSerializable {
    pub index_data: i32,
    pub text_data: IESRowTextSerializable,
    pub float_data: Vec<IESRowFloat>,
    pub string_data: Vec<IESRowTextSerializable>,
}

impl IESColumnData {
    pub fn to_serializable(&self) -> IESColumnDataSerializable {
        IESColumnDataSerializable {
            index_data: self.index_data,
            text_data: self.text_data.to_serializable(),
            float_data: self.float_data.clone(),
            string_data: self
                .string_data
                .iter()
                .map(|s| s.to_serializable())
                .collect(),
        }
    }

    pub fn from_serializable(
        serializable: &IESColumnDataSerializable,
        num_column_string: u16,
    ) -> Self {
        Self {
            index_data: serializable.index_data,
            text_data: IESRowText::from_serializable(&serializable.text_data),
            float_data: serializable.float_data.clone(),
            string_data: serializable
                .string_data
                .iter()
                .map(|s| IESRowText::from_serializable(s))
                .collect(),
            padding: vec![0i8; num_column_string as usize],
        }
    }
}

#[derive(Debug)]
pub struct IESHeader {
    pub idspace: Vec<u8>,
    pub keyspace: Vec<u8>,
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
    pub columns: Vec<IESColumn>,
    pub column_data: Vec<IESColumnData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IESHeaderSerializable {
    pub id_space: String,
    pub key_space: String,
    pub version: u16,
    pub info_size: u32,
    pub data_size: u32,
    pub total_size: u32,
    pub use_class_id: u8,
    pub num_field: u16,
    pub num_column: u16,
    pub num_column_number: u16,
    pub num_column_string: u16,
    pub columns: Vec<IESColumnSerializable>,
    pub column_data: Vec<IESColumnDataSerializable>,
}

impl IESHeader {
    pub fn idspace_str(&self) -> String {
        String::from_utf8_lossy(&self.idspace)
            .trim_end_matches('\0')
            .to_string()
    }

    pub fn keyspace_str(&self) -> String {
        String::from_utf8_lossy(&self.keyspace)
            .trim_end_matches('\0')
            .to_string()
    }

    pub fn to_serializable(&self) -> IESHeaderSerializable {
        IESHeaderSerializable {
            // idspace and keyspace are NOT encrypted, so no decryption needed
            id_space: self.idspace_str(),
            key_space: self.keyspace_str(),
            version: self.version,
            info_size: self.info_size,
            data_size: self.data_size,
            total_size: self.total_size,
            use_class_id: self.use_class_id,
            num_field: self.num_field,
            num_column: self.num_column,
            num_column_number: self.num_column_number,
            num_column_string: self.num_column_string,
            columns: self.columns.iter().map(|c| c.to_serializable()).collect(),
            column_data: self
                .column_data
                .iter()
                .map(|d| d.to_serializable())
                .collect(),
        }
    }

    pub fn from_serializable(serializable: &IESHeaderSerializable) -> Self {
        // idspace and keyspace are NOT encrypted
        let mut idspace = vec![0u8; 64];
        let idspace_bytes = serializable.id_space.as_bytes();
        let copy_len = std::cmp::min(idspace_bytes.len(), 63);
        idspace[..copy_len].copy_from_slice(&idspace_bytes[..copy_len]);

        let mut keyspace = vec![0u8; 64];
        let keyspace_bytes = serializable.key_space.as_bytes();
        let copy_len = std::cmp::min(keyspace_bytes.len(), 63);
        keyspace[..copy_len].copy_from_slice(&keyspace_bytes[..copy_len]);

        Self {
            idspace,
            keyspace,
            version: serializable.version,
            padding: 0,
            info_size: serializable.info_size,
            data_size: serializable.data_size,
            total_size: serializable.total_size,
            use_class_id: serializable.use_class_id,
            padding2: 0,
            num_field: serializable.num_field,
            num_column: serializable.num_column,
            num_column_number: serializable.num_column_number,
            num_column_string: serializable.num_column_string,
            padding3: 0,
            columns: serializable
                .columns
                .iter()
                .map(|c| IESColumn::from_serializable(c))
                .collect(),
            column_data: serializable
                .column_data
                .iter()
                .map(|d| IESColumnData::from_serializable(d, serializable.num_column_string))
                .collect(),
        }
    }

    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // Read header fields manually
        let mut idspace = vec![0u8; 64];
        reader.read_exact(&mut idspace)?;

        let mut keyspace = vec![0u8; 64];
        reader.read_exact(&mut keyspace)?;

        let version: u16 = reader.read_le()?;
        let padding: u16 = reader.read_le()?;
        let info_size: u32 = reader.read_le()?;
        let data_size: u32 = reader.read_le()?;
        let total_size: u32 = reader.read_le()?;
        let use_class_id: u8 = reader.read_le()?;
        let padding2: u8 = reader.read_le()?;
        let num_field: u16 = reader.read_le()?;
        let num_column: u16 = reader.read_le()?;
        let num_column_number: u16 = reader.read_le()?;
        let num_column_string: u16 = reader.read_le()?;
        let padding3: u16 = reader.read_le()?;

        // Read columns
        let mut columns = Vec::new();
        for _ in 0..num_column {
            columns.push(IESColumn::read_le(&mut reader)?);
        }

        // Read column data
        let mut column_data = Vec::new();
        for _ in 0..num_field {
            let index_data: i32 = reader.read_le()?;
            let text_data = IESRowText::read_le(&mut reader)?;

            let mut float_data = Vec::new();
            for _ in 0..num_column_number {
                float_data.push(IESRowFloat::read_le(&mut reader)?);
            }

            let mut string_data = Vec::new();
            for _ in 0..num_column_string {
                string_data.push(IESRowText::read_le(&mut reader)?);
            }

            let mut padding = Vec::new();
            for _ in 0..num_column_string {
                padding.push(reader.read_le::<i8>()?);
            }

            column_data.push(IESColumnData {
                index_data,
                text_data,
                float_data,
                string_data,
                padding,
            });
        }

        Ok(IESHeader {
            idspace,
            keyspace,
            version,
            padding,
            info_size,
            data_size,
            total_size,
            use_class_id,
            padding2,
            num_field,
            num_column,
            num_column_number,
            num_column_string,
            padding3,
            columns,
            column_data,
        })
    }

    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Write header fields manually
        writer.write_all(&self.idspace)?;
        writer.write_all(&self.keyspace)?;

        writer.write_le(&self.version)?;
        writer.write_le(&self.padding)?;
        writer.write_le(&self.info_size)?;
        writer.write_le(&self.data_size)?;
        writer.write_le(&self.total_size)?;
        writer.write_le(&self.use_class_id)?;
        writer.write_le(&self.padding2)?;
        writer.write_le(&self.num_field)?;
        writer.write_le(&self.num_column)?;
        writer.write_le(&self.num_column_number)?;
        writer.write_le(&self.num_column_string)?;
        writer.write_le(&self.padding3)?;

        // Write columns
        for column in &self.columns {
            column.write_le(&mut writer)?;
        }

        // Write column data
        for data in &self.column_data {
            writer.write_le(&data.index_data)?;
            data.text_data.write_le(&mut writer)?;

            for float_item in &data.float_data {
                float_item.write_le(&mut writer)?;
            }

            for string_item in &data.string_data {
                string_item.write_le(&mut writer)?;
            }

            for pad_item in &data.padding {
                writer.write_le(pad_item)?;
            }
        }

        writer.flush()?;
        Ok(())
    }

    // Export to JSON
    pub fn to_json<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let serializable = self.to_serializable();
        let json = serde_json::to_string_pretty(&serializable)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    // Import from JSON
    pub fn from_json<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let serializable: IESHeaderSerializable = serde_json::from_str(&json)?;
        Ok(Self::from_serializable(&serializable))
    }

    // Export to XML
    pub fn to_xml<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let serializable = self.to_serializable();
        let xml = quick_xml::se::to_string(&serializable)?;

        let mut file = std::fs::File::create(path)?;
        // Write UTF-8 BOM bytes explicitly
        file.write_all(&[0xEF, 0xBB, 0xBF])?;
        // Write the XML string bytes
        file.write_all(xml.as_bytes())?;
        file.flush()?;
        Ok(())
    }

    // Import from XML
    pub fn from_xml<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let xml = std::fs::read_to_string(path)?;
        let serializable: IESHeaderSerializable = quick_xml::de::from_str(&xml)?;
        Ok(Self::from_serializable(&serializable))
    }
}

#[derive(Parser)]
#[command(name = "ies-tool")]
#[command(about = "IES file converter (IES <-> JSON/XML)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Read and print basic info from an IES file
    Read {
        #[arg(value_name = "IES_FILE")]
        file: PathBuf,
    },
    /// Export IES file to JSON
    ExportJson {
        #[arg(value_name = "IES_FILE")]
        input: PathBuf,
        #[arg(value_name = "OUTPUT_JSON")]
        output: PathBuf,
    },
    /// Export IES file to XML
    ExportXml {
        #[arg(value_name = "IES_FILE")]
        input: PathBuf,
        #[arg(value_name = "OUTPUT_XML")]
        output: PathBuf,
    },
    /// Import from JSON and save as IES file
    ImportJson {
        #[arg(value_name = "INPUT_JSON")]
        input: PathBuf,
        #[arg(value_name = "OUTPUT_IES")]
        output: PathBuf,
    },
    /// Import from XML and save as IES file
    ImportXml {
        #[arg(value_name = "INPUT_XML")]
        input: PathBuf,
        #[arg(value_name = "OUTPUT_IES")]
        output: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Read { file } => match IESHeader::read_from_file(file) {
            Ok(header) => {
                println!("IES File: {:?}", file);
                println!("ID Space: {}", header.idspace_str());
                println!("Key Space: {}", header.keyspace_str());
                println!("Version: {}", header.version);
                println!("Num Fields: {}", header.num_field);
                println!("Num Columns: {}", header.num_column);
            }
            Err(e) => {
                eprintln!("Failed to read IES file: {}", e);
                process::exit(1);
            }
        },
        Commands::ExportJson { input, output } => match IESHeader::read_from_file(input) {
            Ok(header) => {
                if let Err(e) = header.to_json(output) {
                    eprintln!("Failed to export JSON: {}", e);
                    process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Failed to read IES file: {}", e);
                process::exit(1);
            }
        },
        Commands::ExportXml { input, output } => match IESHeader::read_from_file(input) {
            Ok(header) => {
                if let Err(e) = header.to_xml(output) {
                    eprintln!("Failed to export XML: {}", e);
                    process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Failed to read IES file: {}", e);
                process::exit(1);
            }
        },
        Commands::ImportJson { input, output } => match IESHeader::from_json(input) {
            Ok(header) => {
                if let Err(e) = header.write_to_file(output) {
                    eprintln!("Failed to write IES file: {}", e);
                    process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Failed to read JSON file: {}", e);
                process::exit(1);
            }
        },
        Commands::ImportXml { input, output } => match IESHeader::from_xml(input) {
            Ok(header) => {
                if let Err(e) = header.write_to_file(output) {
                    eprintln!("Failed to write IES file: {}", e);
                    process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Failed to read XML file: {}", e);
                process::exit(1);
            }
        },
    }
}

/*

// Example usage and testing
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = r"C:\Users\Ridwan Hidayatullah\Documents\defpartyname.ies";

    // Try to read the existing file
    match IESHeader::read_from_file(file_path) {
        Ok(header) => {
            println!("Successfully read IES file!");
            println!("ID Space: {}", header.idspace_str());
            println!("Key Space: {}", header.keyspace_str());
            println!("Version: {}", header.version);
            println!("Number of fields: {}", header.num_field);
            println!("Number of columns: {}", header.num_column);

            // Print column information (decrypted)
            for (i, column) in header.columns.iter().enumerate() {
                println!("Column {}: {} ({})", i, column.column_str(), column.name_str());
            }

            // Print some row data (decrypted)
            for (i, row) in header.column_data.iter().enumerate().take(5) {
                println!("Row {}: index={}, text={}", i, row.index_data, row.text_data.text_str());
            }

            // Export to JSON (decrypted data)
            header.to_json("defpartyname.json")?;
            println!("Exported to JSON (decrypted): defpartyname.json");

            // Export to XML (decrypted data)
            header.to_xml("defpartyname.xml")?;
            println!("Exported to XML (decrypted): defpartyname.xml");

            // Test round-trip: JSON -> IES (re-encrypted)
            let header_from_json = IESHeader::from_json("defpartyname.json")?;
            header_from_json.write_to_file("defpartyname_from_json.ies")?;
            println!("Created IES file from JSON (re-encrypted): defpartyname_from_json.ies");

            // Test round-trip: XML -> IES (re-encrypted)
            let header_from_xml = IESHeader::from_xml("defpartyname.xml")?;
            header_from_xml.write_to_file("defpartyname_from_xml.ies")?;
            println!("Created IES file from XML (re-encrypted): defpartyname_from_xml.ies");

        }
        Err(e) => {
            println!("Error reading file: {}", e);

            // Create a sample IES file for demonstration
            let sample_header = create_sample_ies_file();

            // Test all export/import functionality with sample data
            sample_header.write_to_file("sample.ies")?;
            println!("Created sample IES file (encrypted): sample.ies");

            sample_header.to_json("sample.json")?;
            println!("Exported sample to JSON (decrypted): sample.json");

            sample_header.to_xml("sample.xml")?;
            println!("Exported sample to XML (decrypted): sample.xml");

            // Test round-trips
            let from_json = IESHeader::from_json("sample.json")?;
            from_json.write_to_file("sample_from_json.ies")?;
            println!("Created IES from JSON (re-encrypted): sample_from_json.ies");

            let from_xml = IESHeader::from_xml("sample.xml")?;
            from_xml.write_to_file("sample_from_xml.ies")?;
            println!("Created IES from XML (re-encrypted): sample_from_xml.ies");

            // Verify encryption by reading raw bytes
            println!("\n--- Encryption Verification ---");
            println!("Reading raw encrypted data from sample.ies...");
            verify_encryption("sample.ies")?;
        }
    }

    Ok(())
}



fn verify_encryption(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut buffer = vec![0u8; 1024]; // Read first 1KB
    let bytes_read = file.read(&mut buffer)?;

    println!("First few encrypted bytes (excluding header):");
    // Skip idspace (64 bytes) and keyspace (64 bytes) = 128 bytes
    for (i, &byte) in buffer[128..std::cmp::min(200, bytes_read)].iter().enumerate() {
        if i > 0 && i % 16 == 0 {
            println!();
        }
        print!("{:02X} ", byte);
    }
    println!();

    Ok(())
}

fn create_sample_ies_file() -> IESHeader {
    // Create sample data
    let mut idspace = vec![0u8; 64];
    "SAMPLE_ID".as_bytes().iter().enumerate().for_each(|(i, &b)| idspace[i] = b);

    let mut keyspace = vec![0u8; 64];
    "SAMPLE_KEY".as_bytes().iter().enumerate().for_each(|(i, &b)| keyspace[i] = b);

    let columns = vec![
        IESColumn::new("ClassID", "ID", 1, 0, 0, 0),
        IESColumn::new("ClassName", "Name", 2, 0, 0, 1),
        IESColumn::new("Value", "Value", 3, 0, 0, 2),
    ];

    let column_data = vec![
        IESColumnData {
            index_data: 1,
            text_data: IESRowText::new("Sample Row 1"),
            float_data: vec![IESRowFloat { float_data: 1.5 }],
            string_data: vec![IESRowText::new("String1")],
            padding: vec![0],
        },
        IESColumnData {
            index_data: 2,
            text_data: IESRowText::new("Sample Row 2"),
            float_data: vec![IESRowFloat { float_data: 2.5 }],
            string_data: vec![IESRowText::new("String2")],
            padding: vec![0],
        },
    ];

    IESHeader {
        idspace,
        keyspace,
        version: 1,
        padding: 0,
        info_size: 0,
        data_size: 0,
        total_size: 0,
        use_class_id: 1,
        padding2: 0,
        num_field: column_data.len() as u16,
        num_column: columns.len() as u16,
        num_column_number: 1,
        num_column_string: 1,
        padding3: 0,
        columns,
        column_data,
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ies_column_creation() {
        let column = IESColumn::new("TestCol", "TestName", 1, 2, 3, 4);
        assert_eq!(column.column_str(), "TestCol");
        assert_eq!(column.name_str(), "TestName");
        assert_eq!(column.type_data, 1);
    }

    #[test]
    fn test_ies_row_text() {
        let text = IESRowText::new("Hello World");
        assert_eq!(text.text_str(), "Hello World");
        assert_eq!(text.text_length, 11);
    }

    #[test]
    fn test_serialization_conversion() {
        let column = IESColumn::new("TestCol", "TestName", 1, 2, 3, 4);
        let serializable = column.to_serializable();
        let converted_back = IESColumn::from_serializable(&serializable);

        assert_eq!(column.column_str(), converted_back.column_str());
        assert_eq!(column.name_str(), converted_back.name_str());
        assert_eq!(column.type_data, converted_back.type_data);
    }

    #[test]
    fn test_json_serialization() {
        let header = create_sample_ies_file();
        let serializable = header.to_serializable();

        let json = serde_json::to_string(&serializable).unwrap();
        let deserialized: IESHeaderSerializable = serde_json::from_str(&json).unwrap();

        assert_eq!(serializable.version, deserialized.version);
        assert_eq!(serializable.num_field, deserialized.num_field);
        assert_eq!(serializable.id_space, deserialized.id_space);
    }

    #[test]
    fn test_xml_serialization() {
        let header = create_sample_ies_file();
        let serializable = header.to_serializable();

        let xml = quick_xml::se::to_string(&serializable).unwrap();
        let deserialized: IESHeaderSerializable = quick_xml::de::from_str(&xml).unwrap();

        assert_eq!(serializable.version, deserialized.version);
        assert_eq!(serializable.num_field, deserialized.num_field);
        assert_eq!(serializable.id_space, deserialized.id_space);
    }

    #[test]
    fn test_round_trip_json() -> Result<(), Box<dyn std::error::Error>> {
        let original = create_sample_ies_file();

        // Write to JSON and back
        original.to_json("test_round_trip.json")?;
        let from_json = IESHeader::from_json("test_round_trip.json")?;

        // Compare key fields
        assert_eq!(original.version, from_json.version);
        assert_eq!(original.num_field, from_json.num_field);
        assert_eq!(original.num_column, from_json.num_column);
        assert_eq!(original.idspace_str(), from_json.idspace_str());
        assert_eq!(original.keyspace_str(), from_json.keyspace_str());

        // Clean up test files
        std::fs::remove_file("test_round_trip.json").ok();

        Ok(())
    }

    #[test]
    fn test_round_trip_xml() -> Result<(), Box<dyn std::error::Error>> {
        let original = create_sample_ies_file();

        // Write to XML and back
        original.to_xml("test_round_trip.xml")?;
        let from_xml = IESHeader::from_xml("test_round_trip.xml")?;

        // Compare key fields
        assert_eq!(original.version, from_xml.version);
        assert_eq!(original.num_field, from_xml.num_field);
        assert_eq!(original.num_column, from_xml.num_column);
        assert_eq!(original.idspace_str(), from_xml.idspace_str());
        assert_eq!(original.keyspace_str(), from_xml.keyspace_str());

        // Clean up test files
        std::fs::remove_file("test_round_trip.xml").ok();

        Ok(())
    }
}
