# Decentralized Identifiers

## What

A **Decentralized Identifier** (DID) is a globally unique identifier controlled
by cryptographic keys, not centralized authorities.

Format: `did:method:identifier`

[W3C DID Specification](https://www.w3.org/TR/did-core/)

## Why

- **Ownership**: Controlled by keys, not platforms
- **Verification**: Cryptographically verifiable
- **Portability**: Same identity across services
- **Privacy**: Selective disclosure

## DID Methods

**did:key** - Derived from public key. No resolution needed, no key rotation.

**did:web** - Uses web infrastructure. Ties identity to domain ownership.

## Resolution

DIDs resolve to DID documents containing public keys and service endpoints:

```
did:web:alice.com → DID Document → WDS endpoint (https://wds.alice.com)
```

DID documents declare [WDS](../data/wds.md) endpoints for data storage.

## DID URLs

Extend DIDs with service and resource paths:

```
did:web:example.com?service=wds&relativeRef=/records/{record-id}
```

Enable addressable references to [records](../data/records.md) without
hard-coded endpoints.

## Authentication

Challenge-response flow:

1. Client presents DID
1. Server sends nonce
1. Client signs with DID private key
1. Server verifies signature via DID document public key

No passwords or central identity providers.

## In The Wired

DIDs identify:

- **Users**: Self-sovereign identity
- **Spaces**: Referenced by DID URLs
- **Servers**: Host instances with server DID

All operations signed with DID keys. Fully decentralized authentication.
