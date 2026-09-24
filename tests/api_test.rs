use actix_web::{test, App};
use pp_ss_r::app_config;
use std::time::Instant;

#[actix_web::test]
async fn api_returns_order_json() {
    let app = test::init_service(App::new().configure(app_config)).await;
    let req = test::TestRequest::get()
        .uri("/api/orders/ord_demo")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let ct = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(ct.starts_with("application/json"), "content-type was {ct}");

    let v: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(v["process"]["order_id"].as_str(), Some("ord_demo"));
    assert_eq!(v["process"]["currency"].as_str(), Some("GBP"));
}

#[actix_web::test]
async fn api_delay_ms_is_respected() {
    let app = test::init_service(App::new().configure(app_config)).await;
    let start = Instant::now();
    let req = test::TestRequest::get()
        .uri("/api/orders/x?delay_ms=80")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let elapsed = start.elapsed().as_millis();
    assert!(
        elapsed >= 70,
        "delay_ms=80 should slow the response, observed {elapsed}ms"
    );
}
