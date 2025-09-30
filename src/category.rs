use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::ipf::IPFFileTable;

#[derive(Debug, Serialize, Deserialize)]
pub struct Folder {
    pub files: Vec<IPFFileTable>,
    pub subfolders: BTreeMap<String, Folder>,
}

impl Folder {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            subfolders: BTreeMap::new(),
        }
    }

    /// Recursively insert a file into the tree based on path parts
    pub fn insert(&mut self, path: &str, file: IPFFileTable) {
        let mut parts = path.split('/').peekable();

        if let Some(part) = parts.next() {
            if parts.peek().is_none() {
                // Leaf node: insert file here
                self.files.push(file);
            } else {
                // Intermediate folder
                let folder = self
                    .subfolders
                    .entry(part.to_string())
                    .or_insert_with(Folder::new);
                let rest = parts.collect::<Vec<_>>().join("/");
                folder.insert(&rest, file);
            }
        }
    }

    /// Optional: print tree for debugging
    pub fn print(&self, prefix: &str) {
        for (name, folder) in &self.subfolders {
            println!("{}Folder: {}", prefix, name);
            folder.print(&format!("{}  ", prefix));
        }
        for file in &self.files {
            println!("{}File: {:?}", prefix, file.file_path.as_ref().unwrap());
        }
    }

    /// Print tree with optional limit for folders and files
    pub fn print_limited(&self, prefix: &str, folder_limit: usize, file_limit: usize) {
        // Print subfolders
        for (i, (name, folder)) in self.subfolders.iter().enumerate() {
            if i >= folder_limit {
                println!(
                    "{}... ({} more folders)",
                    prefix,
                    self.subfolders.len() - folder_limit
                );
                break;
            }
            println!("{}Folder: {}", prefix, name);
            folder.print_limited(&format!("{}  ", prefix), folder_limit, file_limit);
        }

        // Print files
        for (i, file) in self.files.iter().enumerate() {
            if i >= file_limit {
                println!(
                    "{}... ({} more files)",
                    prefix,
                    self.files.len() - file_limit
                );
                break;
            }
            println!("{}File: {:?}", prefix, file.file_path.as_ref().unwrap());
        }
    }

    /// Shallow search for a folder: returns subfolder names and files directly inside it
    pub fn search_folder_shallow(&self, folder_path: &str) -> Option<(Vec<String>, Vec<String>)> {
        // Normalize path: remove trailing slash
        let path = folder_path.trim_end_matches('/');

        if path.is_empty() {
            // Return root folder content
            let subfolders: Vec<String> = self.subfolders.keys().cloned().collect();
            let files: Vec<String> = self
                .files
                .iter()
                .map(|f| f.directory_name.clone())
                .collect();
            return Some((subfolders, files));
        }

        // Traverse the path parts
        let mut current = self;
        for part in path.split('/') {
            if let Some(subfolder) = current.subfolders.get(part) {
                current = subfolder;
            } else {
                return None; // folder not found
            }
        }

        // Collect shallow content of the target folder
        let subfolders: Vec<String> = current.subfolders.keys().cloned().collect();
        let files: Vec<String> = current
            .files
            .iter()
            .map(|f| f.directory_name.clone())
            .collect();

        Some((subfolders, files))
    }

    /// Recursive search for files matching `file_name`, returns full path and reference
    pub fn search_file_recursive<'a>(
        &'a self,
        file_name: &str,
        current_path: &str,
    ) -> Vec<(String, &'a IPFFileTable)> {
        let mut results = Vec::new();

        // Check files in current folder
        for f in &self.files {
            if f.directory_name.to_lowercase().contains(file_name) {
                let full_path = if current_path.is_empty() {
                    f.directory_name.clone()
                } else {
                    format!("{}/{}", current_path, f.directory_name)
                };
                results.push((full_path, f));
            }
        }

        // Recurse into subfolders
        for (name, folder) in &self.subfolders {
            let path = if current_path.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", current_path, name)
            };
            results.extend(folder.search_file_recursive(file_name, &path));
        }

        results
    }

    /// Search file by full path, e.g., "ui/brush/spraycursor_1.tga"
    pub fn search_file_by_full_path<'a>(
        &'a self,
        full_path: &str,
    ) -> Vec<(String, &'a IPFFileTable)> {
        let mut results = Vec::new();
        let parts: Vec<&str> = full_path.split('/').collect();
        self.search_file_by_parts(&parts, "", &mut results);
        results
    }

    fn search_file_by_parts<'a>(
        &'a self,
        parts: &[&str],
        current_path: &str,
        results: &mut Vec<(String, &'a IPFFileTable)>,
    ) {
        if parts.is_empty() {
            return;
        }

        if parts.len() == 1 {
            // Last part = filename
            let filename = parts[0];
            for file in &self.files {
                if file.directory_name == filename {
                    let full_path = if current_path.is_empty() {
                        filename.to_string()
                    } else {
                        format!("{}/{}", current_path, filename)
                    };
                    results.push((full_path, file));
                }
            }
        } else {
            // Intermediate folder
            let folder_name = parts[0];
            if let Some(subfolder) = self.subfolders.get(folder_name) {
                let new_path = if current_path.is_empty() {
                    folder_name.to_string()
                } else {
                    format!("{}/{}", current_path, folder_name)
                };
                subfolder.search_file_by_parts(&parts[1..], &new_path, results);
            }
        }
    }
}

/// Print shallow folder view, showing top N subfolders and M files per folder
pub fn print_shallow_tree(root: &Folder, max_subfolders: usize, max_files: usize) {
    println!("Root folder:");

    // Show subfolders (shallow)
    for (i, (folder_name, folder_node)) in root.subfolders.iter().enumerate() {
        if i >= max_subfolders {
            break;
        }
        println!("  Folder: {}", folder_name);

        // Show first few files in this subfolder
        for (j, file) in folder_node.files.iter().enumerate() {
            if j >= max_files {
                break;
            }
            println!("    File: {:?}", file.directory_name);
        }
    }

    // Also show root-level files
    println!("  Root files:");
    for (i, file) in root.files.iter().enumerate() {
        if i >= max_files {
            break;
        }
        println!("    File: {:?}", file.directory_name);
    }
}

// Usage
pub fn build_tree(grouped: BTreeMap<String, Vec<IPFFileTable>>) -> Folder {
    let mut root = Folder::new();

    for (dir, mut files) in grouped.into_iter() {
        for mut file in files.drain(..) {
            // Keep only the filename in directory_name
            if let Some(name) = std::path::Path::new(&file.directory_name)
                .file_name()
                .and_then(|s| s.to_str())
            {
                file.directory_name = name.to_string();
            }

            root.insert(&dir, file);
        }
    }

    root
}
