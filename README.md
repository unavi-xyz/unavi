<div align="center">
<img src="./apps/client/public/images/plug.png" />
<h1>The Wired</h1>
<p>A decentralized, open source, browser-based VR social paltform.</p>

<img alt="Discord" src="https://img.shields.io/discord/918705784311939134?label=discord">
<img alt="GitHub" src="https://img.shields.io/github/license/wired-xr/wired">
<img alt="Twitter Follow" src="https://img.shields.io/twitter/follow/TheWiredXR?style=social">
</div>

## What's inside?

This turborepo uses [Yarn](https://yarnpkg.com/) as a package manager. It includes the following apps/packages:

### Apps

- `client`: a website used to access The Wired
- `server`: a server for hosting spaces

### Packages

- `scene`: a library for creating 3d scenes
- `config`: `eslint` configurations
- `tsconfig`: `tsconfig.json`s used throughout the monorepo

Each app/package is 100% [TypeScript](https://www.typescriptlang.org/).

### Utilities

- [TypeScript](https://www.typescriptlang.org/) for static type checking
- [ESLint](https://eslint.org/) for code linting
- [Prettier](https://prettier.io) for code formatting

### Build

To build all apps and packages, run the following command:

```bash
yarn run build
```

### Develop

To develop all apps and packages, run the following command:

```bash
yarn run dev
```

## Useful Links

Learn more about the power of Turborepo:

- [Pipelines](https://turborepo.org/docs/features/pipelines)
- [Caching](https://turborepo.org/docs/features/caching)
- [Remote Caching (Beta)](https://turborepo.org/docs/features/remote-caching)
- [Scoped Tasks](https://turborepo.org/docs/features/scopes)
- [Configuration Options](https://turborepo.org/docs/reference/configuration)
- [CLI Usage](https://turborepo.org/docs/reference/command-line-reference)
