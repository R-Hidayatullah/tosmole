use ipf::IPFRoot;
use std::io;

mod ies;
mod ipf;

fn main() -> io::Result<()> {
    let root = IPFRoot::from_file("tests/379124_001001.ipf")?;

    println!("Header : {:#?}", root.header);
    println!("Data : {:?}", root.file_table);
    Ok(())
}
