# Blobs

## What

Immutable, content-addressed binary data stored in [WDS](./wds.md). Identified
by CID (Content Identifier) - hash of contents.

## Why

- **Content Addressing**: CID guarantees integrity
- **Immutability**: Content never changes
- **Deduplication**: Same content stored once
- **Decentralization**: Fetch from any WDS

## CID Format

IPFS-style Content Identifiers: base32, CIDv1, SHA-256

Example: `bafyreiabc123...`

## Usage

Binary assets referenced by [HSD](../spatial/hsd.md):

- 3D models (glTF/GLB)
- Textures (PNG, JPEG, KTX2)
- Audio (MP3, OGG, WAV)
- Scripts (WASM components)
- Avatars (VRM)

## Caching

Local WDS caches blobs from pinned [records](./records.md). Garbage collected
when no pinned record references them.

## Cross-WDS References

Default: fetch from same WDS as referencing record.

For cross-WDS, include origin hint:

```json
{
  "mesh": {
    "cid": "bafyrei-...",
    "origin": "did:web:bob.com"
  }
}
```

Client resolves [DID](../social/did.md) to find WDS endpoint.
