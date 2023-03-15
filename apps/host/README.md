# Host

A dockerized Node.js server for hosting spaces over WebSockets and WebRTC.

## Deployment

Here's an example of how to deploy the host server using docker. This example assumes you have an SSL certificate and key in the home directory, named `cert.pem` and `key.pem` respectively.

```bash
# /home/deploy.sh

# Stop and remove any existing containers
docker stop host
docker rm host
docker rmi ghcr.io/wired-labs/host

# Pull the latest image
docker pull ghcr.io/wired-labs/host:latest

# Set environment variables
export MEDIASOUP_ANNOUNCED_IP="123.45.67.890" # Your public IP
export RTC_MIN_PORT=20000
export RTC_MAX_PORT=20040

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
-p 443:4000 \
-p $RTC_MIN_PORT-$RTC_MAX_PORT:$RTC_MIN_PORT-$RTC_MAX_PORT/tcp \
-p $RTC_MIN_PORT-$RTC_MAX_PORT:$RTC_MIN_PORT-$RTC_MAX_PORT/udp \
-p $RTC_MIN_PORT-$RTC_MAX_PORT:$RTC_MIN_PORT-$RTC_MAX_PORT/sctp \
ghcr.io/wired-labs/host
```
