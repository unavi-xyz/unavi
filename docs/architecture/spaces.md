# Spaces

## What

A **space** is the fundamental unit of The Wired: A bounded
3D region containing geometry, entities, and interactive elements.

Spaces are hyperdocuments in The Wired's spatial hypermedia system —
addressable, linkable, and composable units of 3D information.

## Properties

- **Metadata**: Name, description, tags, creator [DID](./did.md), timestamp.
- **Geometry**: [glXF](https://github.com/KhronosGroup/glTF-External-Reference)
  scene format (glTF with external references).
- **Scripts**: [WASM](./wasm.md) components for interactive behavior.
- **Slots**: Regions where child spaces can be nested.
- **Portals**: Links to other spaces for seamless navigation.

## Storage

Spaces are stored as records on [DWNs](./dwn.md), referenced by DID URLs.

Space records follow [`/protocol/dwn/schemas/space.json`](../../protocol/dwn/schemas/space.json),
containing metadata (name, description) plus references to assets.

## Instances

### Local

Loaded from filesystem or DWN, runs locally in the client. Offline, single-user.

### Hosted

Remote server hosts a space, facilitating multiplayer networking within it.
Online, multi-user.

Server clones space from DWN on initialization, creating a live instance with
its own DID URL. Runtime mutations affect the instance, not the original space.
However, mutations *are* persistent within the instance and can be saved as a
new space to be loaded or used elsewhere.

#### Chunking

Servers may spatially partition spaces (e.g., octree, grid) across multiple
machines for horizontal scaling. Clients connect to nearby chunks via
[WebTransport](https://www.w3.org/TR/webtransport/), unifying them seamlessly.

## Hypermedia Controls

### Slots (Composition)

Spaces may be composed via designated AABB slots for child spaces to be inserted.
This can be done statically or dynamically at runtime.

Parent spaces can query into the child spaces for cross-space interactions
(e.g., querying entities or mutating data). This only applies to entities the
child explicitly marks as public.

### Portals (Linking)

Portals are seamless windows into other spaces. Agents can walk through portals
between spaces (potentially on different servers) and carry objects with them.

Unlike slots (which embed child spaces), portals link independent spaces.
Portals function as hyperlinks in hyperspace, addressing other spaces by DID URL.

## Key Concepts

- **Separation**: Spatial data is independent of networking infrastructure.
- **Composability**: Fractal nesting via slots and DID links.
- **Sovereignty**: Users own spaces; servers are transient hosts.
- **Scalability**: Horizontal scaling via multi-server chunking.
