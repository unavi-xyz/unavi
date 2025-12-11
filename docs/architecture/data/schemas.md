# Schemas

## What

Immutable definitions specifying [record](./records.md) relations and permission
rules. Enforce type safety and access control.

## Why

- **Type Safety**: Define valid record relations
- **Access Control**: Declarative permission rules
- **Interoperability**: Shared definitions enable compatibility
- **Validation**: [WDS](./wds.md) enforces automatically

## Definition

```ron
Schema(
    id: "wired-protocol.org/schemas/chatroom",
    version: 1,

    relations: {
        "message": (
            schema: "wired-protocol.org/schemas/chatmessage",
            create: Writer,
            delete: [Author, Owner],
        ),
    },

    data: {
        "name": (set: Owner),
    },
)
```

## Relations

Define what records can link via `context`:

- **schema**: Required schema for related records
- **create**: Who can create (Writer, Owner, etc.)
- **delete**: Who can remove ([Author, Owner], etc.)

## Data Rules

Define who can modify data fields:

- **set**: Owner, Writer, Author, or None (immutable)

## Permission Values

- **Author**: Record creator (from genesis)
- **Owner**: Record owner
- **Writer**: Any writer or owner
- **None**: Immutable after creation
