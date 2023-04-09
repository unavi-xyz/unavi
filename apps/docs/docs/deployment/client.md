---
sidebar_label: 💻 Client
title: Client
---

The client is the web app that users interact with. It is comprised of a Next.js website, a MySQL database, and an S3-compatible storage bucket.

## Deployment

:::tip

Check out the [official Next.js docs](https://nextjs.org/docs/deployment) for more information on deploying Next.js apps.

:::

### Requirements

- Hosting provider (such as [AWS EC2](https://aws.amazon.com/ec2/) or [Vercel](https://vercel.com/))
- (optional) MySQL database provider (such as [AWS RDS](https://aws.amazon.com/rds/) or [Planetscale](https://planetscale.com/))
- (optional) S3-compatible storage (such as [AWS S3](https://aws.amazon.com/tw/s3/) or [DigitalOcean Spaces](https://www.digitalocean.com/products/spaces))

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
| NEXT_PUBLIC_DEFAULT_HOST | Host app endpoint                                                        |
| NEXTAUTH_SECRET          | [NextAuth secret](https://next-auth.js.org/configuration/options#secret) |
| NEXTAUTH_URL             | Client app endpoint                                                      |
| ETH_PROVIDER             | Ethereum rpc http URI                                                    |

If using S3, the following environment variables need to be set within your **hosting provider**:

| Variable           | Description                       |
| ------------------ | --------------------------------- |
| NEXT_PUBLIC_HAS_S3 | Set to `true` if you are using S3 |
| S3_ACCESS_KEY_ID   | AWS S3 key ID                     |
| S3_BUCKET          | AWS S3 bucket name                |
| S3_ENDPOINT        | AWS S3 source endpoint            |
| S3_REGION          | AWS S3 source region              |
| S3_SECRET          | AWS S3 secret                     |

If using a database, the following environment variables need to be set on **both your local machine and your hosting provider**:

| Variable                 | Description                               |
| ------------------------ | ----------------------------------------- |
| DATABASE_URL             | Database connection URI                   |
| MYSQL_ROOT_HOST          | Database root for container               |
| MYSQL_DATABASE           | Database name                             |
| MYSQL_USER               | Database username                         |
| MYSQL_PASSWORD           | Database user password                    |
| MYSQL_ROOT_PASSWORD      | Database root user password               |
| NEXT_PUBLIC_HAS_DATABASE | Set to `true` if you are using a database |
| NEXT_PUBLIC_CDN_ENDPOINT | CDN endpoint for S3                       |

### Step 4 - Initialize database

Run the following command to create the database tables:

```bash
yarn prisma db push
```

### Step 5 - Update CORS

Adjust your S3 CORS configuration to allow communication from your domain (check out [AWS S3 CORS](https://docs.aws.amazon.com/AmazonS3/latest/userguide/add-cors-configuration.html) or [DigitalOcean Spaces CORS](https://docs.digitalocean.com/products/spaces/how-to/configure-cors/)).
