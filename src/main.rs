#![allow(unused_variables, unused_imports, dead_code)]
use std::{
    fs::{self, File, read_dir},
    io::{self, BufRead, BufReader, Cursor},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Instant,
};

use quick_xml::{Reader, events::Event};

use crate::{
    binary::{BinaryReader, Endian},
    ipf::{IPFRoot, extract_and_print_example, parse_game_ipfs},
    tsv::parse_language_data,
    xac::XACRoot,
    xml::parse_duplicates_xml,
    xpm::XPMRoot,
    xsm::XSMRoot,
};

mod binary;
mod ies;
mod ipf;
mod shared_formats;
mod tsv;
mod xac;
mod xml;
mod xpm;
mod xsm;

fn parse_all_tests() -> io::Result<()> {
    for entry in fs::read_dir("tests")? {
        let entry = entry?;
        let path = entry.path();

        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let bytes = std::fs::read(&path)?;
            let mut reader = BinaryReader::new(Cursor::new(bytes), Endian::Little);

            match ext {
                "xsm" => {
                    let root = XSMRoot::read_from(&mut reader)?;
                    println!("Parsed XSM: {:?}", root.header);
                }
                "xac" => {
                    let root = XACRoot::read_from(&mut reader)?;
                    println!("Parsed XAC: {:?}", root.header);
                }
                "xpm" => {
                    let root = XPMRoot::read_from(&mut reader)?;
                    println!("Parsed XPM: {:?}", root.header);
                }

                _ => {
                    println!("Skipping file {:?}", path);
                }
            }
        }
    }
    Ok(())
}
fn main() -> io::Result<()> {
    parse_all_tests()?;

    // let game_root = Path::new(r"C:\Users\Ridwan Hidayatullah\Documents\TreeOfSaviorCN");
    // let lang_folder = Path::new(
    //     r"C:\Users\Ridwan Hidayatullah\Documents\TreeOfSaviorCN\release\languageData\English",
    // );

    // // 1. Parse IPF archives with limited threads
    // let start = std::time::Instant::now();
    // let mut parsed_ipfs = parse_game_ipfs(game_root)?;
    // let duration = start.elapsed();

    // println!(
    //     "Parsed total {} IPF archives from both 'data' and 'patch' in {:.2?}",
    //     parsed_ipfs.len(),
    //     duration,
    // );

    // // 2. Extract example file and print info
    // extract_and_print_example(&mut parsed_ipfs)?;

    // // 3. Parse language TSV data concurrently
    // let (etc_data, item_data) = parse_language_data(lang_folder)?;

    // println!("ETC.tsv lines: {}", etc_data.len());
    // println!("ITEM.tsv lines: {}\n", item_data.len());

    // // Print first 3 rows of each
    // for row in etc_data.iter().take(3) {
    //     println!("ETC row: {:?}", row);
    // }
    // for row in item_data.iter().take(3) {
    //     println!("ITEM row: {:?}\n", row);
    // }
    // // Load each duplicates file into its own variable
    // let xac_duplicates =
    //     parse_duplicates_xml(&game_root.join("release").join("xac_duplicates.xml"))?;
    // let xsm_duplicates =
    //     parse_duplicates_xml(&game_root.join("release").join("xsm_duplicates.xml"))?;
    // let dds_duplicates =
    //     parse_duplicates_xml(&game_root.join("release").join("dds_duplicates.xml"))?;
    // let xpm_duplicates =
    //     parse_duplicates_xml(&game_root.join("release").join("xpm_duplicates.xml"))?;
    // let xsmtime_duplicates =
    //     parse_duplicates_xml(&game_root.join("release").join("xsmtime_duplicates.xml"))?;

    // println!("XAC duplicates: {}", xac_duplicates.len());
    // println!("XSM duplicates: {}", xsm_duplicates.len());
    // println!("DDS duplicates: {}", dds_duplicates.len());
    // println!("XPM duplicates: {}", xpm_duplicates.len());
    // println!("XSMTIME duplicates: {}", xsmtime_duplicates.len());

    // println!("XAC data : {:?}", xac_duplicates.get(10).unwrap());
    Ok(())
}
