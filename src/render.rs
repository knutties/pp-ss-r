use crate::model::PaymentPage;
use maud::{html, Markup, DOCTYPE};

pub fn format_amount(currency: &str, amount: &str) -> String {
    let symbol = match currency {
        "GBP" => "£",
        "USD" => "$",
        "EUR" => "€",
        _ => "",
    };
    format!("{symbol}{amount}")
}

pub fn lang_code(language: &str) -> &'static str {
    match language {
        "english" => "en",
        _ => "en",
    }
}

/// Shared document shell: doctype, head, and the Barclaycard header. `content`
/// is placed inside `<body>` after the header.
fn layout(title: &str, lang: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang=(lang) {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) }
                link rel="stylesheet" href="/assets/style.css";
            }
            body {
                header.pp-header {
                    img.pp-logo src="/assets/img/barclaycard-logo.png" alt="Barclaycard";
                }
                (content)
            }
        }
    }
}

/// Rendered when the order data can't be fetched from the data service.
pub fn render_error() -> Markup {
    let content = html! {
        main.pp-main {
            section.pp-summary {
                h1.pp-title { "We couldn't load this order" }
                p { "Something went wrong fetching the payment details. Please try again." }
            }
        }
    };
    layout("Something went wrong", "en", content)
}

pub fn render_confirmation(page: &PaymentPage) -> Markup {
    let amount_display = format_amount(&page.process.currency, &page.process.amount);
    let content = html! {
        main.pp-main {
            section.pp-summary {
                h1.pp-title { "Payment received (demo)" }
                p.pp-amount { (amount_display) }
                p.pp-order-id { "Order: " (page.process.order_id) }
                p { "This is a demo confirmation. No real payment was processed." }
            }
        }
    };
    layout("Payment received", lang_code(&page.init.language), content)
}

pub fn render_payment_page(page: &PaymentPage) -> Markup {
    let amount_display = format_amount(&page.process.currency, &page.process.amount);
    let content = html! {
        main.pp-main {
            section.pp-summary {
                        h1.pp-title { "Order summary" }
                        p.pp-merchant { (page.process.merchant_id) }
                        @if !page.process.description.is_empty() {
                            p.pp-desc { (page.process.description) }
                        }
                        p.pp-amount { (amount_display) }
                        p.pp-order-id { "Order: " (page.process.order_id) }
                    }
                    form.pp-form method="post" action="/pay" {
                        label for="card-name" { "Name on card" }
                        input #card-name name="card-name" type="text" autocomplete="cc-name" required;

                        label for="card-number" { "Card number" }
                        input #card-number name="card-number" type="text" inputmode="numeric"
                            autocomplete="cc-number" placeholder="1234 5678 9012 3456" required;

                        div.pp-row {
                            div.pp-col {
                                label for="card-expiry" { "Expiry" }
                                input #card-expiry name="card-expiry" type="text" inputmode="numeric"
                                    autocomplete="cc-exp" placeholder="MM/YY" required;
                            }
                            div.pp-col {
                                label for="card-cvc" { "CVC" }
                                input #card-cvc name="card-cvc" type="text" inputmode="numeric"
                                    autocomplete="cc-csc" placeholder="123" required;
                            }
                        }
                        button.pp-pay type="submit" { "Pay " (amount_display) }
                    }
                }
                footer.pp-footer {
                    img.pp-secured src="/assets/img/secured.png" alt="Secured";
                    span { "Payments are secure and encrypted" }
                }
    };
    layout("Barclaycard Payment", lang_code(&page.init.language), content)
}
