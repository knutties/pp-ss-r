use actix_web::{web, HttpResponse};

pub mod model;

pub fn app_config(cfg: &mut web::ServiceConfig) {
    cfg.route("/healthz", web::get().to(healthz));
}

async fn healthz() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body("ok")
}
