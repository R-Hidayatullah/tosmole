#![feature(seek_stream_len)]

use xsm_parser::XsmFile;

use crate::xac_parser::XacFile;

mod ies_parser;
mod xac_parser;
mod xsm_parser;

fn main() {
    let xac_file = XacFile::load_from_file("tests\\archer_m_falconer01.xac").expect("");
    println!("{:?}", &xac_file);
}
