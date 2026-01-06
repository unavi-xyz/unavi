# Wired Data Store

## What

Personal data server storing [records](./records.md) and [blobs](./blobs.md).
Built on [DIDs](../social/did.md) for identity and [iroh](https://iroh.computer)
for networking. Runs local (client-embedded) or remote (server-hosted).

## Why

- **Sovereignty**: Full control over storage and access
- **Offline-First**: Local WDS enables immediate offline editing
- **Sync**: Peer-to-peer synchronization via Loro CRDTs
- **Caching**: Local instances cache remote data

## Structure

```
WDS
├── Records (Loro CRDT documents)
│   └── Envelopes (signed incremental updates)
└── Blobs (immutable, blake3-addressed)
```

## Protocols

WDS uses QUIC-based protocols via iroh:

- `wds/auth` - Challenge-response DID authentication
- `wds/api` - Record and blob operations
- `wds/sync` - Cross-store record synchronization

## Authentication

DIDs can specify WDSes in their DID document as a `wds` service. This enables:

- **Discovery**: Resolve a DID to find its WDS endpoint
- **Delegation**: WDS can authenticate on behalf of the DID owner for syncing

Authentication uses challenge-response: client signs a server-provided nonce
with their DID key. For cross-WDS sync, the server checks if the requesting
WDS is listed in the DID's service endpoints.

## Sync

Records sync between WDS instances using envelope exchange:

- Each envelope contains signed CRDT operations with version vectors
- Stores exchange envelopes they're missing (bidirectional)
- Version vectors track what each store has seen

## Pinning

- **Pinned records**: Explicitly subscribed, always kept
- **TTL**: Pins expire after a set time
- **Garbage collection**: Unpinned, unreferenced data dropped

## Quotas

Per-user storage limits. Data must be pinned by at least one
user with available quota to be stored.
