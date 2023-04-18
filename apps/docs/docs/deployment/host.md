---
sidebar_label: ☁️ Host
title: Host
---

The host server adds multiplayer functionality to spaces. It is a Node.js server, and is available as a Docker image for easy deployment.

## Requirements

- Server with at least 1GB of RAM
- SSL certificate (refer to [CertBot](https://certbot.eff.org/) or [Let's Encrypt](https://letsencrypt.org/))

## Deployment

### Step 1 - Create deploy script

Create following script that will download and run the latest host image. Make sure to update `MEDIASOUP_ANNOUNCED_IP` with your public IP, `SSL_CERT` with your SSL certificate, and `SSL_KEY` with your SSL key.

```bash title="/home/deploy.sh"
# Stop and remove any existing containers
docker stop host
docker rm host
docker rmi ghcr.io/unavi-xyz/host

# Pull the latest image
docker pull ghcr.io/unavi-xyz/host:latest

# Set environment variables
export MEDIASOUP_ANNOUNCED_IP="123.45.67.89" # Your public IP
export SSL_CERT="cert.pem" # Your SSL certificate
export SSL_KEY="key.pem" # Your SSL key
export RTC_MIN_PORT=20000
export RTC_MAX_PORT=20020

# Run the container
docker run -d --name host \
  --restart unless-stopped \
  -e MEDIASOUP_ANNOUNCED_IP \
  -e RTC_MAX_PORT \
  -e RTC_MIN_PORT \
  -e SSL_CERT \
  -e SSL_KEY \
  -v /home/$SSL_CERT:/app/cert.pem \
  -v /home/$SSL_KEY:/app/key.pem \
  -p 443:4000 \
  -p $RTC_MIN_PORT-$RTC_MAX_PORT:$RTC_MIN_PORT-$RTC_MAX_PORT/tcp \
  -p $RTC_MIN_PORT-$RTC_MAX_PORT:$RTC_MIN_PORT-$RTC_MAX_PORT/udp \
  -p $RTC_MIN_PORT-$RTC_MAX_PORT:$RTC_MIN_PORT-$RTC_MAX_PORT/sctp \
  ghcr.io/unavi-xyz/host
```

### Step 2 - Run deploy script

Now just run the script to start the host.

```bash
sh /home/deploy.sh
```

## Reverse proxy

If you want to run the host behind a reverse proxy, there are a few things to configure.

### Ports

By default, the host listens to port `443` for WebSocket and HTTP connections, and ports `20000-20020` for WebRTC connections. Make sure you have the following ports exposed.

- `443/tcp`
- `20000-20020/tcp`
- `20000-20020/udp`
