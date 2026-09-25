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

### Automated (no API key needed)

    cargo test          # or: just test

Covers unit + integration, including the checkout flow against a mock Juspay
API and the 502 failure path.

### Manual: checkout flow against a local mock

Port 8080 may be busy; these use 8099 (server) and 8100 (mock).

1. Start a mock checkout API that returns a session id:

   ```bash
   python3 - <<'PY' &
   from http.server import BaseHTTPRequestHandler, HTTPServer
   import json
   class H(BaseHTTPRequestHandler):
       def do_POST(self):
           self.rfile.read(int(self.headers.get('Content-Length', 0)))
           b = json.dumps({"id": "sess_demo_123"}).encode()
           self.send_response(200); self.send_header("Content-Type", "application/json")
           self.send_header("Content-Length", str(len(b))); self.end_headers(); self.wfile.write(b)
       def log_message(self, *a): pass
   HTTPServer(("127.0.0.1", 8100), H).serve_forever()
   PY
   ```

2. Run the server pointed at the mock:

   ```bash
   CHECKOUT_API_BASE=http://127.0.0.1:8100 JUSPAY_API_KEY='Basic test' \
     BIND_ADDR=127.0.0.1:8099 DATA_BASE_URL=http://127.0.0.1:8099 cargo run
   ```

3. Open <http://127.0.0.1:8099/>, enter an amount + currency, and submit — you
   get the payment page for the created session. Or via curl:

   ```bash
   curl -s -X POST http://127.0.0.1:8099/checkout --data "amount=12.34&currency=GBP" \
     | grep -o '£12.34\|sess_demo_123'
   ```

   Watch the server logs for the `checkout_ms` line. Add `?delay_ms=1500` to
   `/order/{id}` to simulate a slow order fetch.

### Manual: against the real Juspay API

`CHECKOUT_API_BASE` already defaults to the real URL, so just supply your key:

    JUSPAY_API_KEY='Basic <your-value>' BIND_ADDR=127.0.0.1:8099 cargo run
    # open http://127.0.0.1:8099/ , fill the form, submit

### E2E screenshot

Server must be running; `CHROME_BIN` is set by the devShell (system Chrome on
darwin, nixpkgs chromium on Linux):

    "$CHROME_BIN" --headless --disable-gpu \
        --screenshot=scratch/home.png --window-size=480,900 \
        http://127.0.0.1:8099

## Routes

- `GET  /`                 precursor form (amount + currency) — the index
- `GET  /checkout`         precursor form (alias of `/`)
- `POST /checkout`         create a checkout session, then render the payment page
- `GET  /order/{id}`       payment page for the given order id (fetches its data)
- `POST /pay`              demo confirmation (rendered locally, no processing)
- `GET  /api/orders/{id}`  order data as JSON (the data service)
- `GET  /healthz`          health check
- `GET  /assets/*`         static files (css, fonts, images)

## Checkout (create a session)

`GET /checkout` renders a small form (amount + currency). On submit, the server
calls the Juspay checkout-sessions API **server-side**:

    POST {CHECKOUT_API_BASE}/v1/checkout-sessions
    X-API-Key: <JUSPAY_API_KEY>
    Idempotency-Key: <generated uuid>

with a `HOSTED_PAGE` / `AUTH_AND_CAPTURE` payment body (merchant id, return url,
and branding profile come from the sample). On success it renders the payment
page for the created session; on failure it returns a `502`. The request log
gains a `checkout_ms` phase (the API round-trip).

Configure via environment variables:

| Var                 | Default                               | Purpose                          |
|---------------------|---------------------------------------|----------------------------------|
| `JUSPAY_API_KEY`    | *(empty)*                             | `X-API-Key` header value         |
| `CHECKOUT_API_BASE` | `https://api.bpl.eu5.prod.juspay.io`  | checkout API base URL            |

    JUSPAY_API_KEY='Basic <value>' cargo run   # then open /checkout

TLS is via `native-tls` (system Security framework on macOS, OpenSSL on Linux).

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

`delay_ms` is clamped to 30s (`MAX_DELAY_MS`) so a stray large value can't tie
up a worker.

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
    src/checkout.rs checkout-session API request/response + client
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

The data endpoint `/api/orders/{id}` logs its own line with `delay_ms`
(simulated wait) and `serialize_ms` (JSON encoding). A single page load thus
emits two lines — the data-side timing and the render-side `fetch_ms` — whose
difference is the HTTP round-trip overhead.

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
