use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::ipf::IPFFileTable;

#[derive(Debug)]
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
}

// Usage
pub fn build_tree(grouped: BTreeMap<String, Vec<IPFFileTable>>) -> Folder {
    let mut root = Folder::new();

    for (dir, mut files) in grouped.into_iter() {
        // Move each file into the tree
        for file in files.drain(..) {
            root.insert(&dir, file);
        }
    }

    root
}
