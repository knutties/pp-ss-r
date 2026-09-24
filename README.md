# pp-ss-r — SSR Payment Page

Fast server-side-rendered payment page in Rust (actix-web + maud). Renders from
a JSON payload modeled on the real Barclaycard/Juspay `ssr_payload`, serving all
assets locally with no runtime CDN dependency.

## Develop

    nix develop                    # rust toolchain (+ chromium on Linux)
    bash scripts/fetch-assets.sh   # one-time: mirror logo/fonts into static/
    cargo run                      # http://127.0.0.1:8080
    BIND_ADDR=127.0.0.1:9099 cargo run   # override the bind address

## Test

    cargo test                     # unit + integration

E2E screenshot (server must be running; `CHROME_BIN` is set by the devShell —
system Chrome on darwin, nixpkgs chromium on Linux):

    "$CHROME_BIN" --headless --disable-gpu \
        --screenshot=scratch/home.png --window-size=480,900 \
        http://127.0.0.1:8080

## Routes

- `GET  /`             payment page (sample order)
- `GET  /order/{id}`   payment page with the given order id
- `POST /pay`          demo confirmation (no processing)
- `GET  /healthz`      health check
- `GET  /assets/*`     static files (css, fonts, images)

## Layout

    src/model.rs    serde structs for the payment payload
    src/render.rs   maud templates (compile-time HTML)
    src/lib.rs      actix routes + app_config
    src/main.rs     server bootstrap (127.0.0.1:8080)
    data/order.json sample payload
    static/         style.css, fonts/, img/

## Notes

- `/pay` is a demo stub — no validation, no payment processing, no storage.
- BarclaysEffra fonts are Dalton Maag proprietary, mirrored here only for local
  reproduction of the reference page.
