#![allow(unused_variables, unused_imports, dead_code)]
use std::{
    fs::read_dir,
    io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Instant,
};

use crate::ipf::IPFRoot;

mod binary;
mod ies;
mod ipf;
mod shared_formats;
mod xac;
mod xpm;
mod xsm;

fn print_hex_viewer(data: &[u8]) {
    const BYTES_PER_LINE: usize = 16;

    for (i, chunk) in data.chunks(BYTES_PER_LINE).enumerate() {
        // Offset decimal (8 digits padded)
        print!("{:08}  ", i * BYTES_PER_LINE);

        // Hex bytes uppercase
        for b in chunk.iter() {
            print!("{:02X} ", b);
        }

        // Pad hex if last line shorter
        let pad_spaces = (BYTES_PER_LINE - chunk.len()) * 3;
        for _ in 0..pad_spaces {
            print!(" ");
        }

        // ASCII chars or '.' if non-printable
        print!(" ");
        for &b in chunk.iter() {
            let c = if b.is_ascii_graphic() || b == b' ' {
                b as char
            } else {
                '.'
            };
            print!("{}", c);
        }

        println!();
    }
}

/// Parse all `.ipf` files in `dir` using a fixed number of worker threads.
/// Returns Vec<(PathBuf, IPFRoot)>
fn parse_all_ipf_files_limited_threads(
    dir: &Path,
    max_threads: usize,
) -> io::Result<Vec<(PathBuf, IPFRoot)>> {
    // Collect all .ipf paths first
    let ipf_paths: Vec<PathBuf> = read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "ipf"))
        .collect();

    let (tx_paths, rx_paths) = mpsc::channel::<PathBuf>();
    let (tx_results, rx_results) = mpsc::channel::<io::Result<(PathBuf, IPFRoot)>>();

    // Wrap receiver in Arc<Mutex<>> to share among workers safely
    let rx_paths = Arc::new(Mutex::new(rx_paths));

    // Spawn worker threads limited by max_threads
    let mut workers = Vec::with_capacity(max_threads);
    for _ in 0..max_threads {
        let rx_paths = Arc::clone(&rx_paths);
        let tx_results = tx_results.clone();

        let worker = thread::spawn(move || {
            loop {
                let path = {
                    let lock = rx_paths.lock().unwrap();
                    lock.recv()
                };

                match path {
                    Ok(ipf_path) => {
                        let res = IPFRoot::open(&ipf_path).map(|ipf| (ipf_path, ipf));
                        // Send back result, ignoring if receiver dropped
                        let _ = tx_results.send(res);
                    }
                    Err(_) => break, // channel closed, no more work
                }
            }
        });

        workers.push(worker);
    }
    let path_count = ipf_paths.len();

    for path in &ipf_paths {
        tx_paths.send(path.clone()).unwrap();
    }
    drop(tx_paths); // close sender to signal no more tasks

    // Collect results from workers
    let mut results = Vec::new();
    for _ in 0..path_count {
        // Propagate first error if any
        results.push(rx_results.recv().unwrap()?);
    }

    // Join worker threads
    for worker in workers {
        worker.join().expect("Worker thread panicked");
    }

    Ok(results)
}

/// Wrapper to parse both folders with limited threads each.
/// You can adjust max_threads for each folder if you want.
fn parse_game_folders_multithread_limited(
    game_root: &Path,
    max_threads: usize,
) -> io::Result<Vec<(PathBuf, IPFRoot)>> {
    let data_dir = game_root.join("data");
    let patch_dir = game_root.join("patch");

    println!(
        "Starting to parse all IPF files in folders:\n  {}\n  {}",
        data_dir.display(),
        patch_dir.display()
    );

    // Spawn threads for both folders concurrently
    let handle_data =
        thread::spawn(move || parse_all_ipf_files_limited_threads(&data_dir, max_threads));
    let handle_patch =
        thread::spawn(move || parse_all_ipf_files_limited_threads(&patch_dir, max_threads));

    let parsed_data = handle_data.join().expect("Data thread panicked")?;
    let parsed_patch = handle_patch.join().expect("Patch thread panicked")?;

    let mut all_parsed = parsed_data;
    all_parsed.extend(parsed_patch);

    Ok(all_parsed)
}

fn extract_file_from_ipf(
    parsed_ipfs: &mut [(PathBuf, crate::ipf::IPFRoot)],
    ipf_filename: &str,
    file_index: usize,
) -> io::Result<Option<(crate::ipf::IPFFileTable, Vec<u8>)>> {
    if let Some((path, ipf)) = parsed_ipfs.iter_mut().find(|(p, _)| {
        p.file_name()
            .and_then(|osstr| osstr.to_str())
            .map_or(false, |name| name.eq_ignore_ascii_case(ipf_filename))
    }) {
        println!("Found IPF archive: {:?}", path);

        if ipf.file_table.len() > file_index {
            let file_entry = ipf.file_table[file_index].clone(); // clone here
            if let Some(result) = ipf.extract_file_if_available(file_index) {
                let data = result?;
                return Ok(Some((file_entry, data)));
            } else {
                println!("Extraction not available for this IPF archive (no internal reader).");
                return Ok(None);
            }
        } else {
            println!(
                "File table has fewer than {} files in archive {}.",
                file_index + 1,
                ipf_filename
            );
            return Ok(None);
        }
    }

    println!("IPF archive '{}' not found in parsed list.", ipf_filename);
    Ok(None)
}

fn main() -> io::Result<()> {
    let game_root = Path::new(r"C:\Users\Ridwan Hidayatullah\Documents\TreeOfSaviorCN");
    let start = std::time::Instant::now();

    let mut parsed_ipfs = parse_game_folders_multithread_limited(game_root, 4)?;

    let duration = start.elapsed();

    println!(
        "Parsed total {} IPF archives from both 'data' and 'patch' in {:.2?}",
        parsed_ipfs.len(),
        duration,
    );

    // Example: extract file at index 7 from archive named "example.ipf"
    let ipf_name = "sound.ipf";
    let file_index = 5;

    match extract_file_from_ipf(&mut parsed_ipfs, ipf_name, file_index)? {
        Some((file_entry, data)) => {
            println!("Extracted from archive '{}':", ipf_name);
            println!("Directory in archive: {}", file_entry.directory_name);
            println!("Filename in archive: {}", file_entry.container_name);
            println!("Extracted data length: {}", data.len());
            println!("Hex view of extracted data (up to 100 bytes):");
            print_hex_viewer(&data[..data.len().min(100)]);
        }
        None => {
            println!(
                "Could not extract file {} from IPF archive '{}'.",
                file_index, ipf_name
            );
        }
    }

    Ok(())
}
