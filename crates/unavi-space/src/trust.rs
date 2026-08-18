use bevy::prelude::*;
use bevy_wds::{
    LocalDocs,
    root_doc,
};
use bytes::Bytes;
use iroh::EndpointId;
use time::OffsetDateTime;
use unavi_policy::{
    identity,
    trust::{
        self,
        Trust,
        vouch::{
            Vouch,
            subject_hash,
        },
    },
};
use unavi_util::async_task::spawn_async_task;

use crate::{
    connection::disconnect,
    state::replicas,
};

/// Where a vouch lives in the voucher's root doc.
const VOUCH_PREFIX: &str = "vouches/";

/// Loads the local trust table, which is the user's own opinion and so has to
/// outlive the session.
pub fn load_trust_table() {
    trust::load(unavi_util::dirs::data_local_dir());
}

/// Publishes the local vouch list under salted hashes.
///
/// Only the hashes go out: anyone may test whether a peer they can already
/// name is vouched for, and nobody can enumerate the list.
pub fn publish_vouches(docs: Query<&LocalDocs>) {
    let Some(ns) = root_doc() else {
        return;
    };
    let Ok(docs) = docs.single().map(|d| d.0.clone()) else {
        return;
    };

    let salt = trust::salt();
    let at = OffsetDateTime::now_utc().unix_timestamp().unsigned_abs();
    let vouches = trust::my_vouches()
        .into_iter()
        .map(|(did, weight)| Vouch {
            subject: subject_hash(&salt, &did),
            weight,
            at,
        })
        .collect::<Vec<_>>();

    spawn_async_task(async move {
        for vouch in vouches {
            let key = format!("{VOUCH_PREFIX}{}", hex(&vouch.subject));
            let Ok(bytes) = postcard::to_allocvec(&vouch) else {
                continue;
            };
            if let Err(err) = wds::kv::set(&docs, ns, &key, Bytes::from(bytes)).await {
                warn!(?err, "failed to publish vouch");
            }
        }
    });
}

/// Blocks `peer` and undoes what they contributed, in one call.
///
/// Retroactive by construction. Their pins, authority claims and owner-authored
/// KV hang off the `RemotePeer` entity and cascade away when the connection
/// drops; their neutral cells are rolled back explicitly, since those
/// deliberately outlive a disconnect. The rung is set first so a reconnect
/// while the rest is still unwinding does not arrive as a guest.
///
/// A peer that proved no DID can still be disconnected but not durably blocked:
/// there is no stable identity to record the decision against, and an endpoint
/// id is not one.
pub fn eject(peer: [u8; 32]) -> Result<(), NoIdentity> {
    let did = identity::did_of(peer).ok_or(NoIdentity)?;
    trust::set_override(did, Trust::Blocked);

    let reverted = replicas::revert_neutral_writes(peer);
    info!(reverted, "Ejected peer");

    if let Err(err) = trust::save(unavi_util::dirs::data_local_dir()) {
        warn!(?err, "failed to persist the block");
    }

    if let Ok(endpoint) = EndpointId::from_bytes(&peer) {
        disconnect(endpoint);
    }
    Ok(())
}

/// A peer that proved no DID, and so has nothing durable to block.
#[derive(Debug, thiserror::Error)]
#[error("peer proved no identity to block")]
pub struct NoIdentity;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut out, b| {
        use std::fmt::Write;
        let _ = write!(out, "{b:02x}");
        out
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_published_key_names_the_hash_not_the_did() {
        let key = format!("{VOUCH_PREFIX}{}", hex(&[0xab; 32]));
        assert_eq!(key, format!("{VOUCH_PREFIX}{}", "ab".repeat(32)));
        assert!(
            !key.contains("did:"),
            "a published key must not carry a plaintext identifier"
        );
    }
}
