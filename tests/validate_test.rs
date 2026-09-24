use pp_ss_r::model::load_sample_checked;

#[test]
fn valid_sample_passes_validation() {
    assert!(
        load_sample_checked().is_ok(),
        "the bundled sample payload must validate"
    );
}

#[test]
fn blank_required_field_is_rejected() {
    let mut page = load_sample_checked().expect("sample must load");
    page.process.amount = String::new();
    assert!(
        page.validate().is_err(),
        "a blank required field must be rejected"
    );
}
