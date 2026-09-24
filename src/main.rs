use actix_web::{App, HttpServer};
use pp_ss_r::model::load_sample_checked;
use pp_ss_r::{app_config, bind_addr};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
