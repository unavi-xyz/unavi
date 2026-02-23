# WebAssembly

## What

Scripts run in sandboxed WASM environments using Component Model.
Portable, composable units with typed interfaces (WIT).

[Component Model Specification](https://github.com/WebAssembly/component-model)

## Why

- **Sandboxing**: Isolated, secure execution
- **Portability**: Write once, run anywhere
- **Performance**: Near-native speed
- **Determinism**: Reproducible for networked environments

## The Wired Protocol

## Guest/Host Model

**Host** (`unavi-script`): Rust runtime using wasmtime. Implements
WIT interfaces, translates resource operations to Loro CRDT mutations
on the backing [HSD](./hsd.md) document.

**Guest**: WASM modules in `/wasm/`, compiled to `wasm32-wasip2`.
Create nodes, set transforms, attach meshes and materials, respond to
physics — all through typed resource handles.

## Script Context

Each script is attached to a node within an HSD document. The host
provides ambient context via `wired:scene/context`:

- `self-node()` — the node this script is attached to
- `self-document()` — the HSD document containing that node

Scripts interact with their own document. The host handles
inter-document physics and rendering within the [Space](./spaces.md).

## Storage

Scripts stored as [blobs](../data/blobs.md) in [WDS](../data/wds.md):

## Loading

[Spaces](./spaces.md) load HSD documents, which reference scripts:

## Security

- **Sandboxing**: Memory isolation, no arbitrary memory access
- **Capabilities**: Only scene operations via WIT interfaces
- **Permissions**: Space owner controls which HSDs (and their
  scripts) are loaded. Scripts inherit their HSD's permission context
