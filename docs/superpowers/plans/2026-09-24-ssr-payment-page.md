# SSR Payment Page Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a fast, server-side-rendered payment page in Rust (actix-web) that renders from a JSON payload, serves all assets locally, and runs in a reproducible Nix dev shell.

**Architecture:** actix-web serves HTML rendered at compile time by maud from serde structs (`PaymentPage`) modeled on the real Barclaycard/Juspay `ssr_payload`. A sample payload lives in `data/order.json`. Static assets (Barclaycard logo, BarclaysEffra fonts, CSS) are mirrored locally under `static/` and served via actix-files. No runtime template parsing, no database, no client JS.

**Tech Stack:** Rust, actix-web 4, actix-files 0.6, maud 0.26, serde 1 / serde_json 1, Nix flake dev shell, headless Chromium (Nix) for e2e.

**Spec:** `docs/superpowers/specs/2026-09-24-ssr-payment-page-design.md`

## Global Constraints

- Rust edition 2021; `actix-web = "4"`, `actix-files = "0.6"`, `maud = "0.26"`, `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`.
- Server binds `127.0.0.1:8080`.
- All HTML responses are `Content-Type: text/html; charset=utf-8`.
- No external network requests at runtime — every asset served from `static/`.
- `/pay` is a demo stub: no validation, no payment processing, no storage.
- Dev environment via `flake.nix` devShell (`nix develop`); it provides the Rust toolchain and `chromium`.
- Currency symbol map is limited to GBP→£, USD→$, EUR→€, fallback empty string.

## Review Focus

- **Unknown currency code** (e.g. `"JPY"`): `format_amount` must still return the numeric amount (no symbol), never panic or drop the number. → tested in Task 3.
- **Empty `description`**: the summary must omit the description line cleanly, not render an empty element or the literal "". → tested in Task 3.
- **`GET /order/{id}` id echoing**: an arbitrary id string must appear as the order id in the rendered page and be HTML-escaped (maud escapes by default) — no injection via the path. → tested in Task 4.
- **Missing/blank sample JSON fields**: deserializing `data/order.json` with a missing required field must fail loudly at startup, not serve a half-rendered page. → tested in Task 2.
- **Static path traversal** (`/assets/../Cargo.toml`): actix-files must not serve files outside `static/`. → tested in Task 5.

---

### Task 1: Nix flake + actix scaffold with health check

**Files:**
- Create: `flake.nix`
- Create: `.envrc`
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Test: `tests/health_test.rs`

**Interfaces:**
- Produces: a runnable actix-web server on `127.0.0.1:8080` with `GET /healthz` → `200 "ok"`. Later tasks add routes to the same `App` factory. The app factory is defined as `fn app_config(cfg: &mut web::ServiceConfig)` so integration tests can mount the same routes.

- [ ] **Step 1: Write `flake.nix`**

```nix
{
  description = "SSR payment page (Rust + actix)";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  outputs = { self, nixpkgs }:
    let
      system = "aarch64-darwin";
      pkgs = import nixpkgs { inherit system; };
    in {
      devShells.${system}.default = pkgs.mkShell {
        packages = [ pkgs.cargo pkgs.rustc pkgs.rustfmt pkgs.clippy pkgs.chromium ];
        CHROME_BIN = "${pkgs.chromium}/bin/chromium";
      };
    };
}
```

Note: if the machine is not `aarch64-darwin`, change `system` accordingly (e.g. `x86_64-linux`).

- [ ] **Step 2: Write `.envrc`**

```bash
use flake
```

- [ ] **Step 3: Write `Cargo.toml`**

```toml
[package]
name = "pp-ss-r"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4"
actix-files = "0.6"
maud = "0.26"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[dev-dependencies]
actix-rt = "2"
```

- [ ] **Step 4: Write the failing test `tests/health_test.rs`**

```rust
use actix_web::{test, App};
use pp_ss_r::app_config;

#[actix_web::test]
async fn healthz_returns_ok() {
    let app = test::init_service(App::new().configure(app_config)).await;
    let req = test::TestRequest::get().uri("/healthz").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    assert_eq!(body, "ok");
}
```

- [ ] **Step 5: Run the test to verify it fails**

Run: `cargo test --test health_test`
Expected: FAIL — `pp_ss_r` crate / `app_config` not found (no lib target yet).

