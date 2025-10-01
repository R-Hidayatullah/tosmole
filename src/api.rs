use actix_files::NamedFile;
use actix_web::{HttpResponse, Responder, get, web};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use tera::{Context, Tera};

use crate::category::Folder;
use crate::ies::IESRoot;
use crate::ipf::FileSizeStats;
use crate::ipf::IPFFileTable;
use crate::xac::XACRoot;
use crate::xml;

/// -------------------------
/// Startup Info Endpoint
/// -------------------------
#[derive(Debug, Serialize)]
pub struct InfoResponse {
    pub game_root: String,
    pub duplicates_xac: usize,
    pub duplicates_xsm: usize,
    pub duplicates_xsmtime: usize,
    pub duplicates_xpm: usize,
    pub duplicates_dds: usize,
    pub count_duplicated: u32,
    pub count_unique: u32,
    pub compressed_lowest: u32,
    pub compressed_highest: u32,
    pub compressed_avg: u32,
    pub uncompressed_lowest: u32,
    pub uncompressed_highest: u32,
    pub uncompressed_avg: u32,
}

pub struct Duplicates {
    pub xac: Arc<HashMap<String, String>>,
    pub xsm: Arc<HashMap<String, String>>,
    pub xsmtime: Arc<HashMap<String, String>>,
    pub xpm: Arc<HashMap<String, String>>,
    pub dds: Arc<HashMap<String, String>>,
}

#[get("/api/info")]
pub async fn api_info(
    folder_tree: web::Data<Arc<Folder>>,
    game_root: web::Data<PathBuf>,
    file_stats: web::Data<FileSizeStats>,
    duplicates: web::Data<Duplicates>,
) -> impl Responder {
    let game_root_data = game_root.to_str().unwrap().to_string();

    HttpResponse::Ok().json(InfoResponse {
        game_root: game_root_data,
        duplicates_xac: duplicates.xac.len(),
        duplicates_xsm: duplicates.xsm.len(),
        duplicates_xsmtime: duplicates.xsmtime.len(),
        duplicates_xpm: duplicates.xpm.len(),
        duplicates_dds: duplicates.dds.len(),
        count_duplicated: file_stats.count_duplicated,
        count_unique: file_stats.count_unique,
        compressed_lowest: file_stats.compressed_lowest,
        compressed_highest: file_stats.compressed_highest,
        compressed_avg: file_stats.compressed_avg,
        uncompressed_lowest: file_stats.uncompressed_lowest,
        uncompressed_highest: file_stats.uncompressed_highest,
        uncompressed_avg: file_stats.uncompressed_avg,
    })
}

/// -------------------------
/// Shallow Folder Search
/// -------------------------
#[derive(Debug, Deserialize)]
pub struct ShallowSearchQuery {
    pub folder_name: String,
}

#[derive(Debug, Serialize)]
pub struct ShallowSearchResponse {
    pub folder_name: String,
    pub subfolders: Vec<String>,
    pub files: Vec<String>,
}

#[get("/api/folder/shallow")]
pub async fn folder_shallow(
    query: web::Query<ShallowSearchQuery>,
    folder_tree: web::Data<Arc<Folder>>,
) -> impl Responder {
    if let Some((subfolders, files)) = folder_tree.search_folder_shallow(&query.folder_name) {
        // deduplicate subfolders
        let subfolders: Vec<String> = subfolders
            .iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .cloned()
            .collect();

        // deduplicate files
        let files: Vec<String> = files
            .iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .cloned()
            .collect();

        HttpResponse::Ok().json(ShallowSearchResponse {
            folder_name: query.folder_name.clone(),
            subfolders,
            files,
        })
    } else {
        HttpResponse::NotFound().body("Folder not found")
    }
}

/// -------------------------
/// Recursive File Search
/// -------------------------
#[derive(Debug, Deserialize)]
pub struct FileSearchQuery {
    pub file_name: String,
}

#[derive(Debug, Serialize)]
pub struct FileSearchItemVersioned<'a> {
    pub version: usize, // version index
    pub file_path: &'a str,
    pub download_url: String,
    pub parse_url: String,
}

#[derive(Debug, Serialize)]
pub struct RecursiveSearchResponseVersioned<'a> {
    pub file_name: &'a str,
    pub found_files: Vec<FileSearchItemVersioned<'a>>,
}

