use std::sync::{
    Arc,
    LazyLock,
    Mutex,
};

use bevy::platform::collections::HashMap;
use iroh::{
    EndpointId,
    endpoint::{
        Connection,
        PathId,
    },
};

static CONNS: LazyLock<Mutex<HashMap<EndpointId, Arc<Connection>>>> =
    LazyLock::new(Mutex::default);

/// A point-in-time bandwidth and latency reading for one peer, pulled from the
/// underlying QUIC connection. Byte counters are cumulative; the panel diffs them.
#[derive(Clone, Copy)]
pub struct PeerNetStats {
    pub peer:     [u8; 32],
    pub bytes_tx: u64,
    pub bytes_rx: u64,
    pub rtt_ms:   f32,
}

/// Tracks a connection for the dev tools network panel, untracking it when the
/// returned guard drops (i.e. when the connection task ends).
pub fn track(peer: EndpointId, conn: Arc<Connection>) -> ConnGuard {
    CONNS.lock().expect("stats lock").insert(peer, conn);
    ConnGuard(peer)
}

pub struct ConnGuard(EndpointId);

impl Drop for ConnGuard {
    fn drop(&mut self) {
        CONNS.lock().expect("stats lock").remove(&self.0);
    }
}

/// Current stats for every live connection.
pub fn snapshot() -> Vec<PeerNetStats> {
    CONNS
        .lock()
        .expect("stats lock")
        .iter()
        .map(|(peer, conn)| {
            let s = conn.stats();
            let rtt_ms = conn
                .rtt(PathId::ZERO)
                .map_or(0.0, |d| d.as_secs_f32() * 1000.0);
            PeerNetStats {
                peer: *peer.as_bytes(),
                bytes_tx: s.udp_tx.bytes,
                bytes_rx: s.udp_rx.bytes,
                rtt_ms,
            }
        })
        .collect()
}