- [ ] **Step 6: Create the lib + bin in `src/main.rs`**

Restructure to expose a library. Create `src/lib.rs`:

```rust
use actix_web::{web, HttpResponse};

pub fn app_config(cfg: &mut web::ServiceConfig) {
    cfg.route("/healthz", web::get().to(healthz));
}

async fn healthz() -> HttpResponse {
    HttpResponse::Ok().content_type("text/plain; charset=utf-8").body("ok")
}
```

And `src/main.rs`:

```rust
use actix_web::{App, HttpServer};
use pp_ss_r::app_config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("listening on http://127.0.0.1:8080");
    HttpServer::new(|| App::new().configure(app_config))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
```

Add to `Cargo.toml`:

```toml
[lib]
name = "pp_ss_r"
path = "src/lib.rs"

[[bin]]
name = "pp-ss-r"
path = "src/main.rs"
```

- [ ] **Step 7: Run the test to verify it passes**

Run: `cargo test --test health_test`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add flake.nix .envrc Cargo.toml Cargo.lock src/lib.rs src/main.rs tests/health_test.rs
git commit -m "feat: nix flake + actix scaffold with health check"
```

---

### Task 2: Data model + sample payload

**Files:**
- Create: `src/model.rs`
- Create: `data/order.json`
- Modify: `src/lib.rs` (add `pub mod model;`)
- Test: `tests/model_test.rs`

**Interfaces:**
- Consumes: nothing from prior tasks.
- Produces:
  - `pub struct PaymentPage { pub init: InitPayload, pub process: ProcessPayload }`
  - `pub struct InitPayload { pub client_auth_token: String, pub client_id: String, pub environment: String, pub merchant_id: String, pub return_url: String, pub language: String, pub is_payment_link: bool, pub tenant: TenantInfo }`
  - `pub struct TenantInfo { pub assets_domain: String, pub tenant_id: String }`
  - `pub struct ProcessPayload { pub amount: String, pub currency: String, pub order_id: String, pub description: String, pub order_type: String, pub payment_links_expiry: String, pub source_object: String, pub merchant_id: String, pub metadata: String }`
  - `pub fn load_sample() -> PaymentPage` — deserializes the embedded `data/order.json` via `include_str!`, panics with a clear message on failure.

- [ ] **Step 1: Write `data/order.json`**

```json
{
  "init": {
    "client_auth_token": "tkn_c01ec4e522b74886a4a5eb713daa4b16",
    "client_id": "defaultclient",
    "environment": "production",
    "merchant_id": "hamishtest",
    "return_url": "https://merchant.example.com/checkout/return",
    "language": "english",
    "is_payment_link": true,
    "tenant": {
      "assets_domain": "https://secure.barclaycardpayments.com",
      "tenant_id": "barclays"
    }
  },
  "process": {
    "amount": "0.01",
    "currency": "GBP",
    "order_id": "pay_7ee8d7f469d343ecbeef25ae6a04142c",
    "description": "Test payment",
    "order_type": "ORDER_PAYMENT",
    "payment_links_expiry": "2026-09-24T17:14:02Z",
    "source_object": "ADAPTER_BARCLAYS",
    "merchant_id": "hamishtest",
    "metadata": "{}"
  }
}
```

- [ ] **Step 2: Write the failing test `tests/model_test.rs`**

```rust
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
```

- [ ] **Step 3: Run the test to verify it fails**

Run: `cargo test --test model_test`
Expected: FAIL — `pp_ss_r::model` does not exist.

- [ ] **Step 4: Write `src/model.rs`**

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PaymentPage {
    pub init: InitPayload,
    pub process: ProcessPayload,
}

#[derive(Debug, Deserialize)]
pub struct InitPayload {
    pub client_auth_token: String,
    pub client_id: String,
    pub environment: String,
    pub merchant_id: String,
    pub return_url: String,
    pub language: String,
    pub is_payment_link: bool,
    pub tenant: TenantInfo,
}

#[derive(Debug, Deserialize)]
pub struct TenantInfo {
    pub assets_domain: String,
    pub tenant_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ProcessPayload {
    pub amount: String,
    pub currency: String,
    pub order_id: String,
    pub description: String,
    pub order_type: String,
    pub payment_links_expiry: String,
    pub source_object: String,
    pub merchant_id: String,
    pub metadata: String,
}

pub fn load_sample() -> PaymentPage {
    let raw = include_str!("../data/order.json");
    serde_json::from_str(raw).expect("data/order.json must be valid PaymentPage JSON")
}
```

