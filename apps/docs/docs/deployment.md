---
sidebar_position: 50
sidebar_label: ☁️ Deployment
title: Deployment
---

:::tip

Read [official Next.js docs](https://nextjs.org/docs/deployment) to handle deployment smoothly.

:::

Please refer to our [development docs](/development) first. then update [./apps/client/.env.local](/development#step-3---update-environment-variables) for production. please make sure you are not commit them to the repository.

## Requirements

- Hosting provider (such as [AWS EC2](https://aws.amazon.com/ec2/) or [Vercel](https://vercel.com/))
- SSL certificate (refer to [CertBot](https://certbot.eff.org/) or [Let's Encrypt](https://letsencrypt.org/))
- [AWS S3](https://aws.amazon.com/tw/s3/) or S3-compatible storage (such as [DigitalOcean Spaces](https://www.digitalocean.com/products/spaces))

## Hosting

### Step 1 - Update security inbound

The wired using webRTC to handle connection, default is 20000~20020 port, you can adjust the env `RTC_MIN_PORT` and `RTC_MAX_PORT`, make sure you have expose the following ports:

- TCP 20000-20010
- UDP 20000-20010

### Step 2 - Update cors

Adjust CORS configuration to allow fetch to object storage from your domain. please refer to [AWS S3 CORS](https://docs.aws.amazon.com/AmazonS3/latest/userguide/add-cors-configuration.html) or [DigitalOcean Spaces CORS](https://docs.digitalocean.com/products/spaces/how-to/configure-cors/).

### Step 3 - Run database and client app

You can simply run following command to start the wired app:

```bash
yarn docker:prod
```

### Step 4 - Run host app

Run following script to start the host app, please make sure you have update the `MEDIASOUP_ANNOUNCED_IP` and `SSL_CERT` and `SSL_KEY` path:

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

### Step 5 - Proxy pass

Now, you have client app listen on port `3000` and host app listen on port `4000`, you can use nginx to proxy pass to the app.

listen path `NEXTAUTH_URL` then pass to your client app, and listen `NEXT_PUBLIC_DEFAULT_HOST` then pass to your host app.

:::info
If you are facing any issues, please join our [Discord](https://discord.gg/cazUfCCgHJ) to get help.
:::
