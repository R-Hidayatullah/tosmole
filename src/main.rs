#![feature(seek_stream_len)]

use ipf_parser::IpfFile;

use std::{fs, thread, time, time::Instant};

mod ies_parser;
mod ipf_parser;
mod xac_parser;
mod xsm_parser;

fn process_folder(
    folder_path: &str,
) -> std::collections::HashMap<String, ipf_parser::IPFFileTable> {
    let entries = fs::read_dir(folder_path).expect("Failed to read directory");
    let mut folder_hashmap = std::collections::HashMap::new();

    for entry in entries {
        if let Ok(entry) = entry {
            let file_path = entry.path();

            if let Some(extension) = file_path.extension() {
                if extension.to_string_lossy().to_lowercase() == "ipf" {
                    let mut ipf_data =
                        IpfFile::load_from_file(&file_path).expect("Failed to load IPF file");

                    if let Ok(hashmap_res) = ipf_data.into_hashmap() {
                        folder_hashmap.extend(hashmap_res);
                    }
                }
            }
        }
    }

    folder_hashmap
}

fn main() {
    let folder_path1 = "C:\\Program Files (x86)\\Steam\\steamapps\\common\\TreeOfSavior\\data";
    let folder_path2 = "C:\\Program Files (x86)\\Steam\\steamapps\\common\\TreeOfSavior\\patch";

    let start_time_total = Instant::now();

    let combined_hashmap1 = process_folder(folder_path1);

    let elapsed_time1 = start_time_total.elapsed();
    println!("Processed folder 1 in {:.2?}", elapsed_time1);

    let start_time2 = Instant::now();
    let combined_hashmap2 = process_folder(folder_path2);

    let elapsed_time2 = start_time2.elapsed();
    println!("Processed folder 2 in {:.2?}", elapsed_time2);

    // Combine the results of both folders
    let mut combined_hashmap = combined_hashmap1;
    combined_hashmap.extend(combined_hashmap2);

    let elapsed_time_total = start_time_total.elapsed();
    println!("Combined HashMap Length: {:?}", combined_hashmap.len());
    println!("Total processing time: {:.2?}", elapsed_time_total);
}
