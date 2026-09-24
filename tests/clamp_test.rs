use pp_ss_r::{clamp_delay, MAX_DELAY_MS};

#[test]
fn clamps_values_above_the_max() {
    assert_eq!(clamp_delay(u64::MAX), MAX_DELAY_MS);
    assert_eq!(clamp_delay(MAX_DELAY_MS + 1), MAX_DELAY_MS);
}

#[test]
fn passes_through_values_at_or_below_the_max() {
    assert_eq!(clamp_delay(0), 0);
    assert_eq!(clamp_delay(250), 250);
    assert_eq!(clamp_delay(MAX_DELAY_MS), MAX_DELAY_MS);
}