#[get("/api/file/search")]
pub async fn search_file_recursive(
    query: web::Query<FileSearchQuery>,
    folder_tree: web::Data<Arc<Folder>>,
) -> impl Responder {
    let results = folder_tree.search_file_recursive(&query.file_name, "");

    let items: Vec<FileSearchItemVersioned> = results
        .iter()
        .enumerate() // enumerate to get version
        .map(
            |(version, (full_path, _file_table))| FileSearchItemVersioned {
                version,
                file_path: full_path.as_str(),
                download_url: format!("/api/file/download?path={}&version={}", full_path, version),
                parse_url: format!("/api/file/parse?path={}&version={}", full_path, version),
            },
        )
        .collect();

    HttpResponse::Ok().json(RecursiveSearchResponseVersioned {
        file_name: &query.file_name,
        found_files: items,
    })
}

/// -------------------------
/// Full Path File Search
/// -------------------------
#[derive(Debug, Deserialize)]
pub struct FileFullPathQuery {
    pub full_path: String,
}

#[derive(Debug, Serialize)]
pub struct FileFullPathInfo<'a> {
    pub version: usize, // 0, 1, 2 ... based on vector index
    pub file_path: String,
    pub container_name: &'a str,
    pub crc32: u32,
    pub file_size_compressed: u32,
    pub file_size_uncompressed: u32,
    pub file_pointer: u32, // offset in the IPF archive
    pub download_url: String,
    pub parse_url: String,
}
#[get("/api/file/fullpath")]
pub async fn search_file_fullpath(
    query: web::Query<FileFullPathQuery>,
    folder_tree: web::Data<Arc<Folder>>,
) -> impl Responder {
    let results = folder_tree.search_file_by_full_path(&query.full_path);

    if results.is_empty() {
        return HttpResponse::NotFound().body("File not found");
    }

    let items: Vec<FileFullPathInfo> = results
        .iter()
        .enumerate() // <-- get vector index for version
        .map(|(version, (_full_path, file_table))| FileFullPathInfo {
            version,
            file_path: file_table
                .file_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "".to_string()),
            container_name: &file_table.container_name,
            crc32: file_table.crc32,
            file_size_compressed: file_table.file_size_compressed,
            file_size_uncompressed: file_table.file_size_uncompressed,
            file_pointer: file_table.file_pointer,
            download_url: format!(
                "/api/file/download?path={}&version={}",
                query.full_path, version
            ),
            parse_url: format!(
                "/api/file/parse?path={}&version={}",
                query.full_path, version
            ),
        })
        .collect();

    HttpResponse::Ok().json(items)
}

/// -------------------------
/// Download Raw Binary File
/// -------------------------
#[derive(Debug, Deserialize)]
pub struct FileDownloadQuery {
    pub path: String,
    #[serde(default)]
    pub version: Option<usize>, // optional, default to 0
}

#[get("/api/file/download")]
pub async fn download_file(
    query: web::Query<FileDownloadQuery>,
    folder_tree: web::Data<Arc<Folder>>,
) -> impl Responder {
    let results = folder_tree.search_file_by_full_path(&query.path);

    let version = query.version.unwrap_or(0); // default to 0
    if let Some((_full_path, file_table)) = results.get(version) {
        if let Ok(data) = file_table.extract_data() {
            let filename = file_table.directory_name.as_str();
            return HttpResponse::Ok()
                .insert_header((
                    "Content-Disposition",
                    format!("attachment; filename=\"{}\"", filename),
                ))
                .content_type("application/octet-stream")
                .body(data);
        }
    }

    HttpResponse::NotFound().body("File not found")
}

/// -------------------------
/// Parse as IES
/// -------------------------
#[get("/api/file/parse")]
pub async fn parse_file_as_ies(
    query: web::Query<FileDownloadQuery>,
    folder_tree: web::Data<Arc<Folder>>,
) -> impl Responder {
    let results = folder_tree.search_file_by_full_path(&query.path);

    let version = query.version.unwrap_or(0); // default to 0
    if let Some((_full_path, file_table)) = results.get(version) {
        if let Ok(data) = file_table.extract_data() {
            if let Ok(ies) = IESRoot::from_bytes(&data) {
                return HttpResponse::Ok().json(ies);
            }
        }
    }

    HttpResponse::InternalServerError().body("Failed to parse as IES")
}

#[derive(Debug, Deserialize)]
pub struct FilePreviewQuery {
    pub path: String,
    pub version: Option<usize>,
}

