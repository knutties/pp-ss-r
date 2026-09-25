use pp_ss_r::checkout::build_request;

#[test]
fn builds_checkout_request_matching_the_api_shape() {
    let req = build_request(
        "00.01",
        "GBP",
        "hamishtest",
        "https://merchant.example.com/checkout/return",
        "defaultclient",
        "ORDER-abc123",
    );
    let v = serde_json::to_value(&req).unwrap();

    assert_eq!(v["merchant_reference"], "ORDER-abc123");
    assert_eq!(v["mode"], "HOSTED_PAGE");
    assert_eq!(v["objective"]["type"], "PAYMENT");
    assert_eq!(v["objective"]["payment"]["merchant_id"], "hamishtest");
    assert_eq!(v["objective"]["payment"]["amount"]["currency"], "GBP");
    assert_eq!(v["objective"]["payment"]["amount"]["value"], "00.01");
    assert_eq!(v["objective"]["payment"]["processing_mode"], "AUTH_AND_CAPTURE");
    assert_eq!(v["objective"]["payment"]["payment_purpose"], "PAYMENT");
    assert_eq!(v["objective"]["payment"]["payment_channel"], "ECOMMERCE");
    assert_eq!(v["experience"]["branding_profile"], "defaultclient");
    assert_eq!(
        v["return_urls"]["default"],
        "https://merchant.example.com/checkout/return"
    );
}
