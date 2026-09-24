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

- `GET  /`                 payment page (fetches the default order)
- `GET  /order/{id}`       payment page for the given order id
- `POST /pay`              demo confirmation (rendered locally, no processing)
- `GET  /api/orders/{id}`  order data as JSON (the data service)
- `GET  /healthz`          health check
- `GET  /assets/*`         static files (css, fonts, images)

## Data fetch

The render service and the data live behind separate endpoints. The page
handlers (`/`, `/order/{id}`) **fetch** order JSON over HTTP from
`{DATA_BASE_URL}/api/orders/{id}` (via a shared `reqwest` client) and then
render it. By default `DATA_BASE_URL=http://127.0.0.1:8080`, so the server
serves its own data; point it at a real data service to split them:

    DATA_BASE_URL=https://orders.internal cargo run

Add `?delay_ms=<n>` to simulate a slow upstream — the API sleeps that long and
the page's `fetch_ms` reflects it:

    curl "http://127.0.0.1:8080/order/abc?delay_ms=250"   # fetch_ms ~250

A failed/timed-out fetch renders a `502` error page. (reqwest is HTTP-only here;
enable its `rustls-tls` feature to fetch external HTTPS.)

## Docker

    docker build -t pp-ss-r .
    docker run -p 8080:8080 pp-ss-r        # http://127.0.0.1:8080

Multi-stage build (Rust → `debian:bookworm-slim`), runs as non-root. In the
container `BIND_ADDR=0.0.0.0:8080` and `DATA_BASE_URL=http://127.0.0.1:8080`.

## CI

`.github/workflows/docker.yml` builds and pushes to
`ghcr.io/<owner>/<repo>` on every push to `main`, tagged with a CalVer
minute-granularity version (`YYYY.MM.DD.HHMM`), `sha-<short>`, and `latest`.

## Layout

    src/model.rs    serde structs for the payment payload
    src/render.rs   maud templates (compile-time HTML)
    src/lib.rs      actix routes + app_config
    src/main.rs     server bootstrap (127.0.0.1:8080)
    data/order.json sample payload
    static/         style.css, fonts/, img/

## Logging

Each rendered request emits one JSON line to stdout with a per-phase timing
breakdown. Fetched pages (`/`, `/order/{id}`) report `fetch_ms` (data fetch)
plus `render_ms` and `total_ms`; the local `/pay` reports `load_ms` instead:

    {"level":"INFO","message":"request","ts_ms":1790271702087,"method":"GET",
     "path":"/order/abc","status":200,"fetch_ms":0.63,"render_ms":0.0022,
     "total_ms":0.63,"bytes":1597,"target":"pp_ss_r"}

Control verbosity with `RUST_LOG` (default `info,actix_server::worker=warn`):

    RUST_LOG=debug cargo run

## Security headers

Rendered pages are served with `Cache-Control: no-store` (never cache a
card-entry page), `X-Content-Type-Options: nosniff`, `Referrer-Policy:
no-referrer`, and a strict same-origin `Content-Security-Policy`
(`script-src 'none'`, `frame-ancestors 'none'`). All assets are same-origin,
so the CSP requires no relaxation.

## Notes

- `/pay` is a demo stub — no validation, no payment processing, no storage.
- BarclaysEffra fonts are Dalton Maag proprietary, mirrored here only for local
  reproduction of the reference page.