#[get("/api/file/preview")]
pub async fn preview_file(
    query: web::Query<FilePreviewQuery>,
    folder_tree: web::Data<Arc<Folder>>,
    mesh_map: web::Data<HashMap<String, String>>,
) -> impl Responder {
    // Find file by full path
    let results = folder_tree.search_file_by_full_path(&query.path);
    let version = query.version.unwrap_or(0);

    let (_full_path, file_table) = match results.get(version) {
        Some(entry) => entry,
        None => return HttpResponse::NotFound().body("File/version not found"),
    };

    // Extract raw file bytes
    let data = match file_table.extract_data() {
        Ok(d) => d,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to extract file data"),
    };

    // Get extension
    let ext = _full_path.split('.').last().unwrap_or("").to_lowercase();

    // Group image formats
    let image_extensions = ["tga", "png", "jpg", "jpeg", "bmp", "dds"];

    if image_extensions.contains(&ext.as_str()) {
        // TGA conversion
        if ext == "tga" {
            return match crate::stb::load_tga_from_memory(&data) {
                Some(img) => match crate::stb::encode_png_to_memory(&img) {
                    Some(png_bytes) => HttpResponse::Ok().content_type("image/png").body(png_bytes),
                    None => {
                        HttpResponse::InternalServerError().body("Failed to encode PNG from TGA")
                    }
                },
                None => HttpResponse::InternalServerError().body("Failed to decode TGA image"),
            };
        }

        // Detect MIME type via magic bytes for other images
        let mime_type = if data.starts_with(b"\x89PNG\r\n\x1a\n") {
            "image/png"
        } else if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            "image/jpeg"
        } else if data.starts_with(b"BM") {
            "image/bmp"
        } else if data.len() > 4 && &data[0..4] == b"DDS " {
            "image/dds"
        } else {
            "application/octet-stream"
        };

        return HttpResponse::Ok().content_type(mime_type).body(data);
    }

    // MP3 audio
    if ext == "mp3" {
        return HttpResponse::Ok().content_type("audio/mpeg").body(data);
    }

    // Fonts
    if ext == "ttf" {
        return HttpResponse::Ok().content_type("font/ttf").body(data);
    }

    // IES format
    if ext == "ies" {
        return match IESRoot::from_bytes(&data) {
            Ok(ies) => HttpResponse::Ok().json(ies),
            Err(_) => HttpResponse::InternalServerError().body("Failed to parse IES file"),
        };
    }

    // XAC format
    if ext == "xac" {
        match crate::xac::XACRoot::from_bytes(&data) {
            Ok(xac_root) => {
                // Try to get texture path
                let texture_path = match mesh_map.get(_full_path) {
                    Some(path) => path.clone(),
                    None => {
                        // Fallback: replace char_hi with char_texture
                        let fallback = {
                            // Replace char_hi -> char_texture
                            let mut path = _full_path.replace("char_hi", "char_texture");

                            // Remove filename, keep folder path only
                            path = match path.rfind('/') {
                                Some(idx) => path[..idx].to_string(),
                                None => path,
                            };

                            // Ensure it ends with '/'
                            if !path.ends_with('/') {
                                path.push('/');
                            }

                            path
                        };

                        println!(
                            "No texture path found for {} — using fallback folder {}",
                            _full_path, fallback
                        );
                        fallback
                    }
                };

                let scene = crate::mesh::Scene::from_xac_root(&xac_root, texture_path);
                return HttpResponse::Ok().json(scene);
            }
            Err(_) => return HttpResponse::InternalServerError().body("Failed to parse XAC file"),
        }
    }

    // Text-like formats
    let text_extensions = [
        "xml", "skn", "3dprop", "3dworld", "3drender", "3deffect", "x", "fx", "fxh", "sani",
        "effect", "json", "atlas", "sprbin", "xsd", "lua", "lst", "export",
    ];

    if text_extensions.contains(&ext.as_str()) {
        let text = String::from_utf8_lossy(&data);
        return HttpResponse::Ok()
            .content_type("text/plain")
            .body(text.to_string());
    }

    // Fallback binary
    HttpResponse::Ok()
        .content_type("application/octet-stream")
        .body(data)
}

/// -------------------------
/// Initialize API Routes
/// -------------------------
pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(api_info);
    cfg.service(folder_shallow);
    cfg.service(search_file_recursive);
    cfg.service(search_file_fullpath);
    cfg.service(download_file);
    cfg.service(parse_file_as_ies);
    cfg.service(preview_file);
}
