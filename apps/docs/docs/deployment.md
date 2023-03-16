---
sidebar_position: 50
sidebar_label: ☁️ Deployment
title: Deployment
---

:::tip

Read [official Next.js docs](https://nextjs.org/docs/deployment) to handle deployment smoothly.

:::

## Requirements

- Hosting provider (such as [AWS EC2](https://aws.amazon.com/ec2/) or [Vercel](https://vercel.com/))
- SSL certificate (refer to [CertBot](https://certbot.eff.org/) or [Let's Encrypt](https://letsencrypt.org/))
- [AWS S3](https://aws.amazon.com/tw/s3/) or S3-compatible storage (such as [DigitalOcean Spaces](https://www.digitalocean.com/products/spaces))

## Hosting

### Step 1 - Expose ports

The Wired uses WebRTC on ports 20000-20020 by default, you can adjust this by setting the `RTC_MIN_PORT` and `RTC_MAX_PORT` enviornment variables. Make sure you have the following ports exposed:

- TCP 20000-20020
- UDP 20000-20020

### Step 2 - Update CORS

Adjust your CORS configuration to allow communication with your S3 provider from your domain (check out [AWS S3 CORS](https://docs.aws.amazon.com/AmazonS3/latest/userguide/add-cors-configuration.html) or [DigitalOcean Spaces CORS](https://docs.digitalocean.com/products/spaces/how-to/configure-cors/)).

### Step 3 - Update environment variables

:::warning

Keep your secrets safe, do not commit them to the repository. You can copy the `.env` file to `.env.local` and update the variables there.

```bash
# Create .env.local
cp apps/client/.env apps/client/.env.local
```

:::

The following environment variables need to be set for the client:

| Variable                 | Description                                                              |
| ------------------------ | ------------------------------------------------------------------------ |
| MYSQL_ROOT_HOST          | Database root for container                                              |
| MYSQL_DATABASE           | Database name                                                            |
| MYSQL_USER               | Database username                                                        |
| MYSQL_PASSWORD           | Database user password                                                   |
| MYSQL_ROOT_PASSWORD      | Database root user password                                              |
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

### Step 4 - Run database and client

Run following command to start the apps in production mode:

```bash
yarn docker:prod
```

### Step 5 - Run host

Run following script to start the host. Make sure to update `MEDIASOUP_ANNOUNCED_IP`, `SSL_CERT`, and `SSL_KEY`:

```bash
# /home/deploy.sh

# Stop and remove any existing containers
docker stop host
docker rm host
docker rmi ghcr.io/wired-labs/host

# Pull the latest image
docker pull ghcr.io/wired-labs/host:latest

# Set environment variables
export MEDIASOUP_ANNOUNCED_IP="123.45.67.89" # Your public IP
export RTC_MIN_PORT=20000
export RTC_MAX_PORT=20020

# Run the container
docker run -d --name host \
  --restart unless-stopped \
  -e SSL_CERT="cert.pem" \
  -e SSL_KEY="key.pem" \
  -e MEDIASOUP_ANNOUNCED_IP \
  -e RTC_MIN_PORT \
  -e RTC_MAX_PORT \
  -v /home/cert.pem:/app/cert.pem \
  -v /home/key.pem:/app/key.pem \
  -p 4000:4000 \
  -p $RTC_MIN_PORT-$RTC_MAX_PORT:$RTC_MIN_PORT-$RTC_MAX_PORT/tcp \
  -p $RTC_MIN_PORT-$RTC_MAX_PORT:$RTC_MIN_PORT-$RTC_MAX_PORT/udp \
  -p $RTC_MIN_PORT-$RTC_MAX_PORT:$RTC_MIN_PORT-$RTC_MAX_PORT/sctp \
  ghcr.io/wired-labs/host
```

### Step 6 - Proxy pass

Currently the client is listening on port `3000`, and the host on port `4000`. You can use nginx to proxy pass to the apps.

Listen path `NEXTAUTH_URL` then pass to your client, and listen `NEXT_PUBLIC_DEFAULT_HOST` then pass to your host.

:::info
If you are facing any issues, please join our [Discord](https://discord.gg/cazUfCCgHJ) to get help.
:::
