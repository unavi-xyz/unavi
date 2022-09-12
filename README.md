<div align="center">
  <p>
    <img src="./assets/Hero.png" />
  </p>

<i>"It's just an advanced medium for communication. Don't get it confused with the real world."</i>

  <img alt="Discord" src="https://img.shields.io/discord/918705784311939134?label=discord">
  <img alt="GitHub" src="https://img.shields.io/github/license/wired-labs/wired">
  <img alt="Twitter Follow" src="https://img.shields.io/twitter/follow/TheWiredXR?style=social">
</div>

## Features

- Decentralized identity / social graph using [Lens Protocol](https://lens.dev/)
- Open backend, anyone can run their own servers
- Custom multi-threaded, glTF-based game engine
- [VRM](https://vrm.dev/) avatar support
- Visual editor for creating spaces

## What's inside?

This [turborepo](https://turborepo.org/) uses [Yarn](https://classic.yarnpkg.com/lang/en/) as a package manager. It includes the following apps / packages:

### Apps

- [client](apps/client): a website used to access the Wired
- [examples](apps/examples): a simple website for testing the engine
- [docs](apps/docs): a documentation website
- [host](apps/host): a dockerized server for hosting spaces

### Packages

- [engine](packages/engine): a 3D game engine
- [eslint-config-custom](packages/eslint-config-custom): a custom eslint config used throughout the repo
- [lens](packages/lens): queries and types for interacting with Lens
- [tsconfig](packages/tsconfig): tsconfigs used throughout the repo

### Utilities

- [TypeScript](https://www.typescriptlang.org/) for static type checking
- [ESLint](https://eslint.org/) for code linting
- [Prettier](https://prettier.io) for code formatting

### Install

To install all apps and packages, run the following command:

```bash
yarn install
```

If you run into issues installing, it's probably mediasoup. Follow the steps on their [installation guide](https://mediasoup.org/documentation/v3/mediasoup/installation/) to get it to work. Mediasoup tends to be much easier to install on Linux, so if you're on Windows, consider using [Windows Subsystem for Linux (WSL)](https://docs.microsoft.com/en-us/windows/wsl/install).

Then run codegen to generate some files:

```bash
yarn codegen
```

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
