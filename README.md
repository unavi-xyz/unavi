<div align="center">
  <img src="./assets/block-logo.png" alt="UNAVI Logo" height="200" />
  <h1>UNAVI</h1>
  <strong>An open and decentralized web-based metaverse platform.</strong>
</div>

<br />

<div align="center">
  <a href="https://docs.unavi.xyz">
    <img src="https://img.shields.io/badge/docs-read-informational" alt="Docs" />
  </a>
  <a href="https://www.unavi.xyz">
    <img alt="Deployment" src="https://img.shields.io/github/deployments/unavi-xyz/unavi/production">
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

## 📦 What's inside?

This [turborepo](https://turborepo.org/) uses [pnpm](https://pnpm.io/) as a package manager. It includes the following apps / packages:

### Apps

- [client](apps/client): the main website used to access UNAVI
- [host](apps/host): dockerized server for hosting multiplayer worlds

### Packages

- [engine](packages/engine): Extensions of [Lattice](https://github.com/unavi-xyz/lattice)
- [eslint-config-custom](packages/eslint-config-custom): Eslint config used throughout the repo
- [gltf-extension](packages/gltf-extension): [glTF-Transform](https://github.com/donmccurdy/glTF-Transform) extensions used by the client
- [protocol](packages/protocol): Extensions of [The Wired Protocol](https://github.com/unavi-xyz/wired-protocol)
- [tsconfig](packages/tsconfig): tsconfigs used throughout the repo
- [utils](packages/utils): Utilities used by both the client and host

### Utilities

- [Changesets](https://github.com/changesets/changesets) for package versioning
- [ESLint](https://eslint.org/) for code linting
- [Prettier](https://prettier.io) for code formatting
- [TypeScript](https://www.typescriptlang.org/) for static type checking

## 🐋 Docker

The client can be run locally using Docker. To do so, run the following command:

```bash
pnpm docker:prod
```

This will start the client on port 3000. You can then access it at [http://localhost:3000](http://localhost:3000).

To stop the client, run the following command:

```bash
pnpm docker:stop
```

## ⚙️ Development

### Install

To install all apps and packages, run the following command:

```bash
pnpm install
```

> ⚠️ If you run into issues installing, it's probably mediasoup. Follow the steps on their [installation guide](https://mediasoup.org/documentation/v3/mediasoup/installation/) to get it to work (pay attention to the versions very carefully). Mediasoup tends to be easier to install on Linux, so if you're on Windows, consider using [Windows Subsystem for Linux (WSL)](https://docs.microsoft.com/en-us/windows/wsl/install).

### Build

To build all apps and packages, run the following command:

```bash
pnpm build
```

### Develop

To work on the client, you will need to use Docker to run services it relies on (such as the database). To do so, run the following command:

```bash
pnpm docker:dev
```

If you don't need that, you can run all apps and packages in development mode using the following command:

```bash
pnpm dev
```
