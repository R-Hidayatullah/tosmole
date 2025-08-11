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

fn main() -> io::Result<()> {
    let game_root = Path::new(r"C:\Users\Ridwan Hidayatullah\Documents\TreeOfSaviorCN");
    let start = std::time::Instant::now();

    // Use max 4 threads per folder
    let parsed_ipfs = parse_game_folders_multithread_limited(game_root, 4)?;

    let duration = start.elapsed();

    println!(
        "Parsed total {} IPF archives from both 'data' and 'patch' in {:.2?}",
        parsed_ipfs.len(),
        duration,
    );

    Ok(())
}
