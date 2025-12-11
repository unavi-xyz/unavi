# Spaces

## What

Bounded 3D region containing geometry, entities, and interactive elements.
Hyperdocuments in The Wired's spatial hypermedia system — addressable,
linkable, and composable.

## Storage

[Records](../data/records.md) in [WDS](../data/wds.md), referenced by DID URLs.
Follows Space [schema](../data/schemas.md) (`wired-protocol.org/schemas/space`).

## Hypermedia Controls

**Slots (Composition)**: AABB regions for child space insertion. Static or
dynamic. Parent can query child entities marked public.

**Portals (Linking)**: Seamless windows into other spaces. Agents walk through,
carry objects. Function as hyperlinks in hyperspace, address by DID URL.

## HSD Integration

Uses [HSD](./hsd.md) as content format:

```
Record
├── genesis: { creator: "did:web:alice.com", schema: "wired-protocol.org/schemas/space" }
├── permissions: { owner: ["did:web:alice.com"], ... }
├── context: null
├── relations: { "object": [...] }
└── data: <HSD>
    ├── name: "Alice's Gallery"
    ├── geometry: "bafyrei-geometry-blob..."
    ├── spawn: { position: [0, 1.6, 0], rotation: [0, 0, 0, 1] }
    ├── environment: { sky: "bafyrei-skybox...", ambient: [0.2, 0.2, 0.3] }
    └── scripts: ["bafyrei-script-wasm..."]
```
