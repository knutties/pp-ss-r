use actix_web::{test, App};
use pp_ss_r::app_config;

#[actix_web::test]
async fn healthz_returns_ok() {
    let app = test::init_service(App::new().configure(app_config)).await;
    let req = test::TestRequest::get().uri("/healthz").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    assert_eq!(body, "ok");
}
