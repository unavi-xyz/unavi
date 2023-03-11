<div align="center">
  <img src="./assets/Logo.png" alt="Wired Logo" height="200" />
  <h1>The Wired</h1>
  <strong>An open and decentralized web-based metaverse platform.</strong>
</div>

<br />

<div align="center">
  <a href="https://docs.thewired.space">
    <img src="https://img.shields.io/badge/docs-read-informational" alt="Docs" />
  </a>
  <a href="https://thewired.space">
    <img src="https://therealsujitk-vercel-badge.vercel.app/?app=client-wired" alt="Vercel" />
  </a>
  <a href="https://github.com/wired-labs/wired/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/wired-labs/wired" alt="License" />
  </a>
  <a href="https://discord.gg/VCsAEneUMn">
    <img src="https://img.shields.io/discord/918705784311939134.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2" alt="Discord" />
  </a>
  <a href="https://twitter.com/wired_xr">
    <img src="https://img.shields.io/badge/wired__xr--1DA1F2?logo=twitter" alt="Twitter" />
  </a>
</div>

## 🔥 Features

- Decentralized identity + content distribution over [Ethereum](https://github.com/wired-labs/contracts)
- Open backend, anyone can run their own servers
- Custom multi-threaded, [glTF](https://github.com/KhronosGroup/glTF)-based game engine
- [VRM](https://vrm.dev/) avatar support
- Visual editor for creating spaces

## 📦 What's inside?

This [turborepo](https://turborepo.org/) uses [Yarn](https://classic.yarnpkg.com/lang/en/) as a package manager. It includes the following apps / packages:

### Apps

- [client](apps/client): the main website used to access the Wired
- [docs](apps/docs): a documentation website
- [host](apps/host): dockerized server for hosting spaces

### Packages

- [contracts](packages/contracts): generated contract types
- [engine](packages/engine): a multi-threaded 3D game engine
- [eslint-config-custom](packages/eslint-config-custom): custom eslint config used throughout the repo
- [protocol](packages/protocol): types describing the interface between client and host
- [tsconfig](packages/tsconfig): tsconfigs used throughout the repo

### Utilities

- [TypeScript](https://www.typescriptlang.org/) for static type checking
- [ESLint](https://eslint.org/) for code linting
- [Prettier](https://prettier.io) for code formatting

## 🐋 Docker

The client can be run locally using Docker. To do so, run the following command:

```bash
docker compose up -d
```

This will start the client on port 3000. You can then access it at [http://localhost:3000](http://localhost:3000).

To stop the client, run the following command:

```bash
docker compose down
```

## ⚙️ Development

### Install

To install all apps and packages, run the following command:

```bash
yarn install
```

> ⚠️ If you run into issues installing, it's probably mediasoup. Follow the steps on their [installation guide](https://mediasoup.org/documentation/v3/mediasoup/installation/) to get it to work (pay attention to the versions very carefully). Mediasoup tends to be easier to install on Linux, so if you're on Windows, consider using [Windows Subsystem for Linux (WSL)](https://docs.microsoft.com/en-us/windows/wsl/install).

### Build

To build all apps and packages, run the following command:

```bash
yarn build
```

### Develop

To develop all apps and packages, run the following command:

```bash
yarn dev
```
