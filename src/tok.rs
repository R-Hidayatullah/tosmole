use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{self, BufReader, Read};

use serde::{Deserialize, Serialize};

/// Type specifiers for attributes in .tok files.
///
/// Integers are stored little-endian (low bytes first).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokAttrType {
    CString = 1,
    SInt32 = 2,
    SInt16 = 3,
    SInt8 = 4,
    UInt32 = 5,
    UInt16 = 6,
    UInt8 = 7,
}

impl TokAttrType {
    pub fn size(&self) -> Option<usize> {
        match self {
            TokAttrType::CString => None,
            TokAttrType::SInt32 | TokAttrType::UInt32 => Some(4),
            TokAttrType::SInt16 | TokAttrType::UInt16 => Some(2),
            TokAttrType::SInt8 | TokAttrType::UInt8 => Some(1),
        }
    }

    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::CString),
            2 => Some(Self::SInt32),
            3 => Some(Self::SInt16),
            4 => Some(Self::SInt8),
            5 => Some(Self::UInt32),
            6 => Some(Self::UInt16),
            7 => Some(Self::UInt8),
            _ => None,
        }
    }
}

/// Representation of a node (element) in the .tok document tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokNode {
    pub element_index: u8,
    pub element_name: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<TokNode>,
}

impl fmt::Display for TokNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "TokNode {{ element_index: {}, element_name: {:?}, attributes: {:?} }}",
            self.element_index, self.element_name, self.attributes
        )?;
        for child in &self.children {
            write!(f, "{}", child)?;
        }
        Ok(())
    }
}

/// The main parser structure.
pub struct TokParser<R: Read> {
    reader: R,
    pos: usize,
    buf: Vec<u8>,
    element_names: HashMap<u8, String>,
    attribute_types: HashMap<u8, (TokAttrType, String)>,
}

impl<R: Read> TokParser<R> {
    pub fn new(mut reader: R) -> io::Result<Self> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        Ok(Self {
            reader,
            pos: 0,
            buf,
            element_names: HashMap::new(),
            attribute_types: HashMap::new(),
        })
    }

    fn read_u8(&mut self) -> u8 {
        let v = self.buf[self.pos];
        self.pos += 1;
        v
    }

    fn read_i8(&mut self) -> i8 {
        self.read_u8() as i8
    }

    fn read_le_i16(&mut self) -> i16 {
        let bytes = &self.buf[self.pos..self.pos + 2];
        self.pos += 2;
        i16::from_le_bytes(bytes.try_into().unwrap())
    }

    fn read_le_i32(&mut self) -> i32 {
        let bytes = &self.buf[self.pos..self.pos + 4];
        self.pos += 4;
        i32::from_le_bytes(bytes.try_into().unwrap())
    }

    fn read_le_u16(&mut self) -> u16 {
        let bytes = &self.buf[self.pos..self.pos + 2];
        self.pos += 2;
        u16::from_le_bytes(bytes.try_into().unwrap())
    }

    fn read_le_u32(&mut self) -> u32 {
        let bytes = &self.buf[self.pos..self.pos + 4];
        self.pos += 4;
        u32::from_le_bytes(bytes.try_into().unwrap())
    }

    fn read_cstring(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.buf.len() && self.buf[self.pos] != 0 {
            self.pos += 1;
        }
        let s = String::from_utf8_lossy(&self.buf[start..self.pos]).to_string();
        self.pos += 1; // skip null terminator
        s
    }

    fn parse_element_names(&mut self) {
        let mut idx = 1;
        loop {
            let s = self.read_cstring();
            if s.is_empty() {
                break;
            }
            self.element_names.insert(idx, s);
            idx += 1;
        }
    }

    fn parse_attribute_types(&mut self) {
        loop {
            let t = self.read_u8();
            if t == 0 {
                break;
            }
            let name = self.read_cstring();
            let attr_type = TokAttrType::from_u8(t).unwrap_or(TokAttrType::CString);
            self.attribute_types
                .insert(self.attribute_types.len() as u8 + 1, (attr_type, name));
        }
    }

    fn read_attribute_value(&mut self, attr_type: TokAttrType) -> String {
        match attr_type {
            TokAttrType::CString => self.read_cstring(),
            TokAttrType::SInt8 => self.read_i8().to_string(),
            TokAttrType::SInt16 => self.read_le_i16().to_string(),
            TokAttrType::SInt32 => self.read_le_i32().to_string(),
            TokAttrType::UInt8 => self.read_u8().to_string(),
            TokAttrType::UInt16 => self.read_le_u16().to_string(),
            TokAttrType::UInt32 => self.read_le_u32().to_string(),
        }
    }

    fn parse_node(&mut self) -> Option<TokNode> {
        let element_index = self.read_u8();
        if element_index == 0 {
            return None;
        }
        let element_name = self
            .element_names
            .get(&element_index)
            .cloned()
            .unwrap_or_else(|| format!("Unknown{}", element_index));

        let mut attributes = Vec::new();
        loop {
            let attr_index = self.read_u8();
            if attr_index == 0 {
                break;
            }

            // Take a copy of the data (TokAttrType is Copy, name is cloned)
            let attr_data = match self.attribute_types.get(&attr_index) {
                Some(&(t, ref name)) => (t, name.clone()), // clone the name here
                None => continue,
            };

            // Now safe to mutably borrow self
            let value = self.read_attribute_value(attr_data.0);
            attributes.push((attr_data.1, value));
        }

        let mut children = Vec::new();
        while let Some(child) = self.parse_node() {
            children.push(child);
        }

        Some(TokNode {
            element_index,
            element_name,
            attributes,
            children,
        })
    }

    pub fn parse(mut self) -> io::Result<TokNode> {
        self.parse_element_names();
        self.parse_attribute_types();
        Ok(self.parse_node().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{BufReader, Cursor, Read};

    #[test]
    fn parse_barrack_noble_tok_file() {
        let path = "tests/barrack_noble.tok";
        let file = File::open(path).expect("missing test file");
        let reader = BufReader::new(file);
        let parser = TokParser::new(reader).unwrap();
        let root = parser.parse().unwrap();

        println!("Parsed root element from file: {}", root.element_name);
        println!("Document tree: {:?}", root);
    }

    #[test]
    fn parse_barrack_noble_tok_buffer() {
        // Read the file into memory
        let path = "tests/barrack_noble.tok";
        let mut file = File::open(path).expect("missing test file");
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();

        // Use a Cursor to provide BufRead/Read interface
        let cursor = Cursor::new(buf);
        let parser = TokParser::new(cursor).unwrap();
        let root = parser.parse().unwrap();

        println!("Parsed root element from buffer: {}", root.element_name);
        println!("Document tree: {:?}", root);
    }
}
