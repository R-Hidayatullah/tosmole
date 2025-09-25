use crate::ipf::{IPFFileTable, IPFRoot};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug)]
pub struct TreeNode {
    pub name: String,
    pub children: HashMap<String, TreeNode>,
    pub files: Vec<IPFFileTable>,
}

impl TreeNode {
    pub fn new(name: &str) -> Self {
        TreeNode {
            name: name.to_string(),
            children: HashMap::new(),
            files: Vec::new(),
        }
    }

    pub fn insert_file(&mut self, file: IPFFileTable) {
        let parts: Vec<&str> = file.directory_name.split('/').collect();
        if parts.is_empty() {
            return;
        }

        let mut current = self;
        let version_id = file
            .file_path
            .as_ref()
            .map(|p| p.file_name().unwrap().to_string_lossy())
            .unwrap_or_else(|| "unknown".into());

        for part in &parts[..parts.len() - 1] {
            let key = format!("{} ({})", part, version_id);
            current = current
                .children
                .entry(key.clone())
                .or_insert(TreeNode::new(part));
        }

        // Last part is file, move it directly
        let file_name = parts[parts.len() - 1].to_string();
        let mut moved_file = file;
        moved_file.directory_name = file_name;
        current.files.push(moved_file);
    }

    /// Print the tree recursively (full)
    pub fn print_full(&self, indent: usize) {
        let pad = " ".repeat(indent * 2);
        println!("{}{}", pad, self.name);

        for child in self.children.values() {
            child.print_full(indent + 1);
        }

        for file in &self.files {
            println!("{}  [file] {}", pad, file.directory_name);
        }
    }

    /// Print only immediate children of this node (shallow)
    pub fn print_shallow_with_parent(&self, parent: Option<&str>) {
        if let Some(p) = parent {
            println!("Parent folder: {}", p);
        }
        println!("Folder: {}", self.name);
        for child in self.children.values() {
            println!("  [folder] {}", child.name);
        }
        for file in &self.files {
            println!("  [file] {}", file.directory_name);
        }
    }

    pub fn print_shallow(&self) {
        println!("{}", self.name);
        for child in self.children.values() {
            println!("  [folder] {}", child.name);
        }
        for file in &self.files {
            println!("  [file] {}", file.directory_name);
        }
    }

    /// Find all nodes matching a multi-part path
    pub fn find_nodes_by_path<'a>(&'a self, path_parts: &[&str]) -> Vec<&'a TreeNode> {
        if path_parts.is_empty() {
            return vec![self];
        }

        let target = path_parts[0];
        let mut results = Vec::new();

        for child in self.children.values() {
            if child.name == target {
                results.extend(child.find_nodes_by_path(&path_parts[1..]));
            }
        }

        results
    }

    /// Check if a file exists in this node (only immediate files)
    pub fn has_file(&self, file_name: &str) -> bool {
        self.files.iter().any(|f| f.directory_name == file_name)
    }

    /// Check if a file exists recursively in this node and all children
    pub fn has_file_recursive(&self, file_name: &str) -> bool {
        if self.has_file(file_name) {
            return true;
        }

        for child in self.children.values() {
            if child.has_file_recursive(file_name) {
                return true;
            }
        }

        false
    }
}

/// Build the tree from all parsed IPFs
pub fn build_versioned_tree(ipfs: Vec<(PathBuf, IPFRoot)>) -> TreeNode {
    let mut root = TreeNode::new("root");

    for (_, ipf) in ipfs {
        for file in ipf.file_table {
            root.insert_file(file); // moved here
        }
    }

    root
}
