# Server

A modular UNAVI home server.

## Containers

Home server functionality is divided into separate features.

Each feature can be enabled or disabled, allowing for flexibility in deployment.
Note that some features rely on other features.

For example, I may want to run a lightweight home server that only handles user identity,
and disable the other features.

Or I may want to split up my service and run each feature on a separate machine.

### DB

The DB feature runs a MySQL database.

### IPFS

The IPFS feature runs an IPFS Kubo node for file storage and retrieval.

### Identity

The identity feature handles allows users to use the server as their home server.
It handles user authentication, and federates social interactions with other servers.

Depends on `DB` and `IPFS`.

### Web

The web feature hosts a web client at `/`.

Depends on `IPFS`.
Can also use `DB` for additional functionality.

### World

The world feature handles world networking over WebSockets and WebRTC.
It is the multiplayer server that connects users in a world together.

Depends on the `DB`.
