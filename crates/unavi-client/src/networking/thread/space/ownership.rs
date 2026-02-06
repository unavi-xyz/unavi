//! Dynamic object ownership resolution.

use std::collections::HashMap;

use iroh::EndpointId;
use parking_lot::RwLock;

use super::{gossip::OwnershipTimestamp, types::object_id::ObjectId};

/// Tracks ownership state for dynamic objects in a space.
#[derive(Debug, Default)]
pub struct ObjectOwnership {
    inner: RwLock<HashMap<ObjectId, OwnershipRecord>>,
}

/// Record of object ownership.
#[derive(Debug, Clone)]
pub struct OwnershipRecord {
    pub owner: EndpointId,
    pub timestamp: OwnershipTimestamp,
    pub seq: u32,
}

impl ObjectOwnership {
    pub fn new() -> Self {
        Self::default()
    }

    /// Attempt to record a claim. Returns true if accepted.
    ///
    /// Resolution rules (in order):
    /// 1. Higher timestamp wins.
    /// 2. On timestamp tie, higher seq wins.
    /// 3. On seq tie, lexicographically higher [`EndpointId`] wins (deterministic).
    pub fn try_claim(
        &self,
        object_id: ObjectId,
        claimer: EndpointId,
        timestamp: OwnershipTimestamp,
        seq: u32,
    ) -> bool {
        let mut guard = self.inner.write();

        let dominated = guard.get(&object_id).is_none_or(|existing| {
            (timestamp, seq, claimer.as_bytes())
                > (existing.timestamp, existing.seq, existing.owner.as_bytes())
        });

        if dominated {
            guard.insert(
                object_id,
                OwnershipRecord {
                    owner: claimer,
                    timestamp,
                    seq,
                },
            );
        }

        dominated
    }

    /// Release ownership. Returns true if released, false if not owner.
    pub fn release(&self, object_id: ObjectId, releaser: EndpointId) -> bool {
        let mut guard = self.inner.write();
        let dominated = guard
            .get(&object_id)
            .is_some_and(|record| record.owner == releaser);
        if dominated {
            guard.remove(&object_id);
        }
        dominated
    }

    /// Get current owner of an object.
    pub fn owner(&self, object_id: ObjectId) -> Option<EndpointId> {
        self.inner.read().get(&object_id).map(|r| r.owner)
    }

    /// Check if a specific endpoint owns the object.
    pub fn is_owner(&self, object_id: ObjectId, endpoint: EndpointId) -> bool {
        self.owner(object_id) == Some(endpoint)
    }

    /// Get all objects owned by a specific endpoint.
    pub fn objects_owned_by(&self, endpoint: EndpointId) -> Vec<ObjectId> {
        self.inner
            .read()
            .iter()
            .filter_map(|(&id, record)| {
                if record.owner == endpoint {
                    Some(id)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Remove all ownership claims by a specific endpoint.
    ///
    /// Called when a peer disconnects.
    pub fn remove_all_by(&self, endpoint: EndpointId) {
        self.inner
            .write()
            .retain(|_, record| record.owner != endpoint);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_endpoint(id: u8) -> EndpointId {
        // Generate a deterministic key from the id seed.
        use rand::SeedableRng;
        let mut seed = [0u8; 32];
        seed[0] = id;
        let mut rng = rand::rngs::StdRng::from_seed(seed);
        let secret = iroh::SecretKey::generate(&mut rng);
        secret.public()
    }

    fn make_object(id: u8) -> ObjectId {
        let mut record = [0u8; 32];
        record[0] = id;
        ObjectId { record, index: 0 }
    }

    #[test]
    fn test_first_claim_wins() {
        let ownership = ObjectOwnership::new();
        let obj = make_object(1);
        let peer1 = make_endpoint(1);

        assert!(ownership.try_claim(obj, peer1, 1000, 0));
        assert!(ownership.is_owner(obj, peer1));
    }

    #[test]
    fn test_higher_timestamp_wins() {
        let ownership = ObjectOwnership::new();
        let obj = make_object(1);
        let peer1 = make_endpoint(1);
        let peer2 = make_endpoint(2);

        assert!(ownership.try_claim(obj, peer1, 1000, 0));
        assert!(ownership.try_claim(obj, peer2, 2000, 0));
        assert!(ownership.is_owner(obj, peer2));
    }

    #[test]
    fn test_lower_timestamp_loses() {
        let ownership = ObjectOwnership::new();
        let obj = make_object(1);
        let peer1 = make_endpoint(1);
        let peer2 = make_endpoint(2);

        assert!(ownership.try_claim(obj, peer1, 2000, 0));
        assert!(!ownership.try_claim(obj, peer2, 1000, 0));
        assert!(ownership.is_owner(obj, peer1));
    }

    #[test]
    fn test_seq_tiebreaker() {
        let ownership = ObjectOwnership::new();
        let obj = make_object(1);
        let peer1 = make_endpoint(1);
        let peer2 = make_endpoint(2);

        assert!(ownership.try_claim(obj, peer1, 1000, 0));
        assert!(ownership.try_claim(obj, peer2, 1000, 1));
        assert!(ownership.is_owner(obj, peer2));
    }

    #[test]
    fn test_endpoint_id_tiebreaker() {
        let ownership = ObjectOwnership::new();
        let obj = make_object(1);
        let peer1 = make_endpoint(1);
        let peer2 = make_endpoint(2);

        assert!(ownership.try_claim(obj, peer1, 1000, 0));
        // Same timestamp and seq, peer2 has higher ID.
        assert!(ownership.try_claim(obj, peer2, 1000, 0));
        assert!(ownership.is_owner(obj, peer2));
    }

    #[test]
    fn test_release() {
        let ownership = ObjectOwnership::new();
        let obj = make_object(1);
        let peer1 = make_endpoint(1);
        let peer2 = make_endpoint(2);

        assert!(ownership.try_claim(obj, peer1, 1000, 0));
        assert!(!ownership.release(obj, peer2)); // Wrong owner.
        assert!(ownership.release(obj, peer1));
        assert!(ownership.owner(obj).is_none());
    }

    #[test]
    fn test_remove_all_by() {
        let ownership = ObjectOwnership::new();
        let peer1 = make_endpoint(1);
        let peer2 = make_endpoint(2);

        let obj1 = make_object(1);
        let obj2 = make_object(2);
        let obj3 = make_object(3);

        ownership.try_claim(obj1, peer1, 1000, 0);
        ownership.try_claim(obj2, peer1, 1000, 0);
        ownership.try_claim(obj3, peer2, 1000, 0);

        ownership.remove_all_by(peer1);

        assert!(ownership.owner(obj1).is_none());
        assert!(ownership.owner(obj2).is_none());
        assert!(ownership.is_owner(obj3, peer2));
    }
}
