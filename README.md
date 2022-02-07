# The Wired

## What's inside?

This monorepo uses [Yarn](https://yarnpkg.com/) as a package manager. It includes the following apps/packages:

### Apps

- `site`: a [Next.js](https://nextjs.org) app

### Packages

- `3d`: [Three.js](https://github.com/pmndrs/react-three-fiber) components
- `ceramic`: helpers for interacting with [Ceramic](https://ceramic.network/)
- `ui`: react ui components
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
