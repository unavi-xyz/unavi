# Server

A nodejs server used to host spaces.

Acts as a signaling server, connecting peers using websockets (socket.io), where they can then communicate with each other directly using WebRTC.

## Docker

Running your own server is as simple as running the docker image.

### Build

Pull the image from the github container registry:

```bash
docker pull ghcr.io/wired-xr/server
```

or build it yourself from this repo:

```bash
yarn build:docker
```

### Run

```bash
docker run -it -p 8080:8080 ghcr.io/wired-xr/server
```
