#![allow(unused_variables, unused_imports, dead_code)]
use std::{
    fs::read_dir,
    io,
    path::{Path, PathBuf},
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

/// Parse all `.ipf` files in a directory concurrently.
fn parse_all_ipf_files_multithread(dir: &Path) -> io::Result<Vec<(PathBuf, IPFRoot)>> {
    let ipf_paths: Vec<PathBuf> = read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "ipf"))
        .collect();

    let mut handles = Vec::with_capacity(ipf_paths.len());

    for ipf_path in ipf_paths {
        let path_clone = ipf_path.clone();

        let handle = thread::spawn(move || -> io::Result<(PathBuf, IPFRoot)> {
            let ipf = IPFRoot::open(&path_clone)?;
            Ok((path_clone, ipf))
        });

        handles.push(handle);
    }

    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        let parsed = handle.join().map_err(|_| {
            io::Error::new(io::ErrorKind::Other, "Thread panicked during IPF parsing")
        })??;
        results.push(parsed);
    }

    Ok(results)
}

/// Parse both `/data` and `/patch` folders under the game root directory concurrently.
fn parse_game_folders_multithread(game_root: &Path) -> io::Result<Vec<(PathBuf, IPFRoot)>> {
    let data_dir = game_root.join("data");
    let patch_dir = game_root.join("patch");

    println!(
        "Starting to parse all IPF files in folders:\n  {}\n  {}",
        data_dir.display(),
        patch_dir.display()
    );

    let handle_data = thread::spawn(move || parse_all_ipf_files_multithread(&data_dir));
    let handle_patch = thread::spawn(move || parse_all_ipf_files_multithread(&patch_dir));

    let parsed_data = handle_data.join().expect("Data thread panicked")?;
    let parsed_patch = handle_patch.join().expect("Patch thread panicked")?;

    let mut all_parsed = parsed_data;
    all_parsed.extend(parsed_patch);

    Ok(all_parsed)
}

fn main() -> io::Result<()> {
    // Pass the root folder for the game data, e.g.:
    let game_root = Path::new(r"C:\Users\Ridwan Hidayatullah\Documents\TreeOfSaviorCN");

    let start = Instant::now();

    let parsed_ipfs = parse_game_folders_multithread(game_root)?;

    let duration = start.elapsed();

    println!(
        "Parsed total {} IPF archives from both 'data' and 'patch' in {:.2?}",
        parsed_ipfs.len(),
        duration,
    );

    Ok(())
}
