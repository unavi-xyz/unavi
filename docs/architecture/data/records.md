# Records

## What

Fundamental data unit in The Wired. Loro CRDT document with structured schema.

## Why

- **Structured**: Consistent schema across all records
- **Conflict-Free**: Loro CRDT automatic merging
- **Addressable**: Globally unique CID-based IDs
- **Relational**: Context and relations link records
- **Secure**: Fine-grained permissions

## Structure

```
Record (Loro document)
├── genesis (immutable)
│   ├── creator: DID
│   ├── created: timestamp
│   ├── nonce: bytes
│   └── schema: string
├── permissions
│   ├── owner: [DID, ...]
│   ├── writer: [DID, ...] | "public"
│   └── reader: [DID, ...] | "public"
├── context: { record: RecordRef, relation: string } | null
├── relations: { relation-name: [RecordRef, ...] }
└── data: schema-specific content
```

## Permissions

- **owner**: Modify permissions and data
- **writer**: Modify data only
- **reader**: Read only

## Record ID

CID of the `genesis` map. Stable, globally unique identifier.

## Record References

Format: `wired://{did}/{record-id}`

Example: `wired://did:web:alice.com/bafyreiabc123`

[DID](../social/did.md) indicates which [WDS](./wds.md) to query.
