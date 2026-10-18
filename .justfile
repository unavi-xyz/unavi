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
    just fmt-hsd

fmt-check:
    nix build -L .#checks.x86_64-linux.treefmt
    just fmt-check-hsd

fmt-hsd:
    cargo run --quiet -p hsd-cli -- fmt hsd

fmt-check-hsd:
    cargo run --quiet -p hsd-cli -- fmt --check hsd

test *ARGS:
    cargo nextest run --build-jobs 1 -j 2 {{ARGS}}

deny:
    nix build -L .#checks.x86_64-linux.deny

npm-install:
    npm install --prefix crates/unavi-script

# Fail if the root, launcher, and hsd versions have drifted apart.
check-versions:
    nu -c 'let root = (open Cargo.toml | get workspace.package.version); \
        let launcher = (open launcher/Cargo.toml | get workspace.package.version); \
        let hsd = (open hsd/Cargo.toml | get workspace.package.version); \
        if $root != $launcher or $root != $hsd { \
            print $"version mismatch: root=($root) launcher=($launcher) hsd=($hsd)"; \
            exit 1 \
        } else { \
            print $"versions match: ($root)" \
        }'

ci: update-wit-deps-locked npm-install check (clippy "deny") deny fmt-check check-versions

# Build the guest HSD components into crates/unavi-app/assets/hsd.
hsd *ARGS:
    nu nu/build-hsd.nu {{ARGS}}

# Build only the HSD components a release ships (skips example-* crates).
hsd-release:
    nu nu/build-hsd.nu --no-examples

# Resolve WIT deps and rewrite deps.lock.
update-wit-deps:
    nu nu/update-wit-deps.nu

# Wipe the local dev document stores so home/spaces re-import current scripts.
reset *ARGS:
    nu nu/reset-dev.nu {{ARGS}}

# Materialize WIT deps and fail if it drifts from deps.lock.
update-wit-deps-locked:
    nu nu/update-wit-deps.nu --locked

# Build the shipped WebGL + WebGPU bundles into dist/.
web *ARGS: hsd-release
    trunk build --dist dist-webgl --public-url /webgl/ --no-default-features {{ARGS}}
    trunk build --dist dist-webgpu --public-url /webgpu/ --no-default-features --features webgpu {{ARGS}}
    rm -rf dist
    mkdir dist
    cp crates/unavi-app/loader.html dist/index.html
    mv dist-webgl dist/webgl
    mv dist-webgpu dist/webgpu
    nu nu/web-sizes.nu

# Write .br and .gz copies of the compressible files in dist/. Run `just web` first.
web-compress:
    nu nu/web-compress.nu

# Build both release web bundles and save raw, gzip -6 and brotli -q 11 sizes.
web-metrics: (web "--release")
    nu nu/web-metrics.nu

web_port := "8080"

# Serve the built bundles in dist/ at localhost:8080. Run `just web` first.
web-serve:
    esbuild --serve={{web_port}} --servedir=dist

port := "5000"
debug := ""
# SECRETSPEC_PROFILE is read at compile time, so it must reach the build.
dev_env := "SECRETSPEC_PROFILE=development BEVY_ASSET_ROOT=crates/unavi-app"

# Build the node and app the dev recipes run, HSD assets first.
dev-build: hsd-release
    {{dev_env}} cargo build -p unavi-node -p unavi-app

node: dev-build
    {{dev_env}} ./target/debug/unavi-node --port {{port}}

# Run the app with its persistent identity and store, syncing to a local node.
app: dev-build
    {{dev_env}} UNAVI_SYNC_TARGETS=did:web:localhost%3A{{port}} ./target/debug/unavi-app {{ if debug != "" { "--debug-log" } else { "" } }}

# Run a throwaway app: fresh identity and empty store.
app-memory n="1": dev-build
    sleep 2
    {{dev_env}} UNAVI_SYNC_TARGETS=did:web:localhost%3A{{port}} ./target/debug/unavi-app --in-memory {{ if debug != "" { "--debug-log" } else { "" } }}

# Run a node plus two throwaway apps.
[parallel]
multi-app: node app (app-memory "2")
