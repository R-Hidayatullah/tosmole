use std::sync::Arc;

use actix_web::{HttpResponse, Responder, get, web};
use tera::{Context, Tera};

use crate::{api::Duplicates, category::Folder};

#[get("/")]
pub async fn index(
    tera: web::Data<Tera>,
    folder_tree: web::Data<Arc<Folder>>,
    duplicates: web::Data<Duplicates>,
) -> impl Responder {
    let mut ctx = Context::new();
    ctx.insert("title", "Tree of Savior Archive Viewer");

    // count files recursively
    let total_files = folder_tree.files.len()
        + folder_tree
            .subfolders
            .values()
            .map(|f| f.files.len())
            .sum::<usize>();
    ctx.insert("total_files", &total_files);
    ctx.insert("duplicates_xac", &duplicates.xac.len()); // for example
    ctx.insert("duplicates_xsm", &duplicates.xsm.len());
    ctx.insert("duplicates_xsmtime", &duplicates.xsmtime.len());
    ctx.insert("duplicates_xpm", &duplicates.xpm.len());
    ctx.insert("duplicates_dds", &duplicates.dds.len());

    let rendered = tera
        .render("index.html", &ctx)
        .unwrap_or_else(|e| e.to_string());
    HttpResponse::Ok().content_type("text/html").body(rendered)
}
