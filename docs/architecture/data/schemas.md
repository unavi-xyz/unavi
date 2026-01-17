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
in their genesis metadata, mapping container names to schema hashes. A record
can implement multiple schemas, one per container.

## Example

```ron
(
    id: "wired/example",
    version: 0,
    layout: Struct({
        "did": String,
        "endpoint": Binary,
        "timestamp": I64,
        "extra": Optional(Any),
    }),
)
```

## Schema Structure

- **id**: Human-readable identifier (e.g., `wired/example`)
- **version**: Schema version number
- **layout**: Field structure definition

Schemas are general-purpose validators that can be applied to any Loro value.
Container names are specified in the record's schema map, not in the schema itself.

## Field Types

### Primitives

- `Any` - Any Loro value
- `Bool` - Boolean
- `Binary` - Raw bytes
- `F64` - 64-bit float
- `I64` - 64-bit integer
- `String` - UTF-8 string

### Containers

- `List(Field)` - Homogeneous list
- `MovableList(Field)` - List with reorder/move support
- `Map(Field)` - Homogeneous map (any string keys, all values same type)
- `Struct({ "key": Field, .. })` - Fixed keys with heterogeneous types
- `Tree(Field)` - Hierarchical tree structure

### Modifiers

- `Optional(Field)` - Field may be null or missing
- `Restricted { actions, value }` - Access control wrapper

## Restrictions

Fields can have access restrictions:

```ron
Restricted(
    actions: [(who: Anyone, can: [Update])],
    value: String,
)
```

- **who**: `Anyone` or `Path("acl.manage")` (reference to ACL field)
- **can**: `Create`, `Delete`, `Update`
