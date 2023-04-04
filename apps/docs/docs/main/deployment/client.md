---
sidebar_label: 💻 Client
title: Client
---

The client is the web app that users interact with. It is comprised of a Next.js website, a MySQL database, and an S3-compatible storage bucket.

## Local instance

The client can be run entirely locally on your own machine, giving you full control over your experience. This is ideal if you only want to run the client for yourself, and not share it with others.

To run the client locally, refer to the [development guide](/development), and run the docker build in `production` mode. This will start the client, database, and storage bucket on your machine in Docker containers.

:::info

Even when running locally, the client is fully functional! You will still be able to join spaces and interact with other users.

:::

## Deployment

The client can also be deployed to a server, allowing you to share it with others. This is ideal if you want to run a public instance of the client.

:::tip

Check out the [official Next.js docs](https://nextjs.org/docs/deployment) for more information on deploying Next.js apps.

:::

### Requirements

- Hosting provider (such as [AWS EC2](https://aws.amazon.com/ec2/) or [Vercel](https://vercel.com/))
- MySQL database provider (such as [Planetscale](https://planetscale.com/) or [AWS RDS](https://aws.amazon.com/rds/))
- [AWS S3](https://aws.amazon.com/tw/s3/) or S3-compatible storage (such as [DigitalOcean Spaces](https://www.digitalocean.com/products/spaces))

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

Within your **hosting provider**, the following environment variables need to be set for the Next.js app to run:

| Variable                 | Description                                                              |
| ------------------------ | ------------------------------------------------------------------------ |
| NEXT_PUBLIC_CDN_ENDPOINT | CDN endpoint for S3                                                      |
| NEXT_PUBLIC_DEFAULT_HOST | Host app endpoint                                                        |
| DATABASE_URL             | Database connection URI                                                  |
| NEXTAUTH_SECRET          | [NextAuth secret](https://next-auth.js.org/configuration/options#secret) |
| NEXTAUTH_URL             | Client app endpoint                                                      |
| S3_ACCESS_KEY_ID         | AWS S3 key ID                                                            |
| S3_BUCKET                | AWS S3 bucket name                                                       |
| S3_ENDPOINT              | AWS S3 source endpoint                                                   |
| S3_REGION                | AWS S3 source region                                                     |
| S3_SECRET                | AWS S3 secret                                                            |
| ETH_PROVIDER             | Ethereum rpc http URI                                                    |

Additionally, the following environment variables need to be set on **both your local machine and your hosting provider**:

| Variable            | Description                 |
| ------------------- | --------------------------- |
| MYSQL_ROOT_HOST     | Database root for container |
| MYSQL_DATABASE      | Database name               |
| MYSQL_USER          | Database username           |
| MYSQL_PASSWORD      | Database user password      |
| MYSQL_ROOT_PASSWORD | Database root user password |

### Step 4 - Initialize database

Run the following command to create the database tables:

```bash
yarn prisma db push
```

### Step 5 - Update CORS

Adjust your S3 CORS configuration to allow communication from your domain (check out [AWS S3 CORS](https://docs.aws.amazon.com/AmazonS3/latest/userguide/add-cors-configuration.html) or [DigitalOcean Spaces CORS](https://docs.digitalocean.com/products/spaces/how-to/configure-cors/)).
