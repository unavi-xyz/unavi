# glTF Extensions

Contains [glTF-Transform](https://github.com/donmccurdy/glTF-Transform) implementations of the following glTF extensions:

- [KHR_audio](https://github.com/omigroup/gltf-extensions/tree/main/extensions/2.0/KHR_audio)
- [KHR_behavior](https://github.com/ux3d/glTF/tree/extensions/KHR_behavior/extensions/2.0/Khronos/KHR_behavior)
- [OMI_collider](https://github.com/omigroup/gltf-extensions/tree/main/extensions/2.0/OMI_collider)
- [OMI_spawn_point](https://github.com/omigroup/gltf-extensions/tree/main/extensions/2.0/OMI_spawn_point)
- WIRED_avatar
- WIRED_space

Also contains partial implementations of the following extensions:

- [VRMC_vrm](https://github.com/vrm-c/vrm-specification/tree/master/specification/VRMC_vrm-1.0)
- [VRM 0.0](https://github.com/vrm-c/vrm-specification/tree/master/specification/0.0)

## Installation

```bash
yarn install @wired-labs/gltf-extensions
```

## Usage

Simply register the extensions you want with your glTF-Transform `io` instance:

```typescript
import { NodeIO } from "@gltf-transform/core";
import { ColliderExtension, SpawnPointExtension } from "@wired-labs/gltf-extensions";

// Register extensions
const io = new NodeIO().registerExtensions([ColliderExtension, SpawnPointExtension]);

// Read from URL
const document = await io.read("path/to/model.glb");

// Write to byte array (Uint8Array)
const glb = await io.writeBinary(document);
```

For more information, see the [glTF-Transform documentation](https://gltf-transform.donmccurdy.com/).
