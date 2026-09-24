use actix_web::{web, HttpResponse};

pub mod model;
pub mod render;

use crate::model::load_sample;
use crate::render::{render_confirmation, render_payment_page};

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
