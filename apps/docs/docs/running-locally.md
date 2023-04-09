---
sidebar_position: 40
sidebar_label: ⚒️ Running Locally
title: Running Locally
---

This page will walk you through the steps to run a Wired client locally on your own machine.

## 🏔️ Environment Setup

### Requirements

Before starting, ensure that you have the following tools:

- [Node.js](https://nodejs.org/) v18 or higher installed
- [yarn](https://yarnpkg.com/) installed
- [Docker](https://www.docker.com/) installed
- [Docker compose plugin](https://docs.docker.com/compose/install/) installed
- Ethereum RPC provider (such as [Alchemy](https://www.alchemy.com/) or [Infura](https://infura.io/))

### Step 1 - Clone the repository

```bash
git clone https://github.com/wired-labs/wired.git
cd wired
```

### Step 2 - Install dependencies

```bash
yarn install
```

### Step 3 - Update environment variables

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

## 🏗️ Running the app

Run one of the following commands to start the app. This will make the client available at [http://localhost:3000](http://localhost:3000).

### Production mode

Use this if you want to run your own local instance of the client.

```bash
yarn docker:prod
```

### Development mode

Use this if you are actively developing the client and want to see any changes you make immediately.

```bash
yarn docker:dev
```

### Stopping

To stop the app, press `Ctrl+C` in the terminal where you ran the command. Additionally, you can run the following command to shut down the docker containers:

```bash
yarn docker:stop
```
