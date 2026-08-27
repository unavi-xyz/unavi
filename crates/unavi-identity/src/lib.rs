//! Who a peer is, and how it proves that over an iroh endpoint.

pub mod auth;
pub mod identity;
pub mod jwk;
pub mod resolve;
pub mod signed_bytes;

/// DID document service `id` naming the iroh endpoint a DID answers on.
pub const ENDPOINT_SERVICE_ID: &str = "iroh";
pub const ENDPOINT_SERVICE_TYPE: &str = "IrohEndpoint";
