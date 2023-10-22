# Server

A modular UNAVI home server.

## Architecture

Home server functionality is divided into separate containers.
Each container can be enabled or disabled, allowing for flexibility in deployment.
Note that some containers rely on other containers.

For example, I may want to run a lightweight home server that only handles user identity,
and disable the other containers.
Or I may want to split up my service and run each container on a separate machine.

<div align="center">
  <img src="../assets/server-architecture.png" height="600" />
</div>

## Containers

### DB

The DB container runs a MySQL database.

### IPFS

The IPFS container runs an IPFS Kubo node for file storage and retrieval.

### Identity

The identity container handles allows users to use the server as their home server.
It handles user authentication, and federates social interactions with other servers.

### Router

The router is the entrypoint into the service.
It recieves external requests and delegates them to other containers.

### Web

The web container hosts a web client.

### World

The world container handles world networking over WebSockets and WebRTC.
It is the multiplayer server that connects users in a world together.
