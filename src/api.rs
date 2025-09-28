use actix_files::NamedFile;
use actix_web::{HttpResponse, Responder, get, web};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::category::Folder;
use crate::ies::IESRoot;
use crate::ipf::IPFFileTable;
use crate::xml::{self, DuplicateEntry};

/// -------------------------
/// Startup Info Endpoint
/// -------------------------
#[derive(Debug, Serialize)]
pub struct InfoResponse {
    pub game_root: String,
    pub total_files: usize,
    pub duplicates_xac: usize,
    pub duplicates_xsm: usize,
    pub duplicates_xsmtime: usize,
    pub duplicates_xpm: usize,
    pub duplicates_dds: usize,
}

pub struct Duplicates {
    pub xac: Arc<Vec<DuplicateEntry>>,
    pub xsm: Arc<Vec<DuplicateEntry>>,
    pub xsmtime: Arc<Vec<DuplicateEntry>>,
    pub xpm: Arc<Vec<DuplicateEntry>>,
    pub dds: Arc<Vec<DuplicateEntry>>,
}

#[get("/api/info")]
pub async fn api_info(
    folder_tree: web::Data<Arc<Folder>>,
    game_root: web::Data<PathBuf>,
    duplicates: web::Data<Duplicates>,
) -> impl Responder {
    let total_files = folder_tree.files.len()
        + folder_tree
            .subfolders
            .values()
            .map(|f| f.files.len())
            .sum::<usize>();
    let game_root_data = game_root.to_str().unwrap().to_string();

    HttpResponse::Ok().json(InfoResponse {
        game_root: game_root_data,
        total_files,
        duplicates_xac: duplicates.xac.len(),
        duplicates_xsm: duplicates.xsm.len(),
        duplicates_xsmtime: duplicates.xsmtime.len(),
        duplicates_xpm: duplicates.xpm.len(),
        duplicates_dds: duplicates.dds.len(),
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
pub struct ShallowSearchResponse<'a> {
    pub folder_name: &'a str,
    pub subfolders: Vec<&'a str>,
    pub files: Vec<&'a str>,
}

#[get("/api/folder/shallow")]
pub async fn folder_shallow(
    query: web::Query<ShallowSearchQuery>,
    folder_tree: web::Data<Arc<Folder>>,
) -> impl Responder {
    if let Some((subfolders, files)) = folder_tree.search_folder_shallow(&query.folder_name) {
        HttpResponse::Ok().json(ShallowSearchResponse {
            folder_name: &query.folder_name,
            subfolders: subfolders.iter().map(|s| s.as_str()).collect(),
            files: files.iter().map(|f| f.as_str()).collect(),
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
) -> impl Responder {
    let results = folder_tree.search_file_by_full_path(&query.path);
    let version = query.version.unwrap_or(0); // default to version 0

    let (_full_path, file_table) = match results.get(version) {
        Some(entry) => entry,
        None => return HttpResponse::NotFound().body("File/version not found"),
    };

    // Determine file type by extension
    let ext = _full_path.split('.').last().unwrap_or("").to_lowercase();

    // Extract raw data from the IPF
    let data = match file_table.extract_data() {
        Ok(d) => d,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to extract file data"),
    };

    match ext.as_str() {
        "ies" => {
            if let Ok(ies) = IESRoot::from_bytes(&data) {
                HttpResponse::Ok().json(ies)
            } else {
                HttpResponse::InternalServerError().body("Failed to parse IES file")
            }
        }
        "xml" => {
            let text = String::from_utf8_lossy(&data); // works for &[u8]
            HttpResponse::Ok()
                .content_type("text/plain")
                .body(text.to_string())
        }

        "lua" => {
            if let Ok(text) = String::from_utf8(data) {
                HttpResponse::Ok().content_type("text/plain").body(text)
            } else {
                HttpResponse::InternalServerError().body("Failed to read Lua file")
            }
        }

        "png" => HttpResponse::Ok().content_type("image/png").body(data),
        "jpg" | "jpeg" => HttpResponse::Ok().content_type("image/jpeg").body(data),
        "bmp" => HttpResponse::Ok().content_type("image/bmp").body(data),
        "tga" => HttpResponse::Ok().content_type("image/x-tga").body(data),

        _ => HttpResponse::Ok()
            .content_type("application/octet-stream")
            .body(data), // fallback for unknown types
    }
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
