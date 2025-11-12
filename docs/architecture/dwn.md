# Decentralized Web Nodes

## What

A **Decentralized Web Node** (DWN) is a personal data server for storing and
sharing data associated with a [DID](./did.md). Users operate their own
infrastructure instead of relying on centralized platforms.

[DWN Specification](https://identity.foundation/decentralized-web-node/spec/)

## Why

- **Sovereignty**: Full control over data, access, and replication.
- **Interoperability**: Standards-based protocols enable cross-platform data.
- **Resilience**: Multi-node sync across devices and locations.
- **Portability**: Data moves with the user, not tied to providers.

## In The Wired

DWNs store nearly all data in The Wired and facilitate interactions between
agents. This includes spatial data ([spaces](./spaces.md)), user metadata,
access control, and coordination for operations like server hosting.

DWN endpoints are declared in DID Documents, enabling discovery.
Queries support advanced filtering and sorting, such as listing public spaces by
arbitrary criteria (tags, creation date, player count).

## Key Concepts

- **Records**: Schema-based storage with versioning and hierarchical relationships.
- **Protocols**: Declarative rules governing data types, constraints, and operations.
- **Sync**: Multiple DWN instances maintain consistent state across the mesh.
