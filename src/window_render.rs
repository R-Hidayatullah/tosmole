#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};

use eframe::egui;

#[derive(Debug, Clone)]
enum FileEntry {
    File(PathBuf),
    Directory(PathBuf, Vec<FileEntry>),
}

enum FileViewer {
    Binary(PathBuf),
    None,
}

struct MyApp {
    file_tree: Vec<FileEntry>,
    selected_file: FileViewer,
    dock: egui::CentralPanel,
}

impl Default for MyApp {
    fn default() -> Self {
        let root_path = std::env::current_dir().unwrap_or_default();
        let file_tree = Self::build_file_tree(&root_path);
        let dock = egui::CentralPanel::default();
        Self {
            file_tree,
            selected_file: FileViewer::None,
            dock,
        }
    }
}

impl MyApp {
    fn build_file_tree(directory: &Path) -> Vec<FileEntry> {
        let mut file_tree = Vec::new();

        if let Ok(entries) = std::fs::read_dir(directory) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        file_tree.push(FileEntry::File(path));
                    } else if path.is_dir() {
                        let sub_directory = Self::build_file_tree(&path);
                        file_tree.push(FileEntry::Directory(path, sub_directory));
                    }
                }
            }
        }

        file_tree
    }

    fn build_ui_for_file_tree(&mut self, ui: &mut egui::Ui, entries: &[FileEntry]) {
        for entry in entries {
            match entry {
                FileEntry::File(path) => {
                    ui.horizontal(|ui| {
                        ui.label(path.file_name().unwrap_or_default().to_string_lossy());
                        ui.style_mut().spacing.item_spacing.x = 0.0;
                        if ui.button("Open").clicked() {
                            self.selected_file = FileViewer::Binary(path.clone());
                        }
                    });
                }
                FileEntry::Directory(path, sub_directory) => {
                    egui::CollapsingHeader::new(
                        path.file_name().unwrap_or_default().to_string_lossy(),
                    )
                    .show(ui, |ui| {
                        self.build_ui_for_file_tree(ui, sub_directory);
                    });
                }
            }
        }
    }
    fn show_binary_viewer(&mut self, ui: &mut egui::Ui) {
        match &self.selected_file {
            FileViewer::Binary(path) => {
                let binary_data = std::fs::read(path).unwrap_or_default();
                let hex_width = 2; // Fixed width for hexadecimal values

                ui.vertical(|ui| {
                    ui.label("Hexadecimal and ASCII View");

                    egui::ScrollArea::vertical()
                        .id_source("hex_ascii_scroll_area")
                        .show(ui, |ui| {
                            // Display headers
                            ui.horizontal(|ui| {
                                ui.label("Offset");
                                for i in 0..16 {
                                    ui.label(format!("{:width$X}", i, width = hex_width));
                                }
                                ui.label(""); // Spacer
                                for _ in 0..16 {
                                    ui.label(""); // Spacer
                                }
                            });

                            // Display data rows
                            for (offset, chunk) in binary_data.chunks(16).enumerate() {
                                ui.horizontal(|ui| {
                                    // Display offset
                                    ui.label(format!("{:width$X}", offset * 16, width = hex_width));

                                    // Display hexadecimal values
                                    for &byte in chunk {
                                        ui.label(format!("{:02X}", byte));
                                    }

                                    ui.label(""); // Spacer

                                    // Display ASCII representation
                                    for &byte in chunk {
                                        let ascii_char = if byte.is_ascii_graphic() {
                                            byte as char
                                        } else {
                                            '.'
                                        };
                                        ui.label(ascii_char.to_string());
                                    }
                                });
                            }
                        });
                });

                ui.separator();
            }
            FileViewer::None => {}
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("File Tree Viewer");
            ui.separator();

            egui::SidePanel::left("File Tree")
                .resizable(true)
                .show_inside(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.vertical(|ui| {
                            let file_tree_data = self.file_tree.clone();
                            self.build_ui_for_file_tree(ui, &file_tree_data);
                        });
                    });
                });

            egui::SidePanel::left("Binary Viewer")
                .resizable(true)
                .show_inside(ui, |ui| {
                    ui.vertical(|ui| {
                        self.show_binary_viewer(ui);
                    });
                });
        });
    }
}

pub fn render() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::<MyApp>::default()
        }),
    )
}
