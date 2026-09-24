use actix_web::{web, App};
use pp_ss_r::{configure, DataSource};

fn app_data_for(base_url: &str) -> (web::Data<reqwest::Client>, web::Data<DataSource>) {
    (
        web::Data::new(reqwest::Client::new()),
        web::Data::new(DataSource {
            base_url: base_url.to_string(),
        }),
    )
}

#[actix_web::test]
async fn page_fetches_data_and_renders() {
    // Data service (serves /api/orders/{id}).
    let data = actix_test::start(|| {
        let (c, d) = app_data_for("http://unused");
        App::new().app_data(c).app_data(d).configure(configure)
    });
    let base = data.url("");
    let base = base.trim_end_matches('/').to_string();

    // Render service, pointed at the data service.
    let page = actix_test::start(move || {
        let (c, d) = app_data_for(&base);
        App::new().app_data(c).app_data(d).configure(configure)
    });

    let body = reqwest::get(page.url("/"))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(body.contains("£0.01"), "fetched page must render the amount");

    let order = reqwest::get(page.url("/order/demo_order_42"))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert!(
        order.contains("demo_order_42"),
        "order id from fetched data must render"
    );
}

#[actix_web::test]
async fn fetch_failure_returns_502() {
    // Point the render service at a dead port so the fetch fails.
    let page = actix_test::start(|| {
        let (c, d) = app_data_for("http://127.0.0.1:1");
        App::new().app_data(c).app_data(d).configure(configure)
    });
    let resp = reqwest::get(page.url("/")).await.unwrap();
    assert_eq!(resp.status().as_u16(), 502, "fetch failure surfaces as 502");
}