- [ ] **Step 5: Add module to `src/lib.rs`**

Add near the top of `src/lib.rs`:

```rust
pub mod model;
```

- [ ] **Step 6: Run the test to verify it passes**

Run: `cargo test --test model_test`
Expected: PASS (both tests).

- [ ] **Step 7: Commit**

```bash
git add src/model.rs src/lib.rs data/order.json tests/model_test.rs
git commit -m "feat: payment payload model + sample order.json"
```

---

### Task 3: maud rendering + amount formatting

**Files:**
- Create: `src/render.rs`
- Modify: `src/lib.rs` (add `pub mod render;`)
- Test: `tests/render_test.rs`

**Interfaces:**
- Consumes: `PaymentPage` from `pp_ss_r::model`.
- Produces:
  - `pub fn format_amount(currency: &str, amount: &str) -> String` — prepends the currency symbol (GBP→£, USD→$, EUR→€, else "").
  - `pub fn lang_code(language: &str) -> &'static str` — maps `"english"`→`"en"`, else `"en"`.
  - `pub fn render_payment_page(page: &PaymentPage) -> maud::Markup` — full HTML document.

- [ ] **Step 1: Write the failing test `tests/render_test.rs`**

```rust
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
```

Note: this requires `description` to be mutable/public — it already is `pub` on the struct.

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test --test render_test`
Expected: FAIL — `pp_ss_r::render` does not exist.

- [ ] **Step 3: Write `src/render.rs`**

```rust
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

pub fn render_payment_page(page: &PaymentPage) -> Markup {
    let amount_display = format_amount(&page.process.currency, &page.process.amount);
    html! {
        (DOCTYPE)
        html lang=(lang_code(&page.init.language)) {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Barclaycard Payment" }
                link rel="stylesheet" href="/assets/style.css";
            }
            body {
                header.pp-header {
                    img.pp-logo src="/assets/img/barclaycard-logo.png" alt="Barclaycard";
                }
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
            }
        }
    }
}
```

- [ ] **Step 4: Add module to `src/lib.rs`**

Add near the top of `src/lib.rs`:

```rust
pub mod render;
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `cargo test --test render_test`
Expected: PASS (all three tests).

- [ ] **Step 6: Commit**

```bash
git add src/render.rs src/lib.rs tests/render_test.rs
git commit -m "feat: maud payment page rendering + amount formatting"
```

---

### Task 4: Page routes (`/` and `/order/{id}`)

**Files:**
- Modify: `src/lib.rs` (add routes to `app_config`, add handlers)
- Test: `tests/routes_test.rs`

**Interfaces:**
- Consumes: `render_payment_page`, `load_sample`, `PaymentPage`.
- Produces: `GET /` and `GET /order/{id}` returning the rendered page as `text/html; charset=utf-8`. For `/order/{id}`, the path id replaces `process.order_id`.

- [ ] **Step 1: Write the failing test `tests/routes_test.rs`**

```rust
use actix_web::{test, App};
use pp_ss_r::app_config;

#[actix_web::test]
async fn root_renders_page() {
    let app = test::init_service(App::new().configure(app_config)).await;
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let ct = resp.headers().get("content-type").unwrap().to_str().unwrap().to_string();
    assert!(ct.starts_with("text/html"));
    let body = String::from_utf8(test::read_body(resp).await.to_vec()).unwrap();
    assert!(body.contains("£0.01"));
}

#[actix_web::test]
async fn order_id_is_echoed_and_escaped() {
    let app = test::init_service(App::new().configure(app_config)).await;
    let req = test::TestRequest::get().uri("/order/ord_%3Cb%3Ex").to_request();
    let resp = test::call_service(&app, req).await;
    let body = String::from_utf8(test::read_body(resp).await.to_vec()).unwrap();
    // decoded id is ord_<b>x ; maud must escape the angle brackets
    assert!(body.contains("ord_&lt;b&gt;x"), "id echoed and HTML-escaped");
    assert!(!body.contains("ord_<b>x"), "must not emit raw tags");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test --test routes_test`
Expected: FAIL — `/` and `/order/{id}` not routed (404).

- [ ] **Step 3: Add handlers and routes in `src/lib.rs`**

