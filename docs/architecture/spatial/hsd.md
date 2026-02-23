# HyperSpace Document (HSD)

## What

The atomic unit of spatial content in The Wired. An HSD defines a
3D object as a CRDT document: a tree of nodes with shared meshes,
materials, physics, and embedded scripts. Stored as a
[record](../data/records.md) in [WDS](../data/wds.md).

HSD is to The Wired what HTML is to the Web — a self-contained
hypermedia document, but spatial.

## Why

- **Autopoietic**: Each HSD carries its own structure, appearance,
  and behavior. Objects define and maintain themselves.
- **Collaborative**: Multi-writer editing via Loro CRDTs.
- **Portable**: An HSD can move between [Spaces](./spaces.md)
  because it is self-contained with its own identity and ownership.
- **Sovereign**: Each HSD is its own record with its own DID and ACL.

## Inspiration

Draws from [glTF](https://www.khronos.org/gltf/) as a runtime
delivery format: structured, typed, and optimized for rendering.
Heavy data (geometry, textures) stored as [blobs](../data/blobs.md)
rather than inline.

## Structure

```
Document
├── materials (List)
│   └── Material { name, base_color, metallic, roughness, ... }
├── meshes (List)
│   └── Mesh { name, topology, indices, attributes }
└── nodes (Tree)
    └── Node { name, transform, mesh, material, collider, ... }
```

## Nodes

Hierarchical tree of scene elements. Each node has typed fields:

- **name**: optional label for debugging and authoring
- **translation, rotation, scale**: local transform
- **mesh**: reference to a mesh by index
- **material**: reference to a material by index
- **collider**: physics collision shape (cuboid, sphere, capsule)
- **rigid_body**: physics body type (dynamic, fixed, kinematic)
- **scripts**: list of [WASM](./wasm.md) blob references

Stored as a Loro Tree for parent-child relationships and efficient
subtree operations.

## Materials

PBR material definitions shared across nodes. Properties follow the
metallic-roughness model: base color, metallic factor, roughness
factor, and associated textures. Textures reference blobs by hash.

## Meshes

Geometry definitions shared across nodes. Each mesh specifies a
primitive topology and references vertex attribute data (positions,
normals, tangents, colors, UVs) and indices as blobs.

## Schema

Defined in `protocol/schemas/hsd/document.ron`.
