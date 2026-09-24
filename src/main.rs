use actix_web::{App, HttpServer};
use pp_ss_r::{app_config, bind_addr};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let addr = bind_addr();
    println!("listening on http://{addr}");
    HttpServer::new(|| App::new().configure(app_config))
        .bind(&addr)?
        .run()
        .await
}
