default:
    @just --list

check: check-main check-hsd check-launcher

check-main:
    cargo check --examples --tests --all-features --workspace

check-hsd:
    cargo check --manifest-path hsd/Cargo.toml --workspace --all-features

check-launcher:
    cargo check --manifest-path launcher/Cargo.toml --all-features

clippy: clippy-main clippy-hsd clippy-launcher

clippy-main:
    cargo clippy --no-deps --examples --tests --all-features --workspace -- -D warnings

clippy-hsd:
    cargo clippy --no-deps --manifest-path hsd/Cargo.toml --workspace --all-features -- -D warnings

clippy-launcher:
    cargo clippy --no-deps --manifest-path launcher/Cargo.toml --all-features -- -D warnings

fmt:
    nix fmt

fmt-check:
    nix build -L .#checks.x86_64-linux.treefmt

test *ARGS:
    cargo nextest run --build-jobs 1 -j 2 {{ARGS}}

deny:
    nix build -L .#checks.x86_64-linux.deny

npm-install:
    npm install --prefix crates/unavi-script

ci: wasm-update-locked npm-install check clippy deny fmt-check

# Build the guest HSD wasm components into crates/unavi-client/assets/hsd.
wasm *ARGS:
    nu nu/build-wasm.nu {{ARGS}}

# Build only the HSD components a release ships (skips example-* crates).
wasm-release:
    nu nu/build-wasm.nu --release

# Re-resolve WIT deps and rewrite deps.lock.
wasm-update:
    nu nu/update-wasm.nu

# Materialize WIT deps from the committed locks; fails if they drifted.
wasm-update-locked:
    nu nu/update-wasm.nu --locked

# Build both web client variants (WebGL + WebGPU) into dist/.
web *ARGS:
    trunk build --dist dist-webgl --public-url /webgl/ {{ARGS}}
    trunk build --dist dist-webgpu --public-url /webgpu/ --features webgpu {{ARGS}}
    rm -rf dist
    mkdir dist
    cp crates/unavi-client/loader.html dist/index.html
    mv dist-webgl dist/webgl
    mv dist-webgpu dist/webgpu

port := "5000"
debug := ""
dev_env := "SECRETSPEC_PROFILE=development BEVY_ASSET_ROOT=crates/unavi-client"

dev-build:
    {{dev_env}} cargo build -p unavi-server -p unavi-client

server: dev-build
    {{dev_env}} UNAVI_SYNC_TARGETS=did:web:localhost%3A{{port}} UNAVI_DOMAIN=localhost:{{port}} ./target/debug/unavi-server --port {{port}}

client n: dev-build
    sleep 2
    {{dev_env}} UNAVI_SYNC_TARGETS=did:web:localhost%3A{{port}} UNAVI_DOMAIN=localhost:{{port}} ./target/debug/unavi-client --in-memory {{ if debug != "" { "--debug-log" } else { "" } }}

# Run a server plus two clients.
[parallel]
multi-client: server (client "1") (client "2")
