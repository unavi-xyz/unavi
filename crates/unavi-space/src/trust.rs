use bevy::prelude::*;
use bevy_wds::{
    LocalDocs,
    root_doc,
};
use bytes::Bytes;
use time::OffsetDateTime;
use unavi_policy::trust::{
    self,
    vouch::{
        Vouch,
        subject_hash,
    },
};
use unavi_util::async_task::spawn_async_task;

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
