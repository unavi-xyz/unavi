# HyperSpace Document (HSD)

## What

CRDT-based scene format for spatial hypermedia. Defines 3D scenes with
hierarchical nodes and non-destructive layers. Stored in a
[record's](../data/records.md) Loro document.

## Why

- **Collaborative**: Multi-writer editing via CRDTs
- **Non-Destructive**: Layer system for reversible edits
- **Extensible**: Open attribute system on nodes

## Inspiration

Draws from [USD](https://en.wikipedia.org/wiki/Universal_Scene_Description)'s
composition model: layers contain opinions that override base scene data.

## Structure

```
HSD Root
├── layers (MovableList)
│   └── Layer { id, opinions }
└── nodes (Tree)
    └── Node { id, attributes }
```

## Nodes

Hierarchical tree of scene entities. Each node has an `id` and an `attributes`
map for extensible data (transforms, meshes, materials, etc).

Stored as a Loro Tree for parent-child relationships and efficient subtree
operations.

## Layers

Ordered list of non-destructive edits. Each layer contains opinions that can
delete or update node attributes. Later layers override earlier ones.

Use cases: animation, user overrides, temporary edits, variants.

## Attributes

Node attributes are schema-free (`Map(Any)`), allowing arbitrary data. Standard
attribute conventions (transforms, meshes, materials) can be defined separately.

Heavy data like geometry and textures should reference [blobs](../data/blobs.md)
by hash rather than inline data.
