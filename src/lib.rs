use actix_web::{web, HttpRequest, HttpResponse};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub mod model;
pub mod render;

use crate::model::{load_sample, PaymentPage};
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

async fn index(req: HttpRequest) -> HttpResponse {
    timed_html(&req, load_sample, |p| render_payment_page(p).into_string())
}

async fn order(req: HttpRequest, path: web::Path<String>) -> HttpResponse {
    let id = path.into_inner();
    timed_html(
        &req,
        move || {
            let mut page = load_sample();
            page.process.order_id = id;
            page
        },
        |p| render_payment_page(p).into_string(),
    )
}

async fn pay(req: HttpRequest) -> HttpResponse {
    timed_html(&req, load_sample, |p| render_confirmation(p).into_string())
}

/// Loads the payload, renders it, and emits one structured JSON log line with a
/// per-phase timing breakdown: `load_ms` (payload deserialization), `render_ms`
/// (HTML generation), and `total_ms` (whole handler).
fn timed_html(
    req: &HttpRequest,
    load: impl FnOnce() -> PaymentPage,
    render: impl FnOnce(&PaymentPage) -> String,
) -> HttpResponse {
    let start = Instant::now();
    let page = load();
    let load_ms = start.elapsed().as_secs_f64() * 1000.0;

    let render_start = Instant::now();
    let body = render(&page);
    let render_ms = render_start.elapsed().as_secs_f64() * 1000.0;

    let bytes = body.len();
    let status = 200u16;
    let total_ms = start.elapsed().as_secs_f64() * 1000.0;
    let ts_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    tracing::info!(
        ts_ms,
        method = %req.method(),
        path = %req.path(),
        status,
        load_ms,
        render_ms,
        total_ms,
        bytes,
        "request"
    );

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body)
}
