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
    let index = 0;
    let start = std::time::Instant::now();
    let mut parsed_ipfs = ipf::parse_game_ipfs(game_root)?;
    let duration = start.elapsed();

    println!(
        "Parsed total {} IPF archives from both 'data' and 'patch' in {:.2?}",
        parsed_ipfs.len(),
        duration,
    );

    let data = parsed_ipfs
        .get(index)
        .unwrap()
        .file_table
        .get(index)
        .unwrap()
        .extract_data()
        .unwrap();

    let mut all_files = ipf::collect_file_tables_from_parsed(&mut parsed_ipfs);
    ipf::sort_file_tables_by_folder_then_name(&mut all_files);

    let (etc_data, item_data) = tsv::parse_language_data(lang_folder)?;

    println!("Total IPF files collected: {}", all_files.len());

    // Move files into grouped map
    let grouped: BTreeMap<String, Vec<IPFFileTable>> =
        ipf::group_file_tables_by_directory(all_files);

    let result = category::build_tree(grouped);

    for file in &result.files {
        if file.directory_name.contains("06.dds") {
            println!(
                "Found {} at result: {:?}",
                file.directory_name, file.file_path
            );
        }
    }
    ipf::print_hex_viewer(&data);

    println!("ETC.tsv lines: {}", etc_data.len());
    println!("ITEM.tsv lines: {}\n", item_data.len());

    // Print first 3 rows of each
    for row in etc_data.iter().take(3) {
        println!("ETC row: {:?}", row);
    }
    println!();
    for row in item_data.iter().take(3) {
        println!("ITEM row: {:?}", row);
    }
    println!();

    // Load each duplicates file into its own variable
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

    println!("XAC duplicates: {}", xac_duplicates.len());
    println!("XSM duplicates: {}", xsm_duplicates.len());
    println!("DDS duplicates: {}", dds_duplicates.len());
    println!("XPM duplicates: {}", xpm_duplicates.len());
    println!("XSMTIME duplicates: {}", xsmtime_duplicates.len());

    println!("XAC data : {:?}", xac_duplicates.get(0).unwrap());
    Ok(())
}
