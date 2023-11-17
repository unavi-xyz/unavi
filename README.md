<div align="center">
  <img src="./assets/images/block-logo.png" alt="UNAVI Logo" height="200" />
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

UNAVI is an open source VR social platform, built on [The Wired](https://github.com/unavi-xyz/wired-protocol).
Anyone can run their own servers, modify their client, and extend the underlying protocol to add new features.

UNAVI is almost entirely written in Rust 🦀.
The app is built using [Bevy](https://bevyengine.org/) and makes heavy use of [WebAssembly](https://webassembly.org/) for user scripting.
Multiplayer is handled by self-hostable servers acting as relays for communication between players.
UNAVI provides both a web client and native build.

This project is still early, any feedback or contributions are really appreciated!
Come join the [Discord](https://discord.gg/VCsAEneUMn) and say hi!

## ❄️ Development (with Nix)

### Build

Build all crates in release mode:

```bash
nix build
```

### Develop

Enter a development shell:

```bash
nix develop
```

From there you can run the native client with something like:

```bash
cargo run -p unavi-native --features bevy/dynamic_linking
```

Or run the web client using `cargo-leptos`:

```bash
cargo leptos watch
```
