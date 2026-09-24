use actix_web::{App, HttpServer};
use pp_ss_r::app_config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("listening on http://127.0.0.1:8080");
    HttpServer::new(|| App::new().configure(app_config))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
