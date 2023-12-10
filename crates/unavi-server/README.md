# unavi-server

A modular UNAVI server.

## Architecture

Server functionality is organized into separate features.
Each feature can be enabled or disabled, allowing for flexibility in deployment.

For example, I may want to run a lightweight server that only handles social functionality, and disable the other features.
Or I may want to split up my server and run worlds on a separate machine.

## Features

### DB

MySQL database.

### IPFS

IPFS Kubo node, used for file storage and retrieval.

### Social 

Manages a user's identity, federates social interactions with other servers.

### Web

Hosts a web client.

### World

Allows worlds to use the server as their world server.
Handles networking, connecting players within a world together.
