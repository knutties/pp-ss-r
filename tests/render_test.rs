use pp_ss_r::model::load_sample;
use pp_ss_r::render::{format_amount, render_payment_page};

#[test]
fn format_amount_known_and_unknown_currency() {
    assert_eq!(format_amount("GBP", "0.01"), "£0.01");
    assert_eq!(format_amount("USD", "5.00"), "$5.00");
    // unknown currency: keep the number, no symbol, no panic
    assert_eq!(format_amount("JPY", "500"), "500");
}

#[test]
fn page_contains_amount_and_fields() {
    let page = load_sample();
    let html = render_payment_page(&page).into_string();
    assert!(html.contains("£0.01"), "amount must render");
    assert!(html.contains("Test payment"), "description must render");
    assert!(html.contains("id=\"card-number\""), "card number field");
    assert!(html.contains("id=\"card-expiry\""), "expiry field");
    assert!(html.contains("id=\"card-cvc\""), "cvc field");
    assert!(html.contains("action=\"/pay\""), "form posts to /pay");
    assert!(html.contains("/assets/img/barclaycard-logo.png"), "logo");
    assert!(html.contains("<!DOCTYPE html>"), "full document");
}

#[test]
fn empty_description_is_omitted() {
    let mut page = load_sample();
    page.process.description = String::new();
    let html = render_payment_page(&page).into_string();
    assert!(!html.contains("class=\"pp-desc\""), "no empty description element");
}
