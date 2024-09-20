# unavi-app

<!-- cargo-rdme start -->

The core UNAVI app, built with [Bevy](https://bevyengine.org/).

## Building

A few additional tools are required to build the `unavi-app` crate.
These are handled automatically if using the Nix flake, or they can be installed manually if you are not so based.

### Cargo

The `cargo-component` and `wac-cli` cargo tools are used to build and compose WASM components.

```bash
cargo install cargo-component
cargo install wac-cli
```

### Cap'n Proto

[Cap'n Proto](https://capnproto.org/install.html) must be installed to compile networking schemas.

### Git Submodules

Make sure your Git submodules have been initialized:

```bash
git submodule update --init
```

<!-- cargo-rdme end -->
