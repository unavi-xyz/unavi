# @unavi/gltf-extensions

## 0.6.0

### Minor Changes

- fb0120a9: remove Space extension. Spaces are now defined using a metadata.json file, according to the [Wired Protocol](https://github.com/wired-protocol/spec).

## 0.5.0

### Minor Changes

- 7a8ae74: add the WIRED_space extension

## 0.4.0

### Minor Changes

- f2fd61f: add KHR_audio extension
- ac5cb1c: use zod schemas within Avatar, Collider, and SpawnPoint extensions to validate input from gltf file

### Patch Changes

- f285a6a: rename Variable -> BehaviorVariable
- bf4a266: don't throw an error when reading an invalid extension, just console.warn it and continue

## 0.3.0

### Minor Changes

- 8bec9cf: add the `VRMMetadata` and `VRM0Metadata` extensions, which are partial implementations of the larger `VRMC_vrm` and `VRM 0.0` gltf extensions. These allow for the reading of VRM metadata from a file.

## 0.2.1

### Patch Changes

- add repository to package json

## 0.2.0

### Minor Changes

- 96cebab: add `WIRED_avatar` extension
