# unavi-server

A self-hostable, modular server for UNAVI.

Server functionality is organized by feature.
Each feature can be enabled or disabled, allowing for flexibility in deployment.

For example, I may want to run a lightweight server that only hosts my identity, and disable the other features.
Or I may want to split up my server and run worlds on a separate machine.

## Features

### Social 

Manages a user's identity, federating social interactions with other servers.

### Web

Hosts a web client.

### World

Allows worlds to use the server as their world server.
Handles networking, connecting players within a world together.
