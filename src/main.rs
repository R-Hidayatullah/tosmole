#![feature(seek_stream_len)]

use crate::ipf::ipf_parser::{ipf_get_data, ipf_parse};
use std::fs::File;
use std::io::BufReader;

mod fsb;
mod ies;
mod ipf;
mod xac;
mod xml;
mod xsm;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let args_count = std::env::args().count();
    if args_count == 1 {
        println!("Usage :\n1. tosmole example.ipf\n2. tosmole example.ipf index_number");
    } else if args_count == 2 {
        println!("Parse first index.");
        let path_file = &args[1];
        let mut location = BufReader::new(File::open(path_file).unwrap());
        let ipf_data = ipf_parse(&mut location);
        ipf_get_data(&mut location, &ipf_data, 0);
        println!("\nFinish parsing first index.");
    } else if args_count >= 3 {
        let path_file = &args[1];
        let index_list = &args[2];
        println!("Parse index : {}", index_list);
        let mut location = BufReader::new(File::open(path_file).unwrap());
        let ipf_data = ipf_parse(&mut location);
        ipf_get_data(
            &mut location,
            &ipf_data,
            index_list.to_string().parse::<usize>().unwrap(),
        );
        println!("\nFinish parsing index : {}", index_list);
        {}
    }
}
