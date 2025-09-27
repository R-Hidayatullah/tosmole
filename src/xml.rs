use std::{fs::File, io::BufReader, path::Path};

use quick_xml::{Reader, events::Event};

#[derive(Debug)]
pub struct DuplicateEntry {
    pub source: String,
    pub targets: Vec<String>,
}

pub fn parse_duplicates_xml(path: &Path) -> std::io::Result<Vec<DuplicateEntry>> {
    let file = File::open(path)?;
    let mut reader = Reader::from_reader(BufReader::new(file));
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut entries = Vec::new();
    let mut current_source: Option<String> = None;
    let mut current_targets: Vec<String> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            // <source file="...">  (start of a source)
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"source" => {
                current_targets.clear();
                current_source = e
                    .attributes()
                    .flatten()
                    .find(|a| a.key.as_ref() == b"file")
                    .map(|a| String::from_utf8_lossy(&a.value).into_owned());
            }

            // <source file="..." />  (self-closing source — no targets)
            Ok(Event::Empty(ref e)) if e.name().as_ref() == b"source" => {
                if let Some(attr) = e.attributes().flatten().find(|a| a.key.as_ref() == b"file") {
                    entries.push(DuplicateEntry {
                        source: String::from_utf8_lossy(&attr.value).into_owned(),
                        targets: Vec::new(),
                    });
                }
            }

            // <target file="..." />  (self-closing target)
            Ok(Event::Empty(ref e)) if e.name().as_ref() == b"target" => {
                if let Some(attr) = e.attributes().flatten().find(|a| a.key.as_ref() == b"file") {
                    current_targets.push(String::from_utf8_lossy(&attr.value).into_owned());
                }
            }

            // <target file="..."> ... </target>  (if targets were not self-closing)
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"target" => {
                if let Some(attr) = e.attributes().flatten().find(|a| a.key.as_ref() == b"file") {
                    current_targets.push(String::from_utf8_lossy(&attr.value).into_owned());
                }
            }

            // </source> — commit the current source + collected targets
            Ok(Event::End(ref e)) if e.name().as_ref() == b"source" => {
                if let Some(src) = current_source.take() {
                    entries.push(DuplicateEntry {
                        source: src,
                        targets: current_targets.clone(),
                    });
                }
            }

            Ok(Event::Eof) => break,

            Err(e) => {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e));
            }
            _ => {}
        }

        buf.clear();
    }

    Ok(entries)
}
