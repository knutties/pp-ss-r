# SSR Payment Page — Design Spec

**Date:** 2026-09-24
**Status:** Draft for review

## Context

We want a foundational setup for serving **fast, server-side-rendered payment
pages** in Rust. The reference is a live Barclaycard/Juspay hosted checkout:
`https://secure.barclaycardpayments.com/payment-page/order/ordeh_84ee33ebe63d40dcabebea151ad5e99f`.

Investigation of that page revealed:

- It is **already SSR'd by a Rust server** (`window.fromRustServer=true`,
  `window.serverSideKeys['fromSSR']=true`).
- The 986 KB response is almost entirely **inlined JSON + JS bundles**; the
  static HTML body contains only ~2 `<div>`s. The visible DOM (order summary,
  card fields, buttons) is drawn **client-side** by the Juspay `hyperpay`
  bundles.
- What the server actually renders is a **bootstrap `ssr_payload` JSON**
  describing the merchant, order, and tenant. This is the natural "key portions
  as a JSON data structure" the project calls for.

Because the visible page is JS-drawn (not static HTML we can copy), our
foundation will **render a clean, representative payment page ourselves**,
driven by a JSON payload modeled on the real `ssr_payload`. The objective is a
solid, fast, extensible base — not a byte-for-byte clone.

## Goals

- actix-web server that renders a payment page from a JSON payload.
- Fastest practical templating: **maud** (compile-time, no runtime parsing).
- Serve all static assets locally from Rust (self-contained, no CDN at runtime).
- Reproducible dev environment via **flake.nix** (Rust toolchain + headless
  Chromium for tests).
- JSON schema modeled on the real payload so real order data can drop in later.

## Non-Goals

- No real payment processing or gateway integration (the "pay" action is a stub).
- No client-side JS framework / Juspay bundles.
- No byte-for-byte visual clone of the JS-rendered original.
- No persistence layer / database.

## Data Model (`src/model.rs`)

Serde structs mirroring the real payload. Representative shape:

```rust
struct PaymentPage {
    init: InitPayload,
    process: ProcessPayload,
}
struct InitPayload {
    client_auth_token: String,
    client_id: String,
    environment: String,      // "production"
    merchant_id: String,      // "hamishtest"
    return_url: String,
    language: String,         // "english"
    is_payment_link: bool,
    tenant: TenantInfo,
}
struct TenantInfo {
    assets_domain: String,
    tenant_id: String,        // "barclays"
}
struct ProcessPayload {
    amount: String,           // "0.01"
    currency: String,         // "GBP"
    order_id: String,
    description: String,
    order_type: String,       // "ORDER_PAYMENT"
    payment_links_expiry: String,
    source_object: String,    // "ADAPTER_BARCLAYS"
    merchant_id: String,
    metadata: String,
}
```

A sample instance lives in `data/order.json`, loaded at startup (and per-request
by id in a later iteration). Amount is formatted for display (e.g. `£0.01`)
using currency + amount.

## Rendering (`src/render.rs`)

- `fn render_payment_page(page: &PaymentPage) -> maud::Markup`
- Produces a complete HTML document: Barclaycard logo header, order summary
  (merchant, description, amount), a card-details form (card number, expiry,
  CVC, name), a "Pay £X" submit button posting to `/pay`, and a secure-payment
  footer. Uses the mirrored BarclaysEffra font and local stylesheet.
- Semantic, accessible markup (labels tied to inputs, `inputmode`/`autocomplete`
  attributes on card fields). No client JS required to display.

## Server (`src/main.rs`)

actix-web with these routes:

| Route              | Handler                                             |
|--------------------|-----------------------------------------------------|
| `GET /`            | Render page from the sample payload.                |
| `GET /order/{id}`  | Render page; `{id}` echoed into the payload's order.|
| `GET /healthz`     | `200 OK` plaintext.                                 |
| `POST /pay`        | Stub: render a simple "payment received (demo)" confirmation page. No processing. |
| `GET /assets/*`    | Static files via `actix-files` from `static/`.      |

Response is `text/html; charset=utf-8`, rendered directly from `Markup` — no
runtime template parsing, no DB.

## Static Assets (`static/`)

**Mirror the real assets**, served locally:

- Barclaycard header logo — from
  `assets.juspay.in/.../testbalaji/jp_barclaycardlogoheaderbig_20260724155833.png`
- "Secured" / shield icons from the Juspay common image set.
- BarclaysEffra fonts (Regular / SemiBold / Bold `.ttf`) from
  `assets.juspay.in/.../fonts/testbalaji/BarclaysEffra/`.
- `static/style.css` — our own stylesheet approximating the Barclaycard look
  (colors, spacing, card layout), referencing `@font-face` for BarclaysEffra.

Assets are downloaded once at build/setup time into `static/` and committed, so
runtime has zero external dependencies. (Note: these assets are Barclays/Juspay
property, used here only to reproduce the reference for this foundation.)

## Project Layout

```
flake.nix                 # devShell: rust toolchain + chromium
.envrc                    # optional direnv: use flake
Cargo.toml
src/
  main.rs                 # actix server + routes + static service
  model.rs                # serde structs
  render.rs               # maud templates
data/order.json           # sample payload
static/
  style.css
  fonts/BarclaysEffra-*.ttf
  img/barclaycard-logo.png, secured.png, ...
tests/
  render_test.rs          # unit: payload -> HTML contains amount/fields
scripts/
  fetch-assets.sh         # one-time asset mirror
  e2e.mjs                 # headless chromium: screenshot + assert (optional)
docs/superpowers/specs/2026-09-24-ssr-payment-page-design.md
```

## Dependencies

- `actix-web` — web server
- `actix-files` — static file serving
- `maud` — compile-time HTML templating
- `serde` / `serde_json` — payload (de)serialization
- (dev) `chromium` via Nix for e2e screenshot testing

## Verification

1. `nix develop` → environment builds; `cargo build` succeeds.
2. `cargo test` → render unit test asserts the HTML contains `£0.01`, `GBP`,
   card field ids, and the Barclaycard logo path.
3. `cargo run` → `curl -s localhost:8080/` returns HTML with the expected
   sections; `curl localhost:8080/healthz` → 200; `curl localhost:8080/assets/style.css`
   serves the stylesheet.
4. Headless Chromium (Nix) loads `localhost:8080`, screenshots it, and asserts
   the amount + form fields are visible. Also record raw render latency via a
   `curl -w "%{time_total}"` timing check.

## Open Questions / Future Work

- Per-order data source (DB / API) — out of scope now; `/order/{id}` currently
  echoes the id into the sample payload.
- Real gateway integration behind a trait — future iteration.
- Localization (payload carries `language`) — single language now.
