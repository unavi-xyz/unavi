# unavi-server

A fully self-hostable, modular server for UNAVI.

## Features

Server functionality is organized by feature.
Each feature can be enabled or disabled, allowing for flexibility in deployment.

For example, I may want to run a lightweight server that only hosts my identity, and disable the other features.
Or I may want to split up my server and run worlds on a separate machine.

| Feature | Description |
| ------- | ----------- |
| social  | Manages a user's identity, federating social interactions with other servers. |
| web     | Hosts a web client. |
| world   | Handles world networking, connecting players within a world together. |
