use pp_ss_r::bind_addr_from;

#[test]
fn defaults_to_localhost_8080() {
    assert_eq!(bind_addr_from(None), "127.0.0.1:8080");
}

#[test]
fn honors_override() {
    assert_eq!(
        bind_addr_from(Some("127.0.0.1:9099".to_string())),
        "127.0.0.1:9099"
    );
}
