# unavi-app

The main UNAVI app.

## Building

A few additional tools are required to build the `unavi-app` crate.
These are installed automatically when using the Nix flake.

### Bevy

Install the required [Bevy dependencies](https://bevyengine.org/learn/quick-start/getting-started/setup/#installing-os-dependencies).

### Cargo

The `cargo-component` and `wac-cli` cargo tools are used to build and compose WASM components.

```bash
cargo install cargo-component
cargo install wac-cli
```

### Mold

The [mold](https://github.com/rui314/mold) linker is used by default when targeting Linux.

### Git Submodules

Make sure your Git submodules are up to date:

```bash
git submodule update --init --recursive
git submodule foreach git pull
```
