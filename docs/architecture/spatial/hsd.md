# HyperSpace Documents

## What

CRDT-based content format for spatial hypermedia, analogous to HTML for 2D web. Defines
3D geometry, entity composition, and asset references. Can live as a file, or
in a [record's](../data/records.md) `data` field.

## Why

- **Syncable**: Designed to be easily networked with multiple writers
- **Addressable**: Reference via CIDs and [DID](../social/did.md) URLs
- **Composable**: Nest and link documents fractally

## Analogy

```
HTML → 2D web documents → URLs
HSD  → 3D web documents → DID URLs
```

## Structure

HSD content stored in record `data` field per spatial
[schema](../data/schemas.md):

```json
{
  "name": "My Space",
  "geometry": "bafyrei-geometry-cid...",
  "spawn": {
    "position": [0, 1.6, 0],
    "rotation": [0, 0, 0, 1]
  },
  "environment": {
    "sky": "bafyrei-skybox-cid...",
    "ambient": [0.2, 0.2, 0.3]
  }
}
```

## Blob References

Reference binary data via [CID](../data/blobs.md):

```json
{
  "mesh": "bafyrei-mesh-cid...",
  "texture": "bafyrei-texture-cid..."
}
```
