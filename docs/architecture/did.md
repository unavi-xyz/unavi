# Decentralized Identifiers

## What

A **Decentralized Identifier** (DID) is a globally unique identifier that doesn't
require a centralized authority.

Format: `did:method:identifier` (e.g., `did:web:example.com`).

[W3C DID Specification](https://www.w3.org/TR/did-core/)

## Why

- **Ownership**: Controlled by cryptographic keys, not platforms.
- **Verification**: Cryptographically verifiable without intermediaries.
- **Portability**: Same identity across services and applications.
- **Privacy**: Selective disclosure of information.

## In The Wired

DIDs identify:

- **Agents**: Self-sovereign identity.
- **Spaces**: Referenced by DID URLs (DID + record ID on that DID's DWN).
- **Servers**: Host instances use server DID + instance record ID.

### DID URLs

DID URLs extend DIDs with service endpoints and resource paths:

```
did:web:example.com?service=dwn&relativeRef=/records/{record-id}
```

- `service=dwn` identifies the DWN service endpoint in the DID Document.
- `relativeRef=/records/{record-id}` navigates to a specific DWN record.

DIDs link to [DWN](./dwn.md) endpoints via DID Documents, enabling discovery of
data storage locations.