Update `app_config` and add handlers:

```rust
use actix_web::{web, HttpResponse};
use crate::model::load_sample;
use crate::render::render_payment_page;

pub fn app_config(cfg: &mut web::ServiceConfig) {
    cfg.route("/healthz", web::get().to(healthz));
    cfg.route("/", web::get().to(index));
    cfg.route("/order/{id}", web::get().to(order));
}

async fn index() -> HttpResponse {
    let page = load_sample();
    html_response(render_payment_page(&page).into_string())
}

async fn order(path: web::Path<String>) -> HttpResponse {
    let mut page = load_sample();
    page.process.order_id = path.into_inner();
    html_response(render_payment_page(&page).into_string())
}

fn html_response(body: String) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body)
}
```

Keep the existing `healthz` handler. Ensure `use` imports at top of `src/lib.rs` are consolidated.

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test --test routes_test`
Expected: PASS (both tests).

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs tests/routes_test.rs
git commit -m "feat: payment page routes / and /order/{id}"
```

---

### Task 5: Mirror static assets + serve them

**Files:**
- Create: `scripts/fetch-assets.sh`
- Create: `static/style.css`
- Create: `static/img/` (populated by the fetch script)
- Create: `static/fonts/` (populated by the fetch script)
- Modify: `src/lib.rs` (mount `actix-files`)
- Test: `tests/static_test.rs`

**Interfaces:**
- Consumes: existing `app_config`.
- Produces: `GET /assets/{path}` served from the `static/` directory; path traversal blocked by actix-files defaults.

- [ ] **Step 1: Write `scripts/fetch-assets.sh`**

```bash
#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p static/img static/fonts
UA="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36"

dl() { curl -fsSL -A "$UA" "$1" -o "$2" && echo "ok  $2" || echo "MISS $1 (using fallback)"; }

dl "https://assets.juspay.in/hyper/assets/in.juspay.merchants/images/testbalaji/jp_barclaycardlogoheaderbig_20260724155833.png" static/img/barclaycard-logo.png
dl "https://assets.juspay.in/hyper/images/common/jp_secured.png" static/img/secured.png
dl "https://assets.juspay.in/hyper/assets/in.juspay.merchants/fonts/testbalaji/BarclaysEffra/BarclaysEffra-Regular.ttf" static/fonts/BarclaysEffra-Regular.ttf
dl "https://assets.juspay.in/hyper/assets/in.juspay.merchants/fonts/testbalaji/BarclaysEffra/BarclaysEffra-SemiBold.ttf" static/fonts/BarclaysEffra-SemiBold.ttf
dl "https://assets.juspay.in/hyper/assets/in.juspay.merchants/fonts/testbalaji/BarclaysEffra/BarclaysEffra-Bold.ttf" static/fonts/BarclaysEffra-Bold.ttf
echo "done"
```

- [ ] **Step 2: Run the fetch script**

Run: `bash scripts/fetch-assets.sh`
Expected: files land in `static/img` and `static/fonts`. If a URL prints `MISS`, note it — the logo test in Step 5 tolerates a placeholder, but record which asset failed. If the logo is missing, create a 1×1 placeholder so the file exists: `printf '' > static/img/barclaycard-logo.png` is NOT valid PNG — instead skip and let the CSS handle absence; the test only checks the CSS route.

- [ ] **Step 3: Write `static/style.css`**

