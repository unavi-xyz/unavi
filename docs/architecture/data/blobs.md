# Blobs

## What

Immutable, content-addressed binary data stored in [WDS](./wds.md). Identified
by blake3 hash of contents.

## Why

- **Content Addressing**: Hash guarantees integrity
- **Immutability**: Content never changes
- **Deduplication**: Same content stored once
- **Decentralization**: Fetch from any WDS

## Usage

Binary assets referenced by [records](./records.md):

- 3D models (glTF/GLB)
- Textures (PNG, JPEG, KTX2)
- Audio (MP3, OGG, WAV)
- Scripts (WASM components)
- Avatars (VRM)
- [Schemas](./schemas.md)

## Cross-WDS References

Default: fetch from same WDS as referencing record.

For cross-WDS, include origin hint:

```ron
(
    mesh: (
        hash: "a1b2c3d4e5f6...",
        origin: "did:web:bob.example",
    ),
)
```

Client resolves [DID](../social/did.md) to find WDS endpoint.
