use actix_web::{test, App};
use pp_ss_r::app_config;

#[actix_web::test]
async fn pay_returns_confirmation() {
    let app = test::init_service(App::new().configure(app_config)).await;
    let req = test::TestRequest::post().uri("/pay").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = String::from_utf8(test::read_body(resp).await.to_vec()).unwrap();
    assert!(body.to_lowercase().contains("received"), "confirmation text");
    assert!(body.contains("£0.01"), "echoes amount");
}
