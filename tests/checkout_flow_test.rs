use actix_web::{test, web, App, HttpResponse};
use pp_ss_r::{app_config, configure, CheckoutConfig};

async fn mock_session() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({ "id": "sess_test_123" }))
}

#[actix_web::test]
async fn checkout_form_renders() {
    let app = test::init_service(App::new().configure(app_config)).await;
    let req = test::TestRequest::get().uri("/checkout").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = String::from_utf8(test::read_body(resp).await.to_vec()).unwrap();
    assert!(body.contains("action=\"/checkout\""), "form posts to /checkout");
    assert!(body.contains("name=\"amount\""), "amount input present");
    assert!(body.contains("name=\"currency\""), "currency input present");
}

#[actix_web::test]
async fn checkout_creates_session_and_renders_page() {
    // Mock Juspay checkout-sessions API.
    let api = actix_test::start(|| {
        App::new().route("/v1/checkout-sessions", web::post().to(mock_session))
    });
    let base = api.url("");
    let base = base.trim_end_matches('/').to_string();

    // Render service pointed at the mock.
    let app = actix_test::start(move || {
        App::new()
            .app_data(web::Data::new(reqwest::Client::new()))
            .app_data(web::Data::new(CheckoutConfig {
                base_url: base.clone(),
                api_key: "test-key".to_string(),
            }))
            .configure(configure)
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(app.url("/checkout"))
        .form(&[("amount", "5.00"), ("currency", "GBP")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let body = resp.text().await.unwrap();
    assert!(body.contains("£5.00"), "renders the entered amount");
    assert!(body.contains("sess_test_123"), "renders the created session id");
}

#[actix_web::test]
async fn checkout_failure_returns_502() {
    let app = actix_test::start(|| {
        App::new()
            .app_data(web::Data::new(reqwest::Client::new()))
            .app_data(web::Data::new(CheckoutConfig {
                base_url: "http://127.0.0.1:1".to_string(),
                api_key: "x".to_string(),
            }))
            .configure(configure)
    });
    let client = reqwest::Client::new();
    let resp = client
        .post(app.url("/checkout"))
        .form(&[("amount", "1.00"), ("currency", "GBP")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 502, "upstream failure surfaces as 502");
}
