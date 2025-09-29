use std::sync::Arc;

use actix_web::{HttpResponse, Responder, get, web};
use tera::{Context, Tera};

use crate::{api::Duplicates, category::Folder, ipf::FileSizeStats};

#[get("/home")]
pub async fn home(
    tera: web::Data<Tera>,
    folder_tree: web::Data<Arc<Folder>>,
    duplicates: web::Data<Duplicates>,
    file_stats: web::Data<FileSizeStats>,
) -> impl Responder {
    let mut ctx = Context::new();
    ctx.insert("title", "Tree of Savior Archive Viewer");

    // File stats
    ctx.insert("count_unique", &file_stats.count_unique);
    ctx.insert("count_duplicated", &file_stats.count_duplicated);
    ctx.insert("compressed_lowest", &file_stats.compressed_lowest);
    ctx.insert("compressed_avg", &file_stats.compressed_avg);
    ctx.insert("compressed_highest", &file_stats.compressed_highest);
    ctx.insert("uncompressed_lowest", &file_stats.uncompressed_lowest);
    ctx.insert("uncompressed_avg", &file_stats.uncompressed_avg);
    ctx.insert("uncompressed_highest", &file_stats.uncompressed_highest);

    // Duplicate stats
    ctx.insert("duplicates_xac", &duplicates.xac.len());
    ctx.insert("duplicates_xsm", &duplicates.xsm.len());
    ctx.insert("duplicates_xsmtime", &duplicates.xsmtime.len());
    ctx.insert("duplicates_xpm", &duplicates.xpm.len());
    ctx.insert("duplicates_dds", &duplicates.dds.len());

    // Render template
    match tera.render("index.html", &ctx) {
        Ok(rendered) => HttpResponse::Ok().content_type("text/html").body(rendered),
        Err(e) => {
            println!("Tera render error: {:?}", e);
            HttpResponse::InternalServerError().body(format!("Failed to render template: {}", e))
        }
    }
}
