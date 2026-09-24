use actix_web::http::StatusCode;
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use std::borrow::Cow;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub mod model;
pub mod render;

use crate::model::{sample, PaymentPage};
use crate::render::{render_confirmation, render_error, render_payment_page};

const CSP: &str = "default-src 'self'; script-src 'none'; style-src 'self'; \
     img-src 'self'; font-src 'self'; form-action 'self'; \
     base-uri 'none'; frame-ancestors 'none'";

/// Where the render service fetches order data from. Points at the server's own
/// `/api` by default; set `DATA_BASE_URL` to a real data service to swap it out.
#[derive(Clone)]
pub struct DataSource {
    pub base_url: String,
}

impl DataSource {
    pub fn from_env() -> Self {
        Self {
            base_url: std::env::var("DATA_BASE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string()),
        }
    }
}

/// Optional `?delay_ms=` query param, used to simulate a slow upstream.
#[derive(Debug, Deserialize)]
struct DelayQuery {
    delay_ms: Option<u64>,
}

/// Address the server binds to. Defaults to `127.0.0.1:8080`; override via the
/// `BIND_ADDR` environment variable (used for e2e when 8080 is occupied).
pub fn bind_addr() -> String {
    bind_addr_from(std::env::var("BIND_ADDR").ok())
}

pub fn bind_addr_from(override_addr: Option<String>) -> String {
    override_addr.unwrap_or_else(|| "127.0.0.1:8080".to_string())
}

/// Routes only — no shared state. Lets tests inject their own reqwest client and
/// `DataSource` before mounting the routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/healthz", web::get().to(healthz));
    cfg.route("/", web::get().to(index));
    cfg.route("/order/{id}", web::get().to(order));
    cfg.route("/pay", web::post().to(pay));
    cfg.route("/api/orders/{id}", web::get().to(api_order));
    cfg.service(actix_files::Files::new("/assets", "static"));
}

/// Full app wiring: a shared reqwest client and the env-configured `DataSource`,
/// then the routes.
pub fn app_config(cfg: &mut web::ServiceConfig) {
    cfg.app_data(web::Data::new(reqwest::Client::new()));
    cfg.app_data(web::Data::new(DataSource::from_env()));
    configure(cfg);
}

async fn healthz() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body("ok")
}

/// Data endpoint: returns the order as JSON. `?delay_ms=` sleeps first to
/// simulate a slow upstream. Emits a JSON request log with a `delay_ms` /
/// `serialize_ms` / `total_ms` breakdown.
async fn api_order(
    req: HttpRequest,
    path: web::Path<String>,
    q: web::Query<DelayQuery>,
) -> HttpResponse {
    let start = Instant::now();
    let delay_ms = q.delay_ms.unwrap_or(0);
    if delay_ms > 0 {
        actix_web::rt::time::sleep(Duration::from_millis(delay_ms)).await;
    }

    let mut page = sample().clone();
    page.process.order_id = path.into_inner();

    let serialize_start = Instant::now();
    let body = serde_json::to_string(&page).unwrap_or_else(|_| "{}".to_string());
    let serialize_ms = serialize_start.elapsed().as_secs_f64() * 1000.0;

    let bytes = body.len();
    let status = 200u16;
    let total_ms = start.elapsed().as_secs_f64() * 1000.0;

    tracing::info!(
        ts_ms = now_ms(),
        method = %req.method(),
        path = %req.path(),
        status,
        delay_ms,
        serialize_ms,
        total_ms,
        bytes,
        "request"
    );

    HttpResponse::Ok()
        .content_type("application/json")
        .body(body)
}

async fn index(
    req: HttpRequest,
    client: web::Data<reqwest::Client>,
    data: web::Data<DataSource>,
    q: web::Query<DelayQuery>,
) -> HttpResponse {
    let id = sample().process.order_id.clone();
    fetch_and_render(&req, &client, &data, &id, q.delay_ms, |p| {
        render_payment_page(p).into_string()
    })
    .await
}

async fn order(
    req: HttpRequest,
    path: web::Path<String>,
    client: web::Data<reqwest::Client>,
    data: web::Data<DataSource>,
    q: web::Query<DelayQuery>,
) -> HttpResponse {
    let id = path.into_inner();
    fetch_and_render(&req, &client, &data, &id, q.delay_ms, |p| {
        render_payment_page(p).into_string()
    })
    .await
}

async fn pay(req: HttpRequest) -> HttpResponse {
    // The confirmation is a local demo — no upstream fetch.
    timed_local(&req, || Cow::Borrowed(sample()), |p| {
        render_confirmation(p).into_string()
    })
}

/// Fetches the order over HTTP from the data service, renders it, and logs one
/// JSON line with a `fetch_ms` / `render_ms` / `total_ms` breakdown. On fetch or
/// decode failure, logs `fetch_failed` and returns a 502 error page.
async fn fetch_and_render(
    req: &HttpRequest,
    client: &reqwest::Client,
    data: &DataSource,
    id: &str,
    delay_ms: Option<u64>,
    render: impl FnOnce(&PaymentPage) -> String,
) -> HttpResponse {
    let start = Instant::now();
    let fetched = fetch_order(client, &data.base_url, id, delay_ms).await;
    let fetch_ms = start.elapsed().as_secs_f64() * 1000.0;

    let page = match fetched {
        Ok(page) => page,
        Err(e) => {
            let total_ms = start.elapsed().as_secs_f64() * 1000.0;
            tracing::error!(
                method = %req.method(),
                path = %req.path(),
                fetch_ms,
                total_ms,
                error = %e,
                "fetch_failed"
            );
            return html(StatusCode::BAD_GATEWAY, render_error().into_string());
        }
    };

    let render_start = Instant::now();
    let body = render(&page);
    let render_ms = render_start.elapsed().as_secs_f64() * 1000.0;

    let bytes = body.len();
    let status = 200u16;
    let total_ms = start.elapsed().as_secs_f64() * 1000.0;

    tracing::info!(
        ts_ms = now_ms(),
        method = %req.method(),
        path = %req.path(),
        status,
        fetch_ms,
        render_ms,
        total_ms,
        bytes,
        "request"
    );

    html(StatusCode::OK, body)
}

async fn fetch_order(
    client: &reqwest::Client,
    base_url: &str,
    id: &str,
    delay_ms: Option<u64>,
) -> Result<PaymentPage, String> {
    let mut url = format!("{base_url}/api/orders/{id}");
    if let Some(ms) = delay_ms {
        url.push_str(&format!("?delay_ms={ms}"));
    }
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("request to {url} failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("data service returned {}", resp.status()));
    }
    resp.json::<PaymentPage>()
        .await
        .map_err(|e| format!("decoding order failed: {e}"))
}

/// Renders a local (non-fetched) page and logs one JSON line with a `load_ms` /
/// `render_ms` / `total_ms` breakdown.
fn timed_local(
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

    tracing::info!(
        ts_ms = now_ms(),
        method = %req.method(),
        path = %req.path(),
        status,
        load_ms,
        render_ms,
        total_ms,
        bytes,
        "request"
    );

    html(StatusCode::OK, body)
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Builds an HTML response with the standard security headers. A card-entry page
/// must never be cached, and all assets are same-origin, so a strict CSP is safe.
fn html(status: StatusCode, body: String) -> HttpResponse {
    HttpResponse::build(status)
        .content_type("text/html; charset=utf-8")
        .insert_header(("Cache-Control", "no-store"))
        .insert_header(("X-Content-Type-Options", "nosniff"))
        .insert_header(("Referrer-Policy", "no-referrer"))
        .insert_header(("Content-Security-Policy", CSP))
        .body(body)
}
