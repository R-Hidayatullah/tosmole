#![allow(dead_code)]

use std::collections::HashMap;
use std::fs::File;

use byteorder::ReadBytesExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TreeNode {
    pub(crate) name: String,
    pub(crate) children: HashMap<String, TreeNode>,
}

impl TreeNode {
    pub(crate) fn new(name: &str) -> TreeNode {
        TreeNode {
            name: name.to_string(),
            children: HashMap::new(),
        }
    }

    // Function to insert a path into the tree
    pub(crate) fn insert_path(&mut self, path: &str) {
        let parts: Vec<&str> = path.split('/').collect();
        let mut current_node = self;

        for part in parts {
            if part.is_empty() {
                continue;
            }

            current_node = current_node
                .children
                .entry(part.to_string())
                .or_insert(TreeNode::new(part));
        }
    }

    // Function to display the tree
    pub(crate) fn display(&self, depth: usize) {
        let indent = "  ".repeat(depth);
        println!("{}{}", indent, self.name);

        for child in self.children.values() {
            child.display(depth + 1);
        }
    }
}

pub(crate) fn ipf_read_string(file: &mut File, length: u16) -> String {
    let mut text = String::new();
    for _ in 0..length {
        let character = file.read_u8().unwrap();
        text.push(character as char);
    }
    text
}
