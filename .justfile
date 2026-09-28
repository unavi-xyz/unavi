# One source of truth for CI: the `check` job runs `just ci`. The `nu`
# wrappers call the same scripts the Nix packages do.

# List recipes.
default:
    @just --list

# Check all three cargo workspaces (root, hsd, launcher).
check: check-main check-hsd check-launcher

check-main:
    cargo check --examples --tests --all-features --workspace

check-hsd:
    cargo check --manifest-path hsd/Cargo.toml --workspace --all-features

check-launcher:
    cargo check --manifest-path launcher/Cargo.toml --all-features

# Lint all three workspaces, denying warnings as CI does.
clippy: clippy-main clippy-hsd clippy-launcher

clippy-main:
    cargo clippy --no-deps --examples --tests --all-features --workspace -- -D warnings

clippy-hsd:
    cargo clippy --no-deps --manifest-path hsd/Cargo.toml --workspace --all-features -- -D warnings

clippy-launcher:
    cargo clippy --no-deps --manifest-path launcher/Cargo.toml --all-features -- -D warnings

# Format the whole tree with treefmt (Rust, Nix, TOML, YAML, ...).
fmt:
    nix fmt

# Fail if anything is unformatted, without rewriting files.
fmt-check:
    nix build -L .#checks.x86_64-linux.treefmt

# Run the workspace test suite.
test *ARGS:
    cargo nextest run --build-jobs 1 -j 2 {{ARGS}}

# cargo-deny over all three workspaces.
deny:
    nix build -L .#checks.x86_64-linux.deny

# Install the script runtime's npm deps (the wasm build's build.rs needs them).
npm-install:
    npm install --prefix crates/unavi-script

# The CI check job; the wasm build and tests are deliberately not wired in yet.
ci: wasm-update-locked npm-install check clippy deny fmt-check

# Build the guest HSD wasm components into crates/unavi-client/assets/hsd.
wasm *ARGS:
    nu nu/build-wasm.nu {{ARGS}}

# Build only the HSD components a release ships (skips example-* crates).
wasm-release:
    nu nu/build-wasm.nu --release

# Re-resolve WIT deps and rewrite deps.lock. Commit the result.
wasm-update:
    nu nu/update-wasm.nu

# Materialize WIT deps from the committed locks; fails if they drifted.
wasm-update-locked:
    nu nu/update-wasm.nu --locked

# Build both web client variants (WebGL + WebGPU) into dist/.
web *ARGS:
    nu nu/build-web.nu {{ARGS}}

web-release:
    nu nu/build-web.nu --release

# Run a local server plus N clients for multiplayer testing.
multi-client *ARGS:
    nu nu/multi-client.nu {{ARGS}}
