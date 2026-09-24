use pp_ss_r::model::sample;

#[test]
fn sample_is_parsed_once_and_cached() {
    let a = sample() as *const _;
    let b = sample() as *const _;
    assert_eq!(a, b, "sample() must return the same cached instance");
}

#[test]
fn cached_sample_has_expected_fields() {
    assert_eq!(sample().process.currency, "GBP");
    assert_eq!(sample().init.tenant.tenant_id, "barclays");
}