```css
@font-face {
  font-family: "BarclaysEffra";
  src: url("/assets/fonts/BarclaysEffra-Regular.ttf") format("truetype");
  font-weight: 400; font-display: swap;
}
@font-face {
  font-family: "BarclaysEffra";
  src: url("/assets/fonts/BarclaysEffra-SemiBold.ttf") format("truetype");
  font-weight: 600; font-display: swap;
}
:root { --bar-blue: #00aeef; --ink: #1d1d1b; --line: #e0e0e0; }
* { box-sizing: border-box; }
body {
  font-family: "BarclaysEffra", system-ui, sans-serif;
  margin: 0; color: var(--ink); background: #f5f6f8;
}
.pp-header { background: #fff; padding: 16px 24px; border-bottom: 1px solid var(--line); }
.pp-logo { height: 32px; }
.pp-main { max-width: 480px; margin: 24px auto; padding: 0 16px; }
.pp-summary { background: #fff; border: 1px solid var(--line); border-radius: 8px; padding: 20px; margin-bottom: 16px; }
.pp-title { font-size: 18px; margin: 0 0 8px; }
.pp-amount { font-size: 28px; font-weight: 600; margin: 8px 0 0; }
.pp-order-id { color: #666; font-size: 12px; }
.pp-form { background: #fff; border: 1px solid var(--line); border-radius: 8px; padding: 20px; display: flex; flex-direction: column; gap: 6px; }
.pp-form label { font-size: 13px; margin-top: 8px; }
.pp-form input { padding: 12px; border: 1px solid var(--line); border-radius: 6px; font-size: 16px; }
.pp-row { display: flex; gap: 12px; }
.pp-col { flex: 1; display: flex; flex-direction: column; }
.pp-pay { margin-top: 16px; padding: 14px; background: var(--bar-blue); color: #fff; border: 0; border-radius: 6px; font-size: 16px; font-weight: 600; cursor: pointer; }
.pp-footer { max-width: 480px; margin: 16px auto; padding: 0 16px; color: #666; font-size: 12px; display: flex; align-items: center; gap: 8px; }
.pp-secured { height: 16px; }
```

- [ ] **Step 4: Write the failing test `tests/static_test.rs`**

```rust
use actix_web::{test, App};
use pp_ss_r::app_config;

#[actix_web::test]
async fn serves_stylesheet() {
    let app = test::init_service(App::new().configure(app_config)).await;
    let req = test::TestRequest::get().uri("/assets/style.css").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = String::from_utf8(test::read_body(resp).await.to_vec()).unwrap();
    assert!(body.contains("pp-pay"));
}

#[actix_web::test]
async fn blocks_path_traversal() {
    let app = test::init_service(App::new().configure(app_config)).await;
    let req = test::TestRequest::get().uri("/assets/%2e%2e/Cargo.toml").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(!resp.status().is_success(), "must not serve files outside static/");
}
```

- [ ] **Step 5: Run the test to verify it fails**

Run: `cargo test --test static_test`
Expected: FAIL — `/assets/*` not mounted (404 for style.css).

- [ ] **Step 6: Mount actix-files in `src/lib.rs`**

Add at the end of `app_config` (after the routes):

```rust
cfg.service(actix_files::Files::new("/assets", "static"));
```

Add the import at top of `src/lib.rs`: `use actix_files::Files;` is optional if using the fully-qualified path above.

- [ ] **Step 7: Run the test to verify it passes**

Run: `cargo test --test static_test`
Expected: PASS (both tests).

- [ ] **Step 8: Commit**

```bash
git add scripts/fetch-assets.sh static/style.css static/img static/fonts src/lib.rs tests/static_test.rs
git commit -m "feat: mirror + serve static assets (css, logo, fonts)"
```

---

### Task 6: `/pay` demo stub confirmation

**Files:**
- Modify: `src/render.rs` (add `render_confirmation`)
- Modify: `src/lib.rs` (add `POST /pay` route + handler)
- Test: `tests/pay_test.rs`

**Interfaces:**
- Consumes: `PaymentPage`, `load_sample`, `render` helpers.
- Produces:
  - `pub fn render_confirmation(page: &PaymentPage) -> maud::Markup` — a simple "payment received (demo)" page echoing amount + order id.
  - `POST /pay` → 200 HTML confirmation. No form parsing, no validation, no storage.

- [ ] **Step 1: Write the failing test `tests/pay_test.rs`**

```rust
use actix_web::{test, App};
use pp_ss_r::app_config;

#[actix_web::test]
async fn pay_returns_confirmation() {
    let app = test::init_service(App::new().configure(app_config)).await;
    let req = test::TestRequest::post().uri("/pay").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body = String::from_utf8(test::read_body(resp).await.to_vec()).unwrap();
    assert!(body.to_lowercase().contains("received"), "confirmation text");
    assert!(body.contains("£0.01"), "echoes amount");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test --test pay_test`
Expected: FAIL — `POST /pay` not routed.

- [ ] **Step 3: Add `render_confirmation` to `src/render.rs`**

