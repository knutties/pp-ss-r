use actix_web::{web, HttpRequest, HttpResponse};
use std::borrow::Cow;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub mod model;
pub mod render;

use crate::model::{sample, PaymentPage};
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
    timed_html(&req, || Cow::Borrowed(sample()), |p| {
        render_payment_page(p).into_string()
    })
}

async fn order(req: HttpRequest, path: web::Path<String>) -> HttpResponse {
    let id = path.into_inner();
    timed_html(
        &req,
        move || {
            let mut page = sample().clone();
            page.process.order_id = id;
            Cow::Owned(page)
        },
        |p| render_payment_page(p).into_string(),
    )
}

async fn pay(req: HttpRequest) -> HttpResponse {
    timed_html(&req, || Cow::Borrowed(sample()), |p| {
        render_confirmation(p).into_string()
    })
}

/// Loads the payload, renders it, and emits one structured JSON log line with a
/// per-phase timing breakdown: `load_ms` (payload deserialization), `render_ms`
/// (HTML generation), and `total_ms` (whole handler).
fn timed_html(
    req: &HttpRequest,
    prepare: impl FnOnce() -> Cow<'static, PaymentPage>,
    render: impl FnOnce(&PaymentPage) -> String,
) -> HttpResponse {
    let start = Instant::now();
    let page = prepare();
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
        // A card-entry page must never be cached, and should not leak its URL or
        // be sniffed. All assets are same-origin, so a strict CSP is safe.
        .insert_header(("Cache-Control", "no-store"))
        .insert_header(("X-Content-Type-Options", "nosniff"))
        .insert_header(("Referrer-Policy", "no-referrer"))
        .insert_header((
            "Content-Security-Policy",
            "default-src 'self'; script-src 'none'; style-src 'self'; \
             img-src 'self'; font-src 'self'; form-action 'self'; \
             base-uri 'none'; frame-ancestors 'none'",
        ))
        .body(body)
}
