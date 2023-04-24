# host

## 0.4.0

### Minor Changes

- fb0120a9: migrate to the [Wired Protocol](https://github.com/wired-protocol/spec).

## 0.3.0

### Minor Changes

- 120335e: Move from Space IDs to space URIs. Spaces are no longer required to be NFTs - a space is now just a glTF file. All you need to access a space is a URI pointing to it. Spaces can still be NFTs, but they can now be many more things as well.

### Patch Changes

- Updated dependencies [120335e]
  - @unavi/protocol@0.3.0

## 0.2.1

### Patch Changes

- 94c1f53: upgrade uWebsocket.js to v20.20.0
- Updated dependencies [2f029aa]
  - @unavi/protocol@0.2.0

## 0.2.0

### Minor Changes

- ba1adae: change space player count API from `/playercount/:id` -> `/spaces/:id/player-count`

### Patch Changes

- Updated dependencies [1a93ffb]
  - @unavi/protocol@0.1.0

## 0.1.1

### Patch Changes

- ab948db: Upgrade mediasoup

## 0.1.0

### Minor Changes

- ee87108: Initial release
