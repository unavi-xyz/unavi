<div align="center">
  <p>
    <img src="./assets/HeroRound.png" />
  </p>

  <p>An open and decentralized 3d social platform</p>

  <img alt="Discord" src="https://img.shields.io/discord/918705784311939134?label=discord">
  <img alt="GitHub" src="https://img.shields.io/github/license/wired-xr/wired">
  <img alt="Twitter Follow" src="https://img.shields.io/twitter/follow/TheWiredXR?style=social">
</div>

## Tech Stack

- Decentralized identity / social graph using [Lens Protocol](https://lens.dev/)
- Decentralized data storage using [IPFS](https://ipfs.io/)
- Open backend, anyone can run their own [host](apps/host) server
- 3d rendering using [Threejs](https://github.com/mrdoob/three.js) / [React Three Fiber](https://github.com/pmndrs/react-three-fiber)
- Physics using [Cannon ES](https://github.com/pmndrs/cannon-es) / [React Three Cannon](https://github.com/pmndrs/use-cannon/tree/master/packages/react-three-cannon#readme)
- VRM avatar support using [Three VRM](https://github.com/pixiv/three-vrm)

## Development

This turborepo uses Yarn as a package manager. It includes the following apps/packages:

### Apps

- [client](apps/client): a website used to access The Wired
- [docs](apps/docs): a documentation website
- [host](apps/host): a server for hosting spaces

### Packages

- [avatar](packages/avatar): a VRM avatar library
- [engine](packages/engine): a 3d game engine for The Wired
- [tsconfig](packages/tsconfig): internal TypeScript configuration files

### Install

To install all apps and packages, run the following command:

```bash
yarn install
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
