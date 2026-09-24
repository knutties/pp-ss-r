use pp_ss_r::model::{load_sample, PaymentPage};

#[test]
fn sample_loads_with_expected_fields() {
    let page: PaymentPage = load_sample();
    assert_eq!(page.process.currency, "GBP");
    assert_eq!(page.process.amount, "0.01");
    assert_eq!(page.init.tenant.tenant_id, "barclays");
    assert_eq!(page.init.merchant_id, "hamishtest");
}

#[test]
fn deserialize_rejects_missing_required_field() {
    let bad = r#"{"init":{},"process":{}}"#;
    let parsed: Result<PaymentPage, _> = serde_json::from_str(bad);
    assert!(parsed.is_err(), "missing fields must fail deserialization");
}
