---
sidebar_position: 40
sidebar_label: 🛠️ Development
title: Development
---

This guide aims to provide a unified development process for developers of "The Wired" open source project, ensuring its maintainability and scalability. The processes and steps outlined in this document are based on best practices and past experiences, but may also be modified and adjusted according to the specific needs of the project.

## 🛠️ Environment Setup

⚙️ Before starting development, make ensure that you have the following tools and environment:

- [Node.js](https://nodejs.org/) v18 or higher installed
- [yarn](https://yarnpkg.com/) installed
- [Docker](https://www.docker.com/) installed
- [Docker compose plugin](https://docs.docker.com/compose/install/) installed
- [Alchemy](https://www.alchemy.com/) api key
- [AWS S3](https://aws.amazon.com/tw/s3/) or [DigitalOcean Spaces](https://www.digitalocean.com/products/spaces)

🔧 Update the environment variables:

:::warning

Keep your secrets safe, do not commit them to the repository. You can copy the `.env` file to `.env.local` and update the variables.

:::

```bash
# Copy .env.local file
pushd apps/client && cp .env .env.local && popd
```

|        variables         |              description              |
| :----------------------: | :-----------------------------------: |
|     MYSQL_ROOT_HOST      |      Database root for container      |
|      MYSQL_DATABASE      |             Database name             |
|        MYSQL_USER        |           Database username           |
|      MYSQL_PASSWORD      |        Database user password         |
|   MYSQL_ROOT_PASSWORD    |      Database root user password      |
| NEXT_PUBLIC_CDN_ENDPOINT | CDN endpoint, e.g. DigitalOcean Space |
| NEXT_PUBLIC_DEFAULT_HOST |           Host app endpoint           |
|       DATABASE_URL       |        Database connection URI        |
|     NEXTAUTH_SECRET      |            Session secret             |
|       NEXTAUTH_URL       |          Client app endpoint          |
|     S3_ACCESS_KEY_ID     |             AWS S3 key ID             |
|        S3_BUCKET         |          AWS S3 bucket name           |
|       S3_ENDPOINT        |        AWS S3 source endpoint         |
|        S3_REGION         |         AWS S3 source region          |
|        S3_SECRET         |             AWS S3 secret             |
|       ETH_PROVIDER       |         Alchemy rpc http url          |

🪚 install dependencies:

```bash
# Install dependencies
yarn install
```

## 📝 Branch Management

Use [Git](https://git-scm.com/) as the version control system, create `dev` branches and `main` branches. Developers work on `dev` branches and initiate merge requests to the main branch after completing tasks.

## 🏗️ Building

```bash
# Run the apps
docker:dev
```

Browse to [http://localhost:3000](http://localhost:3000) to view the app, [http://localhost:3100](http://localhost:3100) to view the documentation.
