# Spaces

## What

Bounded 3D environments in The Wired. Addressable, linkable, and composable
spatial hypermedia documents.

## Why

- **Addressable**: Referenced by [DID](../social/did.md) URLs
- **Composable**: Nest spaces within spaces via slots
- **Linkable**: Connect spaces via portals
- **Collaborative**: Multi-user editing via [WDS](../data/wds.md)

## Storage

Spaces are [records](../data/records.md) combining the `wired/space` schema
with [HSD](./hsd.md) for scene content.

```
Space Record
├── record (genesis metadata)
├── acl (permissions)
├── space (name, metadata)
└── hsd (scene graph)
```

## Hypermedia Controls

**Portals**: Windows into other spaces. Walk through to navigate. Function as
hyperlinks in 3D, addressed by DID URL.

**Slots**: Regions for embedding child spaces. Enables composition without
full navigation.
