use actix_web::{test, App};
use pp_ss_r::app_config;

#[actix_web::test]
async fn root_renders_page() {
    let app = test::init_service(App::new().configure(app_config)).await;
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let ct = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(ct.starts_with("text/html"));
    let body = String::from_utf8(test::read_body(resp).await.to_vec()).unwrap();
    assert!(body.contains("£0.01"));
}

#[actix_web::test]
async fn order_id_is_echoed_and_escaped() {
    let app = test::init_service(App::new().configure(app_config)).await;
    let req = test::TestRequest::get()
        .uri("/order/ord_%3Cb%3Ex")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body = String::from_utf8(test::read_body(resp).await.to_vec()).unwrap();
    // decoded id is ord_<b>x ; maud must escape the angle brackets
    assert!(body.contains("ord_&lt;b&gt;x"), "id echoed and HTML-escaped");
    assert!(!body.contains("ord_<b>x"), "must not emit raw tags");
}
