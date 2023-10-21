<div align="center">
  <img src="./assets/block-logo.png" alt="UNAVI Logo" height="200" />
  <h1>UNAVI</h1>
  <strong>An open source VR social platform.</strong>
</div>

<br />

<div align="center">
  <a href="https://unavi.xyz">
    <img alt="Deployment" src="https://img.shields.io/github/deployments/unavi-xyz/unavi/production?label=deployment">
  </a>
  <a href="https://github.com/unavi-xyz/unavi/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/unavi-xyz/unavi" alt="License" />
  </a>
  <a href="https://discord.gg/VCsAEneUMn">
    <img src="https://img.shields.io/discord/918705784311939134.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2" alt="Discord" />
  </a>
  <a href="https://twitter.com/unavi_xyz">
    <img src="https://img.shields.io/badge/unavi__xyz--1DA1F2?logo=twitter" alt="Twitter" />
  </a>
</div>

## ◻️ About

UNAVI is an open source VR social platform - a view into [The Wired](https://github.com/unavi-xyz/wired-protocol).
Anyone can run their own servers, modify their client, and extend the underlying protocol to add new features.

The UNAVI engine is written in Rust, using [Bevy](https://bevyengine.org/).
UNAVI provides both a web client and native build.

## 📦 What's inside?

- [app](/app) Native client
- [engine](/engine) Core engine library
- [server](/server) Home server
- [web](/web) Web client

## ⚙️ Development (with Nix)

### Build

Build all crates in release mode:

```bash
nix build
```

Or a specific crate with:

```bash
nix build .#app
```

### Develop

Enter a development shell:

```bash
nix develop
```

From there you can run the app with something like:

```bash
cargo run -p unavi-app --features bevy/dynamic_linking --features bevy/wayland
```

Or run the web client using `cargo-leptos`:

```bash
cargo leptos watch
```
