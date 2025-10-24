#![allow(unused)]

use actix_web::{App, HttpServer, web};
use core::option::Option::None;
use serde::Deserialize;
use serde_json::from_reader;
use std::{
    collections::{BTreeMap, HashMap},
    fs::File,
    io::{self, BufReader},
    path::PathBuf,
    sync::Arc,
};
use tera::Tera;

use category::Folder;

use crate::ies::IESRoot;

mod api;
mod category;
mod fsb;
mod gltf;
mod ies;
mod ipf;
mod mesh;
mod stb;
mod tok;
mod tsv;
mod web_data;
mod xac;
mod xml;
mod xpm;
mod xsm;

#[derive(Debug, Deserialize)]
struct PathsConfig {
    game_root: String,
    address: Option<String>, // e.g. "127.0.0.1"
    port: Option<u16>,       // e.g. 8080
}

fn load_game_root_from_json(file_path: &str) -> Result<PathsConfig, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let config: PathsConfig = serde_json::from_reader(reader)?;
    Ok(config)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    use std::time::Instant;

    // ---------------------------
    // Load game_root dynamically (or hardcode if you prefer)
    // ---------------------------
    let config = load_game_root_from_json("paths.json").expect("Failed to load paths.json");

    let game_root = PathBuf::from(&config.game_root);
    let addr = config.address.unwrap_or_else(|| "127.0.0.1".to_string());
    let port = config.port.unwrap_or(8080);

    // ---------------------------
    // Derive lang_folder from game_root
    // ---------------------------
    let lang_folder = game_root.join("release/languageData/English");

    println!("Game root: {:?}", game_root);
    println!("Language folder: {:?}", lang_folder);

    // ---------------------------
    // Parse IPF Archives
    // ---------------------------
    let ipf_start = Instant::now();
    println!("Parsing IPF archives...");
    let mut parsed_ipfs = ipf::parse_game_ipfs(&game_root)?;
    println!("Parsed {} IPF archives", parsed_ipfs.len());
    let mut file_stat_data = ipf::compute_ipf_file_stats(&parsed_ipfs);

    let mut all_files = ipf::collect_file_tables_from_parsed(&mut parsed_ipfs);
    ipf::sort_file_tables_by_folder_then_name(&mut all_files);

    let grouped: BTreeMap<String, Vec<ipf::IPFFileTable>> =
        ipf::group_file_tables_by_directory(all_files);
    file_stat_data.count_unique = grouped.len() as u32;

    let folder_tree = Arc::new(category::build_tree(grouped));
    println!("IPF parsing completed in {:.2?}", ipf_start.elapsed());

    let file_find_start = Instant::now();
    println!("Find and get files...");

    let xac_ies = folder_tree.search_file_by_full_path("ies_client/xac.ies");
    let ability_ies = folder_tree.search_file_by_full_path("ies_ability/ability.ies");
    let ability_name_ies = folder_tree.search_file_by_full_path("ies_ability/ability_ .ies");
    let job_ies = folder_tree.search_file_by_full_path("ies/job.ies");
    let statbase_pc_ies = folder_tree.search_file_by_full_path("ies/statbase_pc.ies");
    let skilltree_ies = folder_tree.search_file_by_full_path("ies/skilltree.ies");
    let skill_ies = folder_tree.search_file_by_full_path("ies/skill.ies");
    let cooldown_ies = folder_tree.search_file_by_full_path("ies/cooldown.ies");
    let skill_simony_ies = folder_tree.search_file_by_full_path("ies/skill_simony.ies");
    let stance_ies = folder_tree.search_file_by_full_path("ies/stance.ies");
    let item_gem_ies = folder_tree.search_file_by_full_path("ies/item_gem.ies");
    let dialogtext_ies = folder_tree.search_file_by_full_path("ies_client/dialogtext.ies");
    let item_ies = folder_tree.search_file_by_full_path("ies/item.ies");
    let cardbattle_ies = folder_tree.search_file_by_full_path("ies/cardbattle.ies");
    let collection_ies = folder_tree.search_file_by_full_path("ies/collection.ies");
    let reward_indun_ies = folder_tree.search_file_by_full_path("ies/reward_indun.ies");
    let setitem_ies = folder_tree.search_file_by_full_path("ies/setitem.ies");
    let item_equip_ies = folder_tree.search_file_by_full_path("ies/item_equip.ies");
    let item_colorspray_ies = folder_tree.search_file_by_full_path("ies/item_colorspray.ies");
    let item_equip_name_ies = folder_tree.search_file_by_full_path("ies/item_equip_ .ies");
    let item_premium_ies = folder_tree.search_file_by_full_path("ies/item_premium.ies");
    let item_quest_ies = folder_tree.search_file_by_full_path("ies/item_quest.ies");
    let recipe_ies = folder_tree.search_file_by_full_path("ies/recipe.ies");
    let map_ies = folder_tree.search_file_by_full_path("ies/map.ies");
    let zonedropitemlist_name_ies =
        folder_tree.search_file_by_full_path("ies_drop/zonedropitemlist_ .ies");
    let zonedropitemlist_f_name =
        folder_tree.search_file_by_full_path("ies_drop/zonedropitemlist_f_ .ies");
    let anchor_name_ies = folder_tree.search_file_by_full_path("ies_drop/anchor_ .ies");
    let gentype_name_ies = folder_tree.search_file_by_full_path("ies_mongem/gentype_ .ies");
    let map_data_name_tok = folder_tree.search_file_by_full_path("bg/ .tok");
    let statbase_monster_ies = folder_tree.search_file_by_full_path("ies/statbase_monster.ies");
    let statbase_monster_type_ies =
        folder_tree.search_file_by_full_path("ies/statbase_monster_type.ies");
    let monster_ies = folder_tree.search_file_by_full_path("ies/monster.ies");
    let monster_event_ies = folder_tree.search_file_by_full_path("ies/monster_event.ies");
    let monster_npc_ies = folder_tree.search_file_by_full_path("ies/monster_npc.ies");
    let monster_solo_dungeon_ies =
        folder_tree.search_file_by_full_path("ies/monster_solo_dungeon.ies");
    let baseskinset_xml = folder_tree.search_file_by_full_path("ui/baseskinset/baseskinset.xml");
    let classicon_xml = folder_tree.search_file_by_full_path("ui/baseskinset/classicon.xml");
    let itemicon_xml = folder_tree.search_file_by_full_path("ui/baseskinset/itemicon.xml");
    let mongem_xml = folder_tree.search_file_by_full_path("ui/baseskinset/mongem.xml");
    let monillust_xml = folder_tree.search_file_by_full_path("ui/baseskinset/monillust.xml");
    let skillicon_xml = folder_tree.search_file_by_full_path("ui/baseskinset/skillicon.xml");
    let wholedicid_xml = folder_tree.search_file_by_full_path("language/wholedicid.xml");

    println!(
        "File find and get completed in {:.2?}",
        file_find_start.elapsed()
    );

    let mut mesh_map: HashMap<String, String> = HashMap::new();

    if let Some((full_path, file_table)) = xac_ies.last() {
        println!("IPF Path : {:?}", file_table.file_path);
        match file_table.extract_data() {
            Ok(raw_data) => match IESRoot::from_bytes(&raw_data) {
                Ok(ies_data) => {
                    mesh_map = ies_data.extract_mesh_path_map();
                    println!(
                        "Successfully parsed '{}'! Mesh map contains {} entries.",
                        full_path,
                        mesh_map.len()
                    );
                }
                Err(e) => {
                    eprintln!("Failed to parse IESRoot from '{}': {}", full_path, e);
                }
            },
            Err(e) => {
                eprintln!("Failed to extract data from '{}': {}", full_path, e);
            }
        }
    } else {
        println!("File 'ies_client/xac.ies' not found!");
    }

    // ---------------------------
    // Parse Language Data
    // ---------------------------
    let lang_start = Instant::now();
    println!("Parsing language data...");
    let (_etc_data, _item_data) = tsv::parse_language_data(&lang_folder)?;
    println!("Language parsing completed in {:.2?}", lang_start.elapsed());

    // ---------------------------
    // Parse Duplicates
    // ---------------------------
    let dup_start = Instant::now();
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

    println!(
        "Duplicates parsing completed in {:.2?}",
        dup_start.elapsed()
    );

    let duplicates_data = web::Data::new(api::Duplicates {
        xac: xac_duplicates,
        xsm: xsm_duplicates,
        xsmtime: xsmtime_duplicates,
        xpm: xpm_duplicates,
        dds: dds_duplicates,
    });

    // ---------------------------
    // Prepare Actix Web Server
    // ---------------------------
    let folder_tree_data = web::Data::new(folder_tree);
    let game_root_data = web::Data::new(game_root);
    let file_stats = web::Data::new(file_stat_data);
    let tera = Tera::new("templates/**/*").expect("Failed to initialize Tera templates");
    let tera_data = web::Data::new(tera);
    let mesh_map_data = web::Data::new(mesh_map);

    println!("Starting server at http://{}:{} ...\n", addr, port);

    HttpServer::new(move || {
        App::new()
            .app_data(folder_tree_data.clone())
            .app_data(game_root_data.clone())
            .app_data(duplicates_data.clone())
            .app_data(tera_data.clone())
            .app_data(file_stats.clone())
            .app_data(mesh_map_data.clone())
            .configure(api::init_routes)
            .service(web_data::index)
            .service(web_data::home)
    })
    .bind((addr, port))?
    .run()
    .await
}
