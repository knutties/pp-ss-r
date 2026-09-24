# pp-ss-r task runner. Recipes wrap `nix develop` so they work from anywhere
# with nix installed. Run `just` (or `just --list`) to see all recipes.

# Show available recipes
default:
    @just --list

# Run the server. Override the port: `just run 8091`
run port="8080":
    nix develop --command bash -c 'BIND_ADDR=127.0.0.1:{{port}} cargo run'

# Run the release binary (build first with `just build`). Override: `just serve 8091`
serve port="8080":
    nix develop --command bash -c 'BIND_ADDR=127.0.0.1:{{port}} cargo run --release'

# Run the full test suite
test:
    nix develop --command cargo test

# Build the optimized release binary
build:
    nix develop --command cargo build --release

# Mirror static assets (Barclaycard logo, BarclaysEffra fonts) into static/
assets:
    nix develop --command bash scripts/fetch-assets.sh

# Format the code
fmt:
    nix develop --command cargo fmt

# Lint with clippy
lint:
    nix develop --command cargo clippy --all-targets

# Screenshot the running server (server must be running on the given port)
shot port="8080":
    nix develop --command bash -c 'mkdir -p scratch && "$CHROME_BIN" --headless --disable-gpu --screenshot=scratch/home.png --window-size=480,900 http://127.0.0.1:{{port}}'
