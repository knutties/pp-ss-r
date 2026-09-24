use actix_web::{test, App};
use pp_ss_r::app_config;

#[actix_web::test]
async fn serves_stylesheet() {
    let app = test::init_service(App::new().configure(app_config)).await;
    let req = test::TestRequest::get().uri("/assets/style.css").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = String::from_utf8(test::read_body(resp).await.to_vec()).unwrap();
    assert!(body.contains("pp-pay"));
}

#[actix_web::test]
async fn blocks_path_traversal() {
    let app = test::init_service(App::new().configure(app_config)).await;
    let req = test::TestRequest::get()
        .uri("/assets/%2e%2e/Cargo.toml")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(
        !resp.status().is_success(),
        "must not serve files outside static/"
    );
}
