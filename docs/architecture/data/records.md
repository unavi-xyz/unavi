# Records

## What

Fundamental data unit in The Wired. A Loro CRDT document with structured
[schemas](./schemas.md) and access control.

## Why

- **Structured**: Schema-enforced data consistency
- **Conflict-Free**: Loro CRDT automatic merging
- **Addressable**: Globally unique blake3 hash IDs
- **Secure**: Fine-grained permissions

## Structure

```
Record (Loro document)
├── record (immutable genesis)
│   ├── creator: DID
│   ├── nonce: [u8; 16]
│   ├── timestamp: i64
│   └── schemas: [Hash, ...]
└── acl (mutable permissions)
    ├── manage: [DID, ...]
    ├── write: [DID, ...]
    └── read: [DID, ...]
```

## Permissions

Hierarchical access control:

- **manage**: Modify permissions and data
- **write**: Modify data only
- **read**: Read only

## Record ID

blake3 hash of the record's genesis metadata. Stable, globally unique.

## Envelopes

Signed incremental updates to a record:

- Author [DID](../social/did.md) who made the change
- Version vectors (from/to) for CRDT sync
- Serialized Loro operations

Envelopes enable sync between [WDS](./wds.md) instances. Each envelope is
cryptographically signed by its author and validated against the ACL.
