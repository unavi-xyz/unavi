# Spaces

## What

Simulation sandboxes in The Wired. A Space is a shared environment
where multiple [HSD](./hsd.md) documents coexist, interact through
physics, and are rendered together. Addressable by
[DID](../social/did.md) URL.

## Why

- **Addressable**: Referenced by DID URLs
- **Composable**: Embed child Spaces via slots
- **Linkable**: Connect Spaces via portals
- **Multi-object**: Each object is an independent HSD with its own
  ownership and sync stream

## Storage

Spaces are [records](../data/records.md) with a manifest of member
HSD documents:

```
Space Record
├── record (genesis metadata)
├── acl (permissions)
├── space (name, object manifest)
```

The `objects` list places HSD records into the Space with root
transforms:

```
objects:
  - record: "did:web:example.com?...#record-id"
    translation: [0, 0, -5]
  - record: "did:web:other.com?...#lamp"
    translation: [2, 1, 0]
    rotation: [0, 0, 0, 1]
```

Each HSD is its own record with its own DID, ACL, and Loro document.
Different objects can be owned by different users.

## Entering a Space

1. Resolve the Space record by DID URL
2. Read the object manifest
3. Fetch and instantiate each HSD document
4. Run physics, scripts, and rendering in the shared sandbox
5. Sync edits to individual HSDs via [WDS](../data/wds.md)

## Hypermedia Controls

**Portals**: Windows into other Spaces. Walk through to navigate.
Hyperlinks in 3D, addressed by DID URL.

**Slots**: Regions for embedding child Spaces. The child Space's
objects appear within the parent, scoped by the slot's transform and
permission boundary.
