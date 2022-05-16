<div align="center">
  <p>
    <img src="./assets/HeroRound.png" />
  </p>

  <p>An open source, decentralized, VR social platform</p>

  <img alt="Discord" src="https://img.shields.io/discord/918705784311939134?label=discord">
  <img alt="GitHub" src="https://img.shields.io/github/license/wired-xr/wired">
  <img alt="Twitter Follow" src="https://img.shields.io/twitter/follow/TheWiredXR?style=social">
</div>

## Tech Stack

- Decentralized backend using [Lens Protocol](https://lens.dev/) and [IPFS](https://ipfs.io/)
- Peer to peer voice and data channels over [WebRTC](https://webrtc.org/)
- 3d rendering using [Threejs](https://github.com/mrdoob/three.js) / [React Three Fiber](https://github.com/pmndrs/react-three-fiber)
- Physics using [Cannon ES](https://github.com/pmndrs/cannon-es) / [React Three Cannon](https://github.com/pmndrs/use-cannon/tree/master/packages/react-three-cannon#readme)

## Development

This turborepo uses [Yarn](https://yarnpkg.com/) as a package manager. It includes the following apps/packages:

### Apps

- `client`: a website used to access The Wired
- `server`: a server for hosting spaces

### Packages

- `avatar`: a VRM avatar library
- `config`: eslint configurations
- `scene`: a library for creating 3d scenes in a standardized format
- `tsconfig`: tsconfigs used throughout the monorepo

### Utilities

- [TypeScript](https://www.typescriptlang.org/) for static type checking
- [ESLint](https://eslint.org/) for code linting
- [Prettier](https://prettier.io) for code formatting

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
