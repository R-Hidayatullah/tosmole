use ipf::IPFRoot;
use std::io;

mod ies;
mod ipf;

fn main() -> io::Result<()> {
    let root = IPFRoot::from_file("tests/379124_001001.ipf")?;

    println!("Header : {:#?}", root.header);
    println!("Data Length : {:?}", root.file_table.len());
    let index = 37;
    if let Some(file_entry) = root.file_table.get(index) {
        let result_data = file_entry.extract_data()?;
        println!("Table Index {}: {:?}", index, file_entry);
        println!("Result Index {} length: {}", index, result_data.len());
        // Convert to string (text) if possible
        match String::from_utf8(result_data) {
            Ok(text) => println!("Result Index 38 as text:\n{}", text),
            Err(_) => println!("Result Index 38 is not valid UTF-8 text"),
        }
    } else {
        println!("File table index 37 does not exist");
    }
    Ok(())
}
