use actix_web::{web, HttpResponse};

pub mod model;
pub mod render;

use crate::model::load_sample;
use crate::render::{render_confirmation, render_payment_page};

/// Address the server binds to. Defaults to `127.0.0.1:8080`; override via the
/// `BIND_ADDR` environment variable (used for e2e when 8080 is occupied).
pub fn bind_addr() -> String {
    bind_addr_from(std::env::var("BIND_ADDR").ok())
}

pub fn bind_addr_from(override_addr: Option<String>) -> String {
    override_addr.unwrap_or_else(|| "127.0.0.1:8080".to_string())
}

pub fn app_config(cfg: &mut web::ServiceConfig) {
    cfg.route("/healthz", web::get().to(healthz));
    cfg.route("/", web::get().to(index));
    cfg.route("/order/{id}", web::get().to(order));
    cfg.route("/pay", web::post().to(pay));
    cfg.service(actix_files::Files::new("/assets", "static"));
}

async fn healthz() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body("ok")
}

async fn index() -> HttpResponse {
    let page = load_sample();
    html_response(render_payment_page(&page).into_string())
}

async fn order(path: web::Path<String>) -> HttpResponse {
    let mut page = load_sample();
    page.process.order_id = path.into_inner();
    html_response(render_payment_page(&page).into_string())
}

async fn pay() -> HttpResponse {
    let page = load_sample();
    html_response(render_confirmation(&page).into_string())
}

fn html_response(body: String) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body)
}
