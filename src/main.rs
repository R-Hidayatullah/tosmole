#![allow(unused)]

use ipf::IPFRoot;
use std::{collections::BTreeMap, io, path::Path};

use crate::ipf::IPFFileTable;

mod category;
mod ies;
mod ipf;
mod tsv;
mod xml;

fn main() -> io::Result<()> {
    let game_root = Path::new(r"/home/ridwan/Documents/TreeOfSaviorCN/");
    let lang_folder =
        Path::new(r"/home/ridwan/Documents/TreeOfSaviorCN/release/languageData/English");

    // Timer for parsing IPFs
    let start_total = std::time::Instant::now();
    let start = std::time::Instant::now();
    let mut parsed_ipfs = ipf::parse_game_ipfs(game_root)?;
    let duration = start.elapsed();
    println!(
        "Parsed total {} IPF archives from both 'data' and 'patch' in {:.2?}",
        parsed_ipfs.len(),
        duration,
    );

    // Timer for extracting a sample file
    let start = std::time::Instant::now();
    let data = parsed_ipfs
        .get(0)
        .unwrap()
        .file_table
        .get(0)
        .unwrap()
        .extract_data()?;
    println!("Extracted sample data in {:.2?}", start.elapsed());

    // Timer for collecting and sorting all files
    let start = std::time::Instant::now();
    let mut all_files = ipf::collect_file_tables_from_parsed(&mut parsed_ipfs);
    ipf::sort_file_tables_by_folder_then_name(&mut all_files);
    println!(
        "Collected and sorted {} files in {:.2?}",
        all_files.len(),
        start.elapsed()
    );

    // Timer for parsing language data
    let start = std::time::Instant::now();
    let (etc_data, item_data) = tsv::parse_language_data(lang_folder)?;
    println!(
        "Parsed language data (ETC: {}, ITEM: {}) in {:.2?}",
        etc_data.len(),
        item_data.len(),
        start.elapsed()
    );

    // Timer for building folder tree
    let start = std::time::Instant::now();
    let grouped: BTreeMap<String, Vec<IPFFileTable>> =
        ipf::group_file_tables_by_directory(all_files);
    let result = category::build_tree(grouped);
    println!("Built folder tree in {:.2?}", start.elapsed());

    // Timer for shallow folder search
    let start = std::time::Instant::now();
    if let Some((subfolders, files)) = result.search_folder_shallow("ui") {
        println!("Folder 'ui' subfolders: {:?}", subfolders);
        println!("Folder 'ui' files: {:?}", files);
    }
    println!("Shallow folder search in {:.2?}", start.elapsed());

    // Timer for recursive file search
    let start = std::time::Instant::now();
    let matches = result.search_file_recursive("R1.txt", "");
    for (full_path, file) in matches {
        println!("Found file: {} ({:?})", full_path, file.file_path);
    }
    println!("Recursive file search in {:.2?}", start.elapsed());

    // Timer for full-path file search
    let start = std::time::Instant::now();
    let matches = result.search_file_by_full_path("ui/brush/spraycursor_1.tga");
    for (full_path, file) in matches {
        println!("Found file: {} ({:?})", full_path, file.file_path);
    }
    println!("Full-path file search in {:.2?}", start.elapsed());

    // Timer for hex view
    let start = std::time::Instant::now();
    ipf::print_hex_viewer(&data);
    println!("Printed hex viewer in {:.2?}", start.elapsed());

    // Timer for parsing duplicates
    let start = std::time::Instant::now();
    let xac_duplicates =
        xml::parse_duplicates_xml(&game_root.join("release").join("xac_duplicates.xml"))?;
    let xsm_duplicates =
        xml::parse_duplicates_xml(&game_root.join("release").join("xsm_duplicates.xml"))?;
    let dds_duplicates =
        xml::parse_duplicates_xml(&game_root.join("release").join("dds_duplicates.xml"))?;
    let xpm_duplicates =
        xml::parse_duplicates_xml(&game_root.join("release").join("xpm_duplicates.xml"))?;
    let xsmtime_duplicates =
        xml::parse_duplicates_xml(&game_root.join("release").join("xsmtime_duplicates.xml"))?;
    println!("Parsed duplicates in {:.2?}", start.elapsed());

    println!(
        "XAC duplicates: {}, XSM: {}, DDS: {}, XPM: {}, XSMTIME: {}",
        xac_duplicates.len(),
        xsm_duplicates.len(),
        dds_duplicates.len(),
        xpm_duplicates.len(),
        xsmtime_duplicates.len()
    );
    println!("XAC data : {:?}", xac_duplicates.get(0).unwrap());

    println!("Total program time: {:.2?}", start_total.elapsed());
    Ok(())
}
