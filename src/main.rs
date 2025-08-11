#![allow(unused_variables, unused_imports, dead_code)]

mod binary;
mod ies;
mod ipf;
mod shared_formats;
mod xac;
mod xpm;
mod xsm;

use std::{fs::File, io};

use crate::{
    binary::{BinaryReader, Endian},
    ies::IESRoot,
};

fn main() -> io::Result<()> {
    let path = "tests/cell.ies";

    // Example 1 — Read directly from file
    {
        let file = File::open(path)?;
        let mut reader = BinaryReader::new(file, Endian::Little);
        let root = IESRoot::read_from(&mut reader)?;
        println!("File IES: {:?}", root);
    }

    Ok(())
}
