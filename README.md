<div align="center">
  <img src="./crates/unavi-app/assets/images/logo.png" alt="UNAVI Logo" height="200" />
  <h1>UNAVI</h1>
  <strong>An open-source VR social platform.</strong>
</div>

<br />

<div align="center">
  <a href="https://github.com/unavi-xyz/unavi/actions/workflows/ci.yml">
    <img alt="CI" src="https://github.com/unavi-xyz/unavi/actions/workflows/ci.yml/badge.svg">
  </a>
  <a href="https://github.com/unavi-xyz/unavi/blob/main/LICENSE">
    <img alt="License" src="https://img.shields.io/github/license/unavi-xyz/unavi" />
  </a>
  <a href="https://discord.gg/cazUfCCgHJ">
    <img alt="Discord" src="https://img.shields.io/discord/918705784311939134.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2" />
  </a>
  <a href="https://twitter.com/unavi_xyz">
    <img alt="Twitter" src="https://img.shields.io/badge/unavi__xyz--1DA1F2?logo=twitter" />
  </a>
</div>

## ⬜ About

UNAVI is an open-source VR social platform, built on [The Wired 🔌](https://github.com/unavi-xyz/wired-protocol).
Anyone can run their own servers, modify their client, and extend the underlying protocol to add new features.

UNAVI is almost entirely written in Rust 🦀.
The app is built using [Bevy](https://bevyengine.org/) and makes heavy use of [WebAssembly](https://webassembly.org/) for user scripting.
Multiplayer is handled by self-hostable servers acting as relays for communication between players.
UNAVI provides both a web app and native build.

This project is still early, any feedback or contributions are greatly appreciated!
Come join the [Discord](https://discord.gg/cazUfCCgHJ) and say hi!

## ❄️ Development (with Nix)

First enter a development shell:

```bash
nix develop
```

You can then run crates using cargo:

```bash
cargo run -p unavi-app
```

Or serve the web app using trunk:

```bash
trunk serve
```
