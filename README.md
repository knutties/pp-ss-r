# pp-ss-r — SSR Payment Page

Fast server-side-rendered payment page in Rust (actix-web + maud). Renders from
a JSON payload modeled on the real Barclaycard/Juspay `ssr_payload`, serving all
assets locally with no runtime CDN dependency.

## Develop

    nix develop                    # rust toolchain + just (+ chromium on Linux)
    bash scripts/fetch-assets.sh   # one-time: mirror logo/fonts into static/
    cargo run                      # http://127.0.0.1:8080
    BIND_ADDR=127.0.0.1:9099 cargo run   # override the bind address

### With `just`

`just` is in the dev shell (and each recipe wraps `nix develop`, so it also
works from the host if nix is installed):

    just               # list recipes
    just run           # run the server on :8080
    just run 8091      # ...on a different port
    just test          # full test suite
    just build         # release binary
    just assets        # mirror static assets

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

## Logging

Each rendered request emits one JSON line to stdout with a per-phase timing
breakdown (`load_ms` = payload deserialization, `render_ms` = HTML generation,
`total_ms` = whole handler):

    {"level":"INFO","message":"request","ts_ms":1790271702087,"method":"GET",
     "path":"/","status":200,"load_ms":0.0097,"render_ms":0.0022,
     "total_ms":0.0120,"bytes":1597,"target":"pp_ss_r"}

Control verbosity with `RUST_LOG` (default `info,actix_server::worker=warn`):

    RUST_LOG=debug cargo run

## Notes

- `/pay` is a demo stub — no validation, no payment processing, no storage.
- BarclaysEffra fonts are Dalton Maag proprietary, mirrored here only for local
  reproduction of the reference page.
