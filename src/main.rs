#![feature(seek_stream_len)]

use crate::window_render::render;

mod ies_parser;
mod ipf_parser;
mod window_render;
mod xac_parser;
mod xsm_parser;

fn main() {
    println!("Hello world");
    render().unwrap();
}
