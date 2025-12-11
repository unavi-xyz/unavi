# Wired Data Store

## What

Personal data server storing [records](./records.md) and [blobs](./blobs.md).
Runs local (client-embedded) or remote (server-hosted). User-operated for data
sovereignty.

## Why

- **Sovereignty**: Full control over storage and access
- **Offline-First**: Local WDS enables offline editing
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

## REST API

```
POST   /records         Create record
GET    /records/{id}    Get record
DELETE /records/{id}    Delete record

POST   /blobs           Upload blob
GET    /blobs/{cid}     Get blob

CONNECT /records/{id}/sync   Real-time sync (WebSocket/QUIC)
```

## Authentication

[DID](../social/did.md)-based challenge-response. Client signs nonce with DID
key, server verifies via DID document public key.
