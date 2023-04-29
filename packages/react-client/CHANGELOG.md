# @unavi/react-client

## 0.6.0

### Minor Changes

- 114862f0: upgrade to the latest version of the wired protocol
- bc2bdc71: move away from profile NFTs. use the new Wired Protocol identity system.

## 0.5.1

### Patch Changes

- Updated dependencies [fb0120a9]
  - @unavi/gltf-extensions@0.6.0
  - engine@0.2.1

## 0.5.0

### Minor Changes

- 120335e: Move from Space IDs to space URIs. Spaces are no longer required to be NFTs - a space is now just a glTF file. All you need to access a space is a URI pointing to it. Spaces can still be NFTs, but they can now be many more things as well.

### Patch Changes

- Updated dependencies [120335e]
  - @unavi/protocol@0.3.0

## 0.4.0

### Minor Changes

- 1b2e29e: add KHR_audio support

### Patch Changes

- b52c4eb: fix transports not closing immediately
- Updated dependencies [4073902]
- Updated dependencies [1b2e29e]
- Updated dependencies [1b2e29e]
- Updated dependencies [d569e98]
  - engine@0.2.0

## 0.3.0

### Minor Changes

- 6ae91ef: expose metadata and host to ClientContext

### Patch Changes

- 632bd65: fix initial player data not being set
- c75e6b9: wrap host url with WebSocket protocol (wss://) if not provided

## 0.2.0

### Minor Changes

- f5af4ee: add max distance to avatar equip
- 73c1b21: store player avatar in localstorage
- f46f168: add profile handle fetching using ethers, allowing the player's profile name to be displayed in the engine

### Patch Changes

- Updated dependencies [f5af4ee]
- Updated dependencies [51b061d]
  - engine@0.1.0

## 0.1.0

### Minor Changes

- 2d56457: Initial release

### Patch Changes

- Updated dependencies [1a93ffb]
  - @unavi/protocol@0.1.0
