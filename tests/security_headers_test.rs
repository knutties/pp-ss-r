use actix_web::{test, App};
use pp_ss_r::app_config;

#[actix_web::test]
async fn payment_page_sets_security_headers() {
    let app = test::init_service(App::new().configure(app_config)).await;
    // /pay renders locally; the security headers come from the shared html()
    // builder used by every rendered response (including the fetched pages).
    let req = test::TestRequest::post().uri("/pay").to_request();
    let resp = test::call_service(&app, req).await;
    let h = resp.headers();

    assert_eq!(
        h.get("cache-control").map(|v| v.to_str().unwrap()),
        Some("no-store"),
        "card-form page must not be cached"
    );
    assert_eq!(
        h.get("x-content-type-options").map(|v| v.to_str().unwrap()),
        Some("nosniff")
    );
    assert_eq!(
        h.get("referrer-policy").map(|v| v.to_str().unwrap()),
        Some("no-referrer")
    );
    assert!(
        h.get("content-security-policy").is_some(),
        "a CSP must be present"
    );
}

#[actix_web::test]
async fn confirmation_page_sets_no_store() {
    let app = test::init_service(App::new().configure(app_config)).await;
    let req = test::TestRequest::post().uri("/pay").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.headers()
            .get("cache-control")
            .map(|v| v.to_str().unwrap()),
        Some("no-store")
    );
}
