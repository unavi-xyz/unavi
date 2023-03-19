# glTf Extensions

Contains [glTF-Transform](https://github.com/donmccurdy/glTF-Transform) implementations of the following glTF extensions:

- [KHR_behavior](src/Behavior/)
- [OMI_collider](src/Collider/)
- [OMI_spawn_point](src/SpawnPoint/)

## Installation

```bash
yarn install @wired-labs/gltf-extensions
```

## Usage

Simply register the extensions with your glTF-Transform `io` instance:

```typescript
import { NodeIO } from "@gltf-transform/core";
import {
  BehaviorExtension,
  ColliderExtension,
  SpawnPointExtension,
} from "@wired-labs/gltf-extensions";

// Register extensions
const io = new NodeIO().registerExtensions([
  BehaviorExtension,
  ColliderExtension,
  SpawnPointExtension,
]);

// Read from URL
const document = await io.read("path/to/model.glb");

// Write to byte array (Uint8Array)
const glb = await io.writeBinary(document);
```

For more information, see the [glTF-Transform documentation](https://gltf-transform.donmccurdy.com/).
