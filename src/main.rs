#![feature(seek_stream_len)]

use ipf_parser::IpfFile;

mod ies_parser;
mod ipf_parser;
mod xac_parser;
mod xsm_parser;

fn main() {
    let mut ipf_data = IpfFile::load_from_file("tests\\379124_001001.ipf").expect("msg");

    ipf_data
        .get_data("tests\\379124_001001.ipf", 0)
        .expect("msg");
}
