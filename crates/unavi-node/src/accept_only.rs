//! A node answers peers and never dials them.

use iroh::{
    EndpointAddr,
    endpoint::{
        BeforeConnectOutcome,
        EndpointHooks,
    },
};

/// Refuses every outgoing connection.
///
/// iroh-docs redials every peer that once completed a sync of a namespace
/// whenever that namespace is served again, and iroh-gossip redials its
/// passive view as members leave. A node serving many short-lived clients
/// would otherwise spend its time dialling clients that are long gone.
/// Clients dial the node, and sync and gossip run over those connections.
#[derive(Debug)]
pub struct AcceptOnly;

impl EndpointHooks for AcceptOnly {
    fn before_connect<'a>(
        &'a self,
        _remote_addr: &'a EndpointAddr,
        _alpn: &'a [u8],
    ) -> impl Future<Output = BeforeConnectOutcome> + Send + 'a {
        std::future::ready(BeforeConnectOutcome::Reject)
    }
}
