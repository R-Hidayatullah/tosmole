use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufReader, Read};

/// Attribute type mapping (from PathEngine docs)
#[repr(u8)]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub enum TokAttrType {
    CString = 1,
    SInt16 = 3,
    SInt8 = 4,
    Unknown(u8),
}

impl From<u8> for TokAttrType {
    fn from(v: u8) -> Self {
        match v {
            1 => Self::CString,
            3 => Self::SInt16,
            4 => Self::SInt8,
            other => Self::Unknown(other),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TokElement {
    pub index: u8,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TokAttribute {
    pub index: u8,
    pub attr_type: TokAttrType,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TokNode {
    pub element_index: u8,
    pub element_name: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<TokNode>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TokFile {
    pub elements: Vec<TokElement>,
    pub attributes: Vec<TokAttribute>,
    pub root_nodes: Vec<TokNode>,
}

/// Reads a null-terminated C string
fn read_cstring(buf: &[u8], pos: &mut usize) -> io::Result<String> {
    let start = *pos;
    while *pos < buf.len() && buf[*pos] != 0 {
        *pos += 1;
    }
    let s = String::from_utf8_lossy(&buf[start..*pos]).to_string();
    *pos += 1; // skip null
    Ok(s)
}

fn read_i16(buf: &[u8], pos: &mut usize) -> io::Result<i16> {
    if *pos + 2 > buf.len() {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read_i16"));
    }
    let val = i16::from_le_bytes([buf[*pos], buf[*pos + 1]]);
    *pos += 2;
    Ok(val)
}

fn read_i8(buf: &[u8], pos: &mut usize) -> io::Result<i8> {
    if *pos >= buf.len() {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "read_i8"));
    }
    let val = buf[*pos] as i8;
    *pos += 1;
    Ok(val)
}

/// Parse .tok file
pub fn parse_tok_file<R: Read>(mut reader: R) -> io::Result<TokFile> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    let mut pos = 0;

    // --- Enumerated Elements ---
    let mut elements = Vec::new();
    loop {
        let s = read_cstring(&buf, &mut pos)?;
        if s.is_empty() {
            break;
        }
        elements.push(TokElement {
            index: elements.len() as u8 + 1,
            name: s,
        });
    }

    // --- Enumerated Attributes ---
    let mut attributes = Vec::new();
    while pos < buf.len() {
        let t = buf[pos];
        pos += 1;
        if t == 0 {
            break;
        }
        let name = read_cstring(&buf, &mut pos)?;
        attributes.push(TokAttribute {
            index: attributes.len() as u8 + 1,
            attr_type: TokAttrType::from(t),
            name,
        });
    }

    // --- Document Section (recursive) ---
    fn parse_node(
        buf: &[u8],
        pos: &mut usize,
        elements: &[TokElement],
        attrs: &[TokAttribute],
    ) -> io::Result<Option<TokNode>> {
        if *pos >= buf.len() {
            return Ok(None);
        }

        let elem_index = buf[*pos];
        *pos += 1;

        if elem_index == 0 {
            return Ok(None); // end marker
        }

        let element_name = elements
            .get((elem_index - 1) as usize)
            .map(|e| e.name.clone())
            .unwrap_or_else(|| format!("unknown_{}", elem_index));

        let mut attributes = Vec::new();

        loop {
            let attr_index = buf[*pos];
            *pos += 1;
            if attr_index == 0 {
                break;
            }

            let attr = attrs.get((attr_index - 1) as usize);
            if let Some(a) = attr {
                let val = match a.attr_type {
                    TokAttrType::CString => read_cstring(buf, pos)?,
                    TokAttrType::SInt16 => read_i16(buf, pos)?.to_string(),
                    TokAttrType::SInt8 => read_i8(buf, pos)?.to_string(),
                    TokAttrType::Unknown(_) => "??".into(),
                };
                attributes.push((a.name.clone(), val));
            } else {
                // Unknown attribute index, skip?
                break;
            }
        }

        // Parse children recursively
        let mut children = Vec::new();
        while let Some(child) = parse_node(buf, pos, elements, attrs)? {
            children.push(child);
        }

        Ok(Some(TokNode {
            element_index: elem_index,
            element_name,
            attributes,
            children,
        }))
    }

    let mut root_nodes = Vec::new();
    while let Some(node) = parse_node(&buf, &mut pos, &elements, &attributes)? {
        root_nodes.push(node);
    }

    Ok(TokFile {
        elements,
        attributes,
        root_nodes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn parse_barrack_noble_tok() -> io::Result<()> {
        let path = Path::new("tests/barrack_noble.tok");
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let tok = parse_tok_file(reader)?;

        println!("Parsed elements:");
        for e in &tok.elements {
            println!("  {}: {}", e.index, e.name);
        }

        println!("\nParsed attributes:");
        for a in &tok.attributes {
            println!("  {}: {:?} {}", a.index, a.attr_type, a.name);
        }

        println!("\nDocument tree:");
        for node in &tok.root_nodes {
            println!("{:?}", node);
        }

        assert!(!tok.elements.is_empty());
        assert!(!tok.attributes.is_empty());
        assert!(!tok.root_nodes.is_empty());

        Ok(())
    }
}
