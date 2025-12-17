# Wired Data Store

## What

Personal data server storing [records](./records.md) and [blobs](./blobs.md).
Runs local (client-embedded) or remote (server-hosted).

## Why

- **Sovereignty**: Full control over storage and access
- **Offline-First**: Local WDS enables immediate offline editing
- **Sync**: Peer-to-peer synchronization via Loro CRDTs
- **Caching**: Local instances cache remote data

## Structure

```
WDS
├── Records (Loro CRDT documents)
└── Blobs (immutable binary, CID-addressed)
```

## Pinning

- **Pinned records**: Explicitly subscribed, always synced
- **Related records**: Auto-synced from pinned record relations
- **Garbage collection**: Unpinned, unreferenced records dropped

## Authentication

[DID](../social/did.md)-based operation signing. All read or synced data is
verified as coming from the author's DID.
