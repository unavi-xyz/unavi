# client

## 0.8.1

### Patch Changes

- 75abfcdd: - Upgrade to wagmi v1 / viem
  - Upgrade three to v151

## 0.8.0

### Minor Changes

- d3218932: move editor to /studio

## 0.7.0

### Minor Changes

- 114862f0: upgrade to the latest version of the wired protocol
- bc2bdc71: move away from profile NFTs. use the new Wired Protocol identity system.
- 9b66bdbb: add pagination to avatar browser

### Patch Changes

- Updated dependencies [114862f0]
- Updated dependencies [bc2bdc71]
  - @unavi/react-client@0.6.0

## 0.6.0

### Minor Changes

- fb0120a9: migrate to the [Wired Protocol](https://github.com/wired-protocol/spec).

### Patch Changes

- Updated dependencies [fb0120a9]
  - @unavi/gltf-extensions@0.6.0
  - engine@0.2.1
  - @unavi/react-client@0.5.1

## 0.5.0

### Minor Changes

- 120335e: Move from Space IDs to space URIs. Spaces are no longer required to be NFTs - a space is now just a glTF file. All you need to access a space is a URI pointing to it. Spaces can still be NFTs, but they can now be many more things as well.

### Patch Changes

- Updated dependencies [120335e]
- Updated dependencies [7a8ae74]
  - @unavi/react-client@0.5.0
  - @unavi/protocol@0.3.0
  - @unavi/gltf-extensions@0.5.0

## 0.4.1

### Patch Changes

- 824c0e7: fix not fetching player coount from correct host server
- dfba4bd: fix avatar upload not visible

## 0.4.0

### Minor Changes

- 3c3e604: Refactor project and publication storage. May break existing projects and spaces. Will allow for better caching behavior, as published models are now immutable. When updates are pushed to a publication, a unique S3 path will be created to store the model.
- 1b2e29e: add KHR_audio support

### Patch Changes

- 2fafc0e: add loading indicator when creating a project
- a552e54: Can now run the client without needing S3 storage or a database. Features that rely on those services (such as the editor) will be removed from the UI if the needed environment variables are not present.
- Updated dependencies [4073902]
- Updated dependencies [f2fd61f]
- Updated dependencies [f285a6a]
- Updated dependencies [bf4a266]
- Updated dependencies [1b2e29e]
- Updated dependencies [1b2e29e]
- Updated dependencies [b52c4eb]
- Updated dependencies [d569e98]
- Updated dependencies [ac5cb1c]
  - engine@0.2.0
  - @unavi/gltf-extensions@0.4.0
  - @unavi/react-client@0.4.0

## 0.3.0

### Minor Changes

- e7ac7b4: adds an avatar browser menu to the settings dialog
- 83ba811: add VRM file drop support
- 6ae91ef: add space info ui to overlay
- c75e6b9: add host to space metadata
- ceed3ac: add new default avatar

### Patch Changes

- Updated dependencies [632bd65]
- Updated dependencies [c75e6b9]
- Updated dependencies [6ae91ef]
  - @unavi/react-client@0.3.0

## 0.2.0

### Minor Changes

- f5af4ee: add max distance to avatar equip

### Patch Changes

- 51b061d: fix system chat messages not using player name
- Updated dependencies [f5af4ee]
- Updated dependencies [73c1b21]
- Updated dependencies [51b061d]
- Updated dependencies [f46f168]
  - @unavi/react-client@0.2.0
  - engine@0.1.0

## 0.1.1

### Patch Changes

- 468dd79: fix editor saving broken

## 0.1.0

### Minor Changes

- 0b4121d: migrate play + editor pages to app directory
- 2d56457: Start using `@unavi/react-client` within play page
- ba1adae: change space player count API from `/playercount/:id` -> `/spaces/:id/player-count`

### Patch Changes

- Updated dependencies [8bec9cf]
- Updated dependencies [2d56457]
- Updated dependencies [1a93ffb]
  - @unavi/gltf-extensions@0.3.0
  - @unavi/react-client@0.1.0
  - @unavi/protocol@0.1.0
