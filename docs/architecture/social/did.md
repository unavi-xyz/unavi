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

## DID URLs

Extend DIDs with service and resource paths:

```
did:web:example.com?service=wds&relativeRef=/records/{record-id}
```

Enable addressable references to [records](../data/records.md) without
hard-coded endpoints.
