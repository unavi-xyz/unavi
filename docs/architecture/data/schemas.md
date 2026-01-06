# Schemas

## What

Immutable definitions specifying [record](./records.md) structure and validation
rules. Stored as [blobs](./blobs.md) and referenced by blake3 hash.

## Why

- **Type Safety**: Define valid record structure
- **Validation**: [WDS](./wds.md) enforces automatically
- **Interoperability**: Shared definitions enable compatibility

## Storage

Schemas are serialized and stored as blobs. Records reference schemas by hash
in their genesis metadata. A record can implement multiple schemas.

## Example

```ron
(
    id: "wired/beacon",
    version: 0,
    container: "beacon",
    layout: Map({
        "did": String,
        "endpoint": Binary,
        "expires": I64,
        "space": Binary,
    }),
)
```

## Fields

- **id**: Human-readable identifier (e.g., `wired/beacon`)
- **version**: Schema version number
- **container**: Target Loro container name in the record
- **layout**: Field structure definition

## Field Types

- `Any` - Any Loro value
- `Bool` - Boolean
- `Binary` - Raw bytes
- `F64` - 64-bit float
- `I64` - 64-bit integer
- `String` - UTF-8 string
- `List(Field)` - Homogeneous list
- `Map({ "key": Field, ... })` - Named fields

## Restrictions

Fields can have access restrictions:

```ron
Restricted(
    actions: [(who: Anyone, can: [Create])],
    value: String,
)
```

- **who**: `Anyone` or `Path("acl/manage")` (reference to ACL field)
- **can**: `Create`, `Delete`, `Update`
