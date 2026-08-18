use std::collections::HashMap;

use serde::{
    Deserialize,
    Serialize,
};
use xdid::core::did::Did;

use crate::trust::Trust;

/// Longest path the metric will follow. Three hops is where a chain stops
/// meaning anything concrete to the person at the near end of it.
pub const HORIZON: usize = 3;
/// Applied once per hop past the first, so a longer path is worth less than a
/// short one carrying the same weights.
pub const DECAY: f32 = 0.75;
/// Total weight one intermediate may pass on. Bounds the damage a careless
/// friend does by vouching for everyone.
pub const CAPACITY: f32 = 10.0;

pub const TRUSTED_SCORE: f32 = 0.8;
pub const KNOWN_SCORE: f32 = 0.3;

/// A signed statement that one DID trusts another, published under
/// `vouches/<subject>` in the voucher's root doc.
///
/// The subject is a salted hash rather than the DID, with the salt public.
/// Anyone may *test* whether a specific peer is vouched for — you always know
/// the DID of the person in front of you — but nobody can enumerate who you
/// know, so publishing a vouch list does not leak a social graph.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Vouch {
    pub subject: [u8; 32],
    /// `0..=100`.
    pub weight:  u8,
    pub at:      u64,
}

#[must_use]
pub fn subject_hash(salt: &[u8], did: &Did) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(salt);
    hasher.update(did.to_string().as_bytes());
    *hasher.finalize().as_bytes()
}

/// Resolved vouches, as edges between DIDs the viewer can actually name.
///
/// Building this is where the salted hashes are undone, and it is only
/// possible against a candidate set: a peer's published vouches are hashes,
/// so an edge can be discovered but never enumerated. The candidates are the
/// DIDs the viewer already knows — the ones it has vouched for itself, plus
/// every peer currently present, each of which proved a DID over its own
/// connection. That bound is what makes a horizon past two hops reachable at
/// all.
#[derive(Debug, Default)]
pub struct Graph {
    edges: HashMap<Did, HashMap<Did, f32>>,
}

impl Graph {
    pub fn add(&mut self, voucher: Did, subject: Did, weight: u8) {
        self.edges
            .entry(voucher)
            .or_default()
            .insert(subject, f32::from(weight.min(100)) / 100.0);
    }

    /// Resolves `published` against `candidates`, keeping the edges whose
    /// subject the viewer can name.
    pub fn add_published(
        &mut self,
        voucher: Did,
        salt: &[u8],
        published: &[Vouch],
        candidates: &[Did],
    ) {
        for vouch in published {
            if let Some(subject) = candidates
                .iter()
                .find(|did| subject_hash(salt, did) == vouch.subject)
            {
                self.add(voucher.clone(), subject.clone(), vouch.weight);
            }
        }
    }

    /// The ego-centric score `from` has for `to`.
    ///
    /// The maximum over simple paths, never the sum. That is the sybil
    /// defence: a thousand identities vouching for each other achieve nothing,
    /// because every path still has to enter the graph through someone `from`
    /// vouched for, and a maximum ignores how many of them there are.
    #[must_use]
    pub fn score(&self, from: &Did, to: &Did) -> f32 {
        if from == to {
            return 1.0;
        }
        let mut visited = vec![from.clone()];
        self.walk(from, to, &mut visited, 0)
    }

    fn walk(&self, at: &Did, to: &Did, visited: &mut Vec<Did>, hops: usize) -> f32 {
        if hops >= HORIZON {
            return 0.0;
        }
        let Some(out) = self.edges.get(at) else {
            return 0.0;
        };

        // The source spends its own opinion freely; only what an intermediate
        // passes along is capped.
        let scale = if hops == 0 {
            1.0
        } else {
            (CAPACITY / out.len() as f32).min(1.0)
        };

        let mut best = 0.0f32;
        for (next, weight) in out {
            let edge = weight * scale;
            let value = if next == to {
                edge * DECAY.powi(i32::try_from(hops).unwrap_or(i32::MAX))
            } else if visited.contains(next) {
                continue;
            } else {
                visited.push(next.clone());
                let onward = edge * self.walk(next, to, visited, hops + 1);
                visited.pop();
                onward
            };
            best = best.max(value);
        }
        best
    }
}

