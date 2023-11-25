# unavi-server

A modular UNAVI server.

## Architecture

Home server functionality is divided into separate services.
Each service can be enabled or disabled, allowing for flexibility in deployment.

For example, I may want to run a lightweight server that only handles user identity, and disable the other services.
Or I may want to split up my server and run worlds on a separate machine.

## Services

### DB

MySQL database.

### IPFS

IPFS Kubo node, used for file storage and retrieval.

### Identity

Handles user authentication, federates social interactions with other servers.

### Web

Hosts a web client.

### World

Allows worlds to use the server as their world server.
Handles networking, connecting players within a world together.
