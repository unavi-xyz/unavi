# Server

UNAVI home server.

## Blocks

Home server functionality is split into different "blocks".
Each block can be enabled or disabled, allowing for flexibility in deployment.

For example, I may want to run a lightweight home server that only handles user identity,
and disable the World block.

Some blocks can be replaced by a remote URL.
For example, I may want to use a hosted DB service, and disable the DB block (setting the DB url to my provider).

Or I may want to run the World block on a separate machine for performance reasons.

### User

The user block handles user authentication and federation.

### World

The world block handles world networking over WebSockets and WebRTC.
It is the multiplayer server that connects users in a world together.

### DB

The DB block runs a MySQL database.

### IPFS

The IPFS block runs an IPFS Kubo node for file storage and retrieval.
