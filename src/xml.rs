use quick_xml::{Reader, events::Event};
use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

pub fn parse_duplicates_xml(path: &Path) -> std::io::Result<HashMap<String, String>> {
    let file = File::open(path)?;
    let mut reader = Reader::from_reader(BufReader::new(file));
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut map: HashMap<String, String> = HashMap::new();

    let mut current_source: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            // <source file="...">
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"source" => {
                current_source = e
                    .attributes()
                    .flatten()
                    .find(|a| a.key.as_ref() == b"file")
                    .map(|a| {
                        let s = String::from_utf8_lossy(&a.value).into_owned();
                        s.replace('\\', "/") // normalize
                    });
            }

            // <target file="..." />
            Ok(Event::Empty(ref e)) if e.name().as_ref() == b"target" => {
                if let (Some(src), Some(attr)) = (
                    current_source.as_ref(),
                    e.attributes().flatten().find(|a| a.key.as_ref() == b"file"),
                ) {
                    let target = String::from_utf8_lossy(&attr.value).into_owned();
                    map.insert(target.replace('\\', "/"), src.clone());
                }
            }

            // <source file="..." /> (self-closing, no targets)
            Ok(Event::Empty(ref e)) if e.name().as_ref() == b"source" => {
                // do nothing (self-closing source without targets)
            }

            Ok(Event::End(ref e)) if e.name().as_ref() == b"source" => {
                current_source.take();
            }

            Ok(Event::Eof) => break,

            Err(e) => {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, e));
            }
            _ => {}
        }

        buf.clear();
    }

    Ok(map)
}
