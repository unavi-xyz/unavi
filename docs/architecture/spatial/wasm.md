# WebAssembly

## What

Scripts run in sandboxed WASM environments using Component Model. Portable,
composable units with typed interfaces (WIT).

[Component Model Specification](https://github.com/WebAssembly/component-model)

## Why

- **Sandboxing**: Isolated, secure execution
- **Portability**: Write once, run anywhere
- **Performance**: Near-native speed
- **Determinism**: Reproducible for networked environments

## The Wired Protocol

Scripts interact via WIT interfaces in `/protocol/wit/`:

**wired:math** - Vectors, quaternions, transforms

**wired:scene** - Scene graph, nodes, meshes, hierarchy

**wired:ecs** - Entity-component-system:

- Entities: Unique IDs
- Components: Data attached to entities
- Systems: Query and mutate components
- Schedules: `startup`, `update`, `render`

## Guest/Host Model

**Host** (`unavi-script`): Rust runtime using wasmtime. Implements WIT
interfaces, provides ECS access.

**Guest**: WASM modules in `/wasm/`, compiled to `wasm32-wasip2`. Define
systems, spawn entities, add components.

## Storage

Scripts stored as [blobs](../data/blobs.md) in [WDS](../data/wds.md):

1. Compile to `wasm32-wasip2`
1. Optimize with `wasm-opt -O4`
1. Convert to component with `wasm-tools`
1. Upload to WDS as blob → receive CID
1. Reference CID in [HSD](./hsd.md)

## Loading

[Spaces](./spaces.md) reference scripts via CID:

```json
{
  "scripts": [
    "bafyrei-physics-wasm...",
    "bafyrei-interaction-wasm..."
  ]
}
```

Flow:

1. Client loads space [record](../data/records.md)
1. Parse HSD from `data` field
1. Fetch script blobs by CID from WDS
1. Instantiate WASM components via wasmtime
1. Scripts execute in sandbox
1. Interact with ECS via WIT interfaces

## Security

- **Sandboxing**: Memory isolation, no arbitrary memory access
- **Capabilities**: Only ECS operations via WIT interfaces
- **Permissions**: Space owner controls scripts. Inherit space permission
  context