/// The rung a score earns.
///
/// Never [`Trust::Myself`]: that is who you are, not something a graph can
/// conclude. Never [`Trust::Blocked`] either — a low score means the graph has
/// nothing to say, and distrust does not propagate.
#[must_use]
pub fn rung(score: f32) -> Trust {
    if score >= TRUSTED_SCORE {
        Trust::Trusted
    } else if score >= KNOWN_SCORE {
        Trust::Known
    } else {
        Trust::Guest
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    fn did(name: &str) -> Did {
        Did::from_str(&format!("did:web:{name}.example")).expect("did")
    }

    #[test]
    fn a_salted_subject_does_not_reveal_the_did() {
        let target = did("bob");
        assert_ne!(
            subject_hash(b"salt-a", &target),
            subject_hash(b"salt-b", &target),
            "two vouchers must not publish the same hash for one subject"
        );
    }

    #[test]
    fn a_published_vouch_resolves_only_against_a_named_candidate() {
        let (alice, bob, carol) = (did("alice"), did("bob"), did("carol"));
        let salt = b"alice-salt";
        let published = [Vouch {
            subject: subject_hash(salt, &bob),
            weight:  90,
            at:      0,
        }];

        let mut graph = Graph::default();
        graph.add_published(
            alice.clone(),
            salt,
            &published,
            std::slice::from_ref(&carol),
        );
        assert!(
            graph.score(&alice, &bob) <= f32::EPSILON,
            "a subject the viewer cannot name stays unresolved"
        );

        graph.add_published(alice.clone(), salt, &published, &[bob.clone(), carol]);
        assert!(graph.score(&alice, &bob) > 0.0);
    }

    #[test]
    fn a_direct_vouch_is_not_decayed() {
        let (me, bob) = (did("me"), did("bob"));
        let mut graph = Graph::default();
        graph.add(me.clone(), bob.clone(), 80);

        assert!((graph.score(&me, &bob) - 0.8).abs() < f32::EPSILON);
        assert_eq!(rung(graph.score(&me, &bob)), Trust::Trusted);
    }

    #[test]
    fn a_friend_of_a_friend_is_known_not_trusted() {
        let (me, alice, bob) = (did("me"), did("alice"), did("bob"));
        let mut graph = Graph::default();
        graph.add(me.clone(), alice.clone(), 100);
        graph.add(alice, bob.clone(), 100);

        let score = graph.score(&me, &bob);
        assert!((score - DECAY).abs() < f32::EPSILON, "one hop of decay");
        assert_eq!(rung(score), Trust::Known);
    }

    #[test]
    fn nothing_is_reachable_past_the_horizon() {
        let chain = ["me", "a", "b", "c", "d"].map(did);
        let mut graph = Graph::default();
        for pair in chain.windows(2) {
            graph.add(pair[0].clone(), pair[1].clone(), 100);
        }

        assert!(graph.score(&chain[0], &chain[HORIZON]) > 0.0);
        assert!(
            graph.score(&chain[0], &chain[HORIZON + 1]) <= f32::EPSILON,
            "a path longer than the horizon is not a path"
        );
    }

    #[test]
    fn minting_identities_that_vouch_for_each_other_achieves_nothing() {
        let (me, mule, target) = (did("me"), did("mule"), did("target"));
        let sybils = (0..500)
            .map(|i| did(&format!("sybil{i}")))
            .collect::<Vec<_>>();

        let mut graph = Graph::default();
        graph.add(me.clone(), mule.clone(), 30);
        for sybil in &sybils {
            graph.add(mule.clone(), sybil.clone(), 100);
            graph.add(sybil.clone(), target.clone(), 100);
        }

        assert_eq!(
            rung(graph.score(&me, &target)),
            Trust::Guest,
            "multiplicity must not add up: the max ignores how many paths there are"
        );
    }

    /// Builds me -> voucher -> target, where the voucher also vouches for
    /// `extra` other peers.
    fn through_a_voucher_of(extra: usize) -> f32 {
        let (me, voucher, target) = (did("me"), did("voucher"), did("target"));
        let mut graph = Graph::default();
        graph.add(me.clone(), voucher.clone(), 100);
        graph.add(voucher.clone(), target.clone(), 100);
        for i in 0..extra {
            graph.add(voucher.clone(), did(&format!("other{i}")), 100);
        }
        graph.score(&me, &target)
    }

    #[test]
    fn a_node_vouching_for_everyone_passes_less_per_edge() {
        let focused = through_a_voucher_of(0);
        let at_capacity = through_a_voucher_of(CAPACITY as usize - 1);
        let spread_thin = through_a_voucher_of(CAPACITY as usize * 4);

        assert!(
            (focused - at_capacity).abs() < f32::EPSILON,
            "a voucher inside its capacity is not penalised"
        );
        assert!(
            spread_thin < focused,
            "spreading vouches past capacity must dilute each one"
        );
    }

    #[test]
    fn a_low_score_is_a_guest_never_a_block() {
        assert_eq!(rung(0.0), Trust::Guest);
        assert_eq!(rung(KNOWN_SCORE - 0.01), Trust::Guest);
        assert_eq!(rung(KNOWN_SCORE), Trust::Known);
        assert_eq!(rung(TRUSTED_SCORE), Trust::Trusted);
    }
}
