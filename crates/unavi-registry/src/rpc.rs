//! The `wired/registry` wire protocol.
//!
//! The caller's DID is read off the connection, proven by `wired/auth`, so no
//! request carries a credential. Reads need none; writes refuse a caller that
//! proved nothing.

use iroh_docs::NamespaceId;
use irpc::{
    channel::oneshot,
    rpc_requests,
};
use serde::{
    Deserialize,
    Serialize,
};
use unavi_identity::{
    authorship::Authorship,
    signed::Signed,
};

use crate::{
    claim::{
        Presence,
        Submission,
    },
    error::RegistryError,
    views::ViewIds,
};

pub const ALPN: &[u8] = b"wired/registry";

#[rpc_requests(message = RegistryMessage)]
#[derive(Debug, Serialize, Deserialize)]
pub enum RegistryService {
    /// Lists a namespace, or refreshes its listing. The caller must author it.
    #[rpc(tx=oneshot::Sender<Result<(), RegistryError>>)]
    #[wrap(Submit)]
    Submit {
        submission: Signed<Submission>,
        authorship: Authorship,
    },
    /// Removes the caller's listing of a namespace.
    #[rpc(tx=oneshot::Sender<Result<(), RegistryError>>)]
    #[wrap(Retract)]
    Retract { ns: NamespaceId },
    /// Heartbeats live occupancy. Never persisted.
    #[rpc(tx=oneshot::Sender<Result<(), RegistryError>>)]
    #[wrap(Announce)]
    Announce { presence: Signed<Presence> },
    /// Current occupants of a namespace, each individually signed.
    ///
    /// A public directory: anyone may ask who is in a space, as anyone joining
    /// its gossip topic would learn anyway.
    #[rpc(tx=oneshot::Sender<Result<Vec<Signed<Presence>>, RegistryError>>)]
    #[wrap(Occupants)]
    Occupants { ns: NamespaceId },
    /// The namespaces of this registry's view docs.
    #[rpc(tx=oneshot::Sender<Result<ViewIds, RegistryError>>)]
    #[wrap(Views)]
    Views,
}
