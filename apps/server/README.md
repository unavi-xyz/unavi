# Server

A server used to host spaces.

Acts as a signaling server, connecting peers using WebSockets, where they can then communicate with each other directly using WebRTC.

## Tech Stack

- WebSocket communication using a [Socket.io](https://github.com/socketio/socket.io) / [Nodejs](https://nodejs.org/en/) server
- [Nginx](https://www.nginx.com/) reverse proxy
- TLS certificate from [Let's Encrypt](https://letsencrypt.org/) using [Certbot](https://github.com/certbot/certbot)

## Running a Server

To run your own server, first install [Docker](https://www.docker.com/get-started/).

Set your domain name and email in the `.env` file.

Open `nginx/nginx.conf`

Delete the second `server` block

Run docker compose

```bash
docker compose --env-file .env up
```

This should create the SSL certificate. After it has done that, shut it down with

```bash
docker compose down
```

Purge docker

```bash
docker purge -a -f
```

Add the second server block back to `nginx/nginx.conf`.

Finally, start the server with

```bash
docker compose --env-file .env up
```