```rust
pub fn render_confirmation(page: &PaymentPage) -> Markup {
    let amount_display = format_amount(&page.process.currency, &page.process.amount);
    html! {
        (DOCTYPE)
        html lang=(lang_code(&page.init.language)) {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Payment received" }
                link rel="stylesheet" href="/assets/style.css";
            }
            body {
                header.pp-header {
                    img.pp-logo src="/assets/img/barclaycard-logo.png" alt="Barclaycard";
                }
                main.pp-main {
                    section.pp-summary {
                        h1.pp-title { "Payment received (demo)" }
                        p.pp-amount { (amount_display) }
                        p.pp-order-id { "Order: " (page.process.order_id) }
                        p { "This is a demo confirmation. No real payment was processed." }
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 4: Add route + handler in `src/lib.rs`**

In `app_config` add: `cfg.route("/pay", web::post().to(pay));`

Handler:

```rust
use crate::render::render_confirmation;

async fn pay() -> HttpResponse {
    let page = load_sample();
    html_response(render_confirmation(&page).into_string())
}
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `cargo test --test pay_test`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/render.rs src/lib.rs tests/pay_test.rs
git commit -m "feat: /pay demo confirmation stub"
```

---

### Task 7: End-to-end verification (headless Chromium) + docs

**Files:**
- Create: `scripts/e2e.mjs`
- Create: `README.md`
- Test: manual e2e run (documented steps below)

**Interfaces:**
- Consumes: the running server on `127.0.0.1:8080`.
- Produces: a screenshot `scratch/home.png` and console assertions; a README documenting build/run/test.

- [ ] **Step 1: Write `scripts/e2e.mjs`**

Uses the Nix-provided Chromium via the DevTools protocol with no extra npm deps (launch headless, navigate, screenshot, dump text). If a Node CDP client is unavailable, fall back to Chromium's built-in screenshot flag:

```js
// Minimal, dependency-free: shell out to chromium --headless --screenshot.
// Kept as documentation; the canonical check is the shell command in Step 3.
```

Given zero-dependency constraint, the real check is a shell command (Step 3); `e2e.mjs` is a thin placeholder documenting intent.

- [ ] **Step 2: Write `README.md`**

```markdown
# pp-ss-r — SSR Payment Page

Fast server-side-rendered payment page in Rust (actix-web + maud).

## Develop
    nix develop           # rust toolchain + chromium
    bash scripts/fetch-assets.sh   # one-time: mirror logo/fonts into static/
    cargo run             # http://127.0.0.1:8080

## Test
    cargo test            # unit + integration
    # e2e screenshot (server must be running):
    "$CHROME_BIN" --headless --disable-gpu --screenshot=scratch/home.png --window-size=480,900 http://127.0.0.1:8080

## Routes
- GET  /             payment page (sample order)
- GET  /order/{id}   payment page with the given order id
- POST /pay          demo confirmation (no processing)
- GET  /healthz      health check
- GET  /assets/*     static files
```

- [ ] **Step 3: Run the end-to-end check**

```bash
cargo run &          # start server
sleep 2
curl -s -o /dev/null -w "root: %{http_code} %{time_total}s\n" http://127.0.0.1:8080/
curl -s http://127.0.0.1:8080/ | grep -q "£0.01" && echo "amount present"
curl -s -o /dev/null -w "healthz: %{http_code}\n" http://127.0.0.1:8080/healthz
mkdir -p scratch
"$CHROME_BIN" --headless --disable-gpu --screenshot=scratch/home.png --window-size=480,900 http://127.0.0.1:8080 && echo "screenshot ok"
kill %1
```

Expected: `root: 200`, `amount present`, `healthz: 200`, `screenshot ok`, render time well under 50ms.

- [ ] **Step 4: Commit**

```bash
git add scripts/e2e.mjs README.md
git commit -m "docs: README + e2e verification steps"
```

---

## Self-Review Notes

- **Spec coverage:** model (Task 2), maud rendering (Task 3), routes (Task 4), local assets mirrored + served (Task 5), `/pay` stub (Task 6), Nix flake + healthz (Task 1), verification incl. headless Chromium (Task 7). All spec sections covered.
- **Review Focus:** unknown currency (Task 3), empty description (Task 3), order id echo + escaping (Task 4), missing JSON field fails at load (Task 2), static path traversal (Task 5) — each has a test in its owning task.
- **Type consistency:** `app_config`, `load_sample`, `PaymentPage`, `format_amount`, `lang_code`, `render_payment_page`, `render_confirmation`, `html_response` names are consistent across tasks.
