#![allow(unused)]

use actix_web::{App, HttpServer, web};
use std::{collections::BTreeMap, io, path::PathBuf, sync::Arc};
use tera::Tera;

use category::Folder;

mod api;
mod category;
mod ies;
mod ipf;
mod tsv;
mod web_data;
mod xac;
mod xml;

#[actix_web::main]
async fn main() -> io::Result<()> {
    // ---------------------------
    // Paths
    // ---------------------------
    let game_root = PathBuf::from("/home/ridwan/Documents/TreeOfSaviorCN/");
    let lang_folder =
        PathBuf::from("/home/ridwan/Documents/TreeOfSaviorCN/release/languageData/English");

    // ---------------------------
    // Parse IPF Archives
    // ---------------------------
    println!("Parsing IPF archives...");
    let mut parsed_ipfs = ipf::parse_game_ipfs(&game_root)?;
    println!("Parsed {} IPF archives", parsed_ipfs.len());

    let mut all_files = ipf::collect_file_tables_from_parsed(&mut parsed_ipfs);
    ipf::sort_file_tables_by_folder_then_name(&mut all_files);

    let grouped: BTreeMap<String, Vec<ipf::IPFFileTable>> =
        ipf::group_file_tables_by_directory(all_files);

    let folder_tree = Arc::new(category::build_tree(grouped));

    // ---------------------------
    // Parse Language Data
    // ---------------------------
    println!("Parsing language data...");
    let (_etc_data, _item_data) = tsv::parse_language_data(&lang_folder)?;
    println!("Parsed language data.");

    // ---------------------------
    // Parse Duplicates
    // ---------------------------
    println!("Parsing duplicates...");

    let xac_duplicates = Arc::new(xml::parse_duplicates_xml(
        &game_root.join("release/xac_duplicates.xml"),
    )?);
    let xsm_duplicates = Arc::new(xml::parse_duplicates_xml(
        &game_root.join("release/xsm_duplicates.xml"),
    )?);
    let xsmtime_duplicates = Arc::new(xml::parse_duplicates_xml(
        &game_root.join("release/xsmtime_duplicates.xml"),
    )?);
    let xpm_duplicates = Arc::new(xml::parse_duplicates_xml(
        &game_root.join("release/xpm_duplicates.xml"),
    )?);
    let dds_duplicates = Arc::new(xml::parse_duplicates_xml(
        &game_root.join("release/dds_duplicates.xml"),
    )?);

    println!("Parsed duplicates.");

    let duplicates_data = web::Data::new(api::Duplicates {
        xac: xac_duplicates,
        xsm: xsm_duplicates,
        xsmtime: xsmtime_duplicates,
        xpm: xpm_duplicates,
        dds: dds_duplicates,
    });

    // ---------------------------
    // Start Actix Web Server
    // ---------------------------
    let folder_tree_data = web::Data::new(folder_tree);
    let game_root_data = web::Data::new(game_root);

    // Initialize Tera
    let tera = Tera::new("templates/**/*").expect("Failed to initialize Tera templates");
    let tera_data = web::Data::new(tera);

    println!("Starting server at http://127.0.0.1:8080 ...\n");
    println!("Available endpoints:\n");

    println!("1. GET /api/info");
    println!("   Usage: curl http://127.0.0.1:8080/api/info");
    println!("   Description: Returns game root info and duplicate counts.\n");

    println!("2. GET /api/folder/shallow?folder_name=<folder>");
    println!("   Usage: curl \"http://127.0.0.1:8080/api/folder/shallow?folder_name=ui/brush\"");
    println!(
        "   Description: Returns subfolders and files directly inside the specified folder.\n"
    );

    println!("3. GET /api/file/search?file_name=<file>");
    println!("   Usage: curl \"http://127.0.0.1:8080/api/file/search?file_name=R1.txt\"");
    println!(
        "   Description: Recursively searches for files by name and returns all matches with version indices.\n"
    );

    println!("4. GET /api/file/fullpath?full_path=<file>");
    println!("   Usage: curl \"http://127.0.0.1:8080/api/file/fullpath?full_path=ies/actor.ies\"");
    println!(
        "   Description: Search for a file by exact path, returns all available versions with version index.\n"
    );

    println!("5. GET /api/file/download?path=<file>&version=<index>");
    println!(
        "   Usage: curl -O \"http://127.0.0.1:8080/api/file/download?path=ies/actor.ies&version=0\""
    );
    println!(
        "   Description: Download the raw file. Optionally specify a version (default is 0).\n"
    );

    println!("6. GET /api/file/parse?path=<file>&version=<index>");
    println!(
        "   Usage: curl \"http://127.0.0.1:8080/api/file/parse?path=ies/actor.ies&version=0\""
    );
    println!(
        "   Description: Parse the file as an IES (lighting profile). Optionally specify a version (default is 0).\n"
    );

    println!("7. GET /api/file/preview?path=<file>&version=<index>");
    println!(
        "   Usage: curl \"http://127.0.0.1:8080/api/file/preview?path=ies/actor.ies&version=0\""
    );
    println!(
        "   Description: Preview the file according to its type:\n\
     - .ies → parsed JSON IES lighting profile\n\
     - .xml → raw XML text\n\
     - .lua → raw Lua text\n\
     - .png/.jpg/.jpeg/.bmp/.tga → image bytes\n\
     - others → raw binary data\n\
     Optionally specify a version (default is 0).\n"
    );

    HttpServer::new(move || {
        App::new()
            .app_data(folder_tree_data.clone())
            .app_data(game_root_data.clone())
            .app_data(duplicates_data.clone())
            .app_data(tera_data.clone()) // register Tera
            .configure(api::init_routes)
            .service(web_data::index) // our template route
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
