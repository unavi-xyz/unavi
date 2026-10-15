//! Who a peer is, and how it proves that over an iroh endpoint.
//!
//! A DID signs for a user; an endpoint key for one of their devices.
//! [`auth`] binds the two per connection, [`authorship`] binds a DID to the
//! documents it writes, and [`signed`] carries any other claim either makes.

pub mod auth;
pub mod authorship;
pub mod did_document;
pub mod identity;
pub mod jwk;
pub mod resolver;
pub mod signed;
