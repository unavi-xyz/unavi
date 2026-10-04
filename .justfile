default:
    @just --list

check: check-main check-hsd check-launcher

check-main:
    cargo check --examples --tests --all-features --workspace

check-hsd:
    cargo check --manifest-path hsd/Cargo.toml --workspace --all-features

check-launcher:
    cargo check --manifest-path launcher/Cargo.toml --all-features

clippy mode="fix": \
    (_clippy mode "--workspace --examples --tests") \
    (_clippy mode "--manifest-path hsd/Cargo.toml --workspace") \
    (_clippy mode "--manifest-path launcher/Cargo.toml")

_clippy mode flags:
    cargo clippy --no-deps --all-features {{flags}} {{ if mode == "deny" { "-- -D warnings" } else { "--fix --allow-dirty" } }}

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

ci: update-wit-deps-locked npm-install check (clippy "deny") deny fmt-check

# Build the guest HSD components into crates/unavi-client/assets/hsd.
hsd *ARGS:
    nu nu/build-hsd.nu {{ARGS}}

# Build only the HSD components a release ships (skips example-* crates).
hsd-release:
    nu nu/build-hsd.nu --no-examples

# Resolve WIT deps and rewrite deps.lock.
update-wit-deps:
    nu nu/update-wit-deps.nu

# Materialize WIT deps and fail if it drifts from deps.lock.
update-wit-deps-locked:
    nu nu/update-wit-deps.nu --locked

# Build both web client variants (WebGL + WebGPU) into dist/. This is the
# shipped build: devtools (egui inspector, bevy/debug) stay off here even
# though the crate defaults them on for local native dev builds.
web *ARGS:
    trunk build --dist dist-webgl --public-url /webgl/ --no-default-features {{ARGS}}
    trunk build --dist dist-webgpu --public-url /webgpu/ --no-default-features --features webgpu {{ARGS}}
    rm -rf dist
    mkdir dist
    cp crates/unavi-client/loader.html dist/index.html
    mv dist-webgl dist/webgl
    mv dist-webgpu dist/webgpu

port := "5000"
debug := ""
# SECRETSPEC_PROFILE is read when unavi-config expands, at compile time, so it
# has to reach the build; at runtime it does nothing.
dev_env := "SECRETSPEC_PROFILE=development BEVY_ASSET_ROOT=crates/unavi-client"

dev-build:
    {{dev_env}} cargo build -p unavi-server -p unavi-client

server: dev-build
    {{dev_env}} UNAVI_DOMAIN=localhost:{{port}} ./target/debug/unavi-server --port {{port}}

client n: dev-build
    sleep 2
    {{dev_env}} UNAVI_SYNC_TARGETS=did:web:localhost%3A{{port}} ./target/debug/unavi-client --in-memory {{ if debug != "" { "--debug-log" } else { "" } }}

# Run a server plus two clients.
[parallel]
multi-client: server (client "1") (client "2")
