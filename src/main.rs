use actix_web::{App, HttpServer};
use pp_ss_r::model::load_sample_checked;
use pp_ss_r::{app_config, bind_addr};
use tracing_subscriber::EnvFilter;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Structured JSON logs to stdout. One flat line per event; timestamps come
    // from our own `ts_ms` field, so the subscriber's timer is disabled.
    tracing_subscriber::fmt()
        .json()
        .flatten_event(true)
        .without_time()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            // Default: our request logs + server lifecycle, minus per-worker noise.
            // Override with RUST_LOG (e.g. RUST_LOG=debug).
            EnvFilter::new("info,actix_server::worker=warn")
        }))
        .init();

    // Validate the payload at startup so a malformed/blank order.json fails
    // loudly here rather than 500-ing every page request while /healthz passes.
    if let Err(e) = load_sample_checked() {
        eprintln!("invalid payment payload: {e}");
        std::process::exit(1);
    }
    let addr = bind_addr();
    println!("listening on http://{addr}");
    HttpServer::new(|| App::new().configure(app_config))
        .bind(&addr)?
        .run()
        .await
}
