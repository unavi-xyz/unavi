---
sidebar_position: 40
sidebar_label: 🛠️ Development
title: Development
---

This guide aims to provide a unified development process for developers of The Wired, ensuring its maintainability and scalability. The processes and steps outlined in this document are based on best practices and past experiences, but may also be modified and adjusted according to the specific needs of your setup.

## 🛠️ Environment Setup

### Step 1 - Install tools

Before starting development, ensure that you have the following tools:

- [Node.js](https://nodejs.org/) v18 or higher installed
- [yarn](https://yarnpkg.com/) installed
- [Docker](https://www.docker.com/) installed
- [Docker compose plugin](https://docs.docker.com/compose/install/) installed
- Ethereum RPC provider (such as [Alchemy](https://www.alchemy.com/))

### Step 2 - Clone the repository

```bash
git clone https://github.com/wired-labs/wired.git
cd wired
```

### Step 3 - Install dependencies

```bash
yarn install
```

### Step 4 - Update environment variables

:::warning

Keep your secrets safe, do not commit them to the repository. You can copy the `.env` file to `.env.local` and update the variables there.

```bash
# Create .env.local
cp apps/client/.env apps/client/.env.local
```

:::

The only environment variable you need to set before running the app is `ETH_PROVIDER`. This is the Ethereum HTTP RPC provider that will be used to connect to the blockchain. You can get one from [Alchemy](https://www.alchemy.com/) or [Infura](https://infura.io/) for free.

```env title=".env.local"
ETH_PROVIDER="..." # Your Ethereum RPC provider
```

## 📝 Branch Management

Use [Git](https://git-scm.com/) as the version control system, create `dev` branches and `main` branches. Developers work on `dev` branches and initiate merge requests to the main branch after completing tasks.

## 🏗️ Building

```bash
# Run the apps in development mode
yarn docker:dev
```

The client is hosted at [http://localhost:3000](http://localhost:3000), and the documentation at [http://localhost:3100](http://localhost:3100).
