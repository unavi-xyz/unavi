# WebAssembly

## Component Model

The **WASM Component Model** defines portable, composable units with typed
interfaces. Unlike core WASM modules, components have:

- **Rich types**: Records, variants, resources (not just numbers).
- **Interface types**: Defined via [WIT](https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md)
  (WebAssembly Interface Types).
- **Composition**: Components link to other components via imports/exports.

[Component Model Specification](https://github.com/WebAssembly/component-model)

## Why

- **Sandboxing**: Scripts run in isolated, secure environments.
- **Portability**: Write once, run anywhere (any platform, any language).
- **Performance**: Near-native execution speed.
- **Determinism**: Reproducible computation for networked environments.

## The Wired Protocol

Scripts interact with [spaces](./spaces.md) via WIT-defined interfaces in
`/protocol/wit/`. Such as:

### wired:math

Math primitives: vectors, quaternions, transforms.

### wired:scene

Scene graph: nodes, meshes, transforms, hierarchy.

### wired:ecs

Entity Component System for scripts:

- **Entities**: Unique IDs for objects.
- **Components**: Data attached to entities.
- **Systems**: Functions that query and mutate component data.
- **Schedules**: `startup`, `update`, `render`.

## Guest/Host Model

- **Host** (`unavi-script`): Rust runtime using [wasmtime](https://wasmtime.dev/).
  Implements WIT interfaces, providing ECS access to guest scripts.
- **Guest**: WASM modules in `/wasm/` compiled to `wasm32-wasip2`. Scripts
  define systems, spawn entities, add components.

Scripts are stored within glXF scene files, such as those used by
[spaces](./spaces.md). The WASM is loaded and executed within the 3D runtime.
