use std::{
    collections::{
        HashMap,
        HashSet,
    },
    sync::{
        Arc,
        LazyLock,
    },
};

use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdChild,
    HsdDocId,
};
use hsd::id::DocId;
use parking_lot::RwLock;

use crate::{
    error::PolicyError,
    firewall::{
        Access,
        Channel,
        Firewall,
    },
};

static FIREWALL_REGISTRY: LazyLock<RwLock<HashMap<DocId, Firewall>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

#[derive(Component)]
pub struct RegisteredFirewall(pub DocId);

/// Registers a child document's firewall ahead of the spawn that attaches it.
///
/// The returned [`Firewall`] is the one the caller must attach:
/// [`register_docs`] accepts an already-registered document only when handed
/// back the same `Arc`, so anything else registering that id first is a
/// privilege leak.
#[must_use]
pub fn reserve_child_firewall(child: DocId, creator: DocId) -> Firewall {
    let firewall = Firewall::for_child_doc(creator);
    FIREWALL_REGISTRY.write().insert(child, firewall.clone());
    firewall
}

/// Closes a document to every script write, in either direction.
pub fn seal_scene_writes(doc: DocId) {
    let firewall = FIREWALL_REGISTRY.read().get(&doc).cloned();
    if let Some(firewall) = firewall {
        firewall
            .0
            .write()
            .insert(Channel::SceneWrite, Access::Restricted(HashSet::new()));
    }
}

pub fn register_docs(
    trigger: On<Add, Firewall>,
    docs: Query<(&HsdDocId, &Firewall)>,
    mut commands: Commands,
) {
    let Ok((doc, firewall)) = docs.get(trigger.entity) else {
        error!("unable to register firewall: document not found");
        return;
    };

    let mut reg = FIREWALL_REGISTRY.write();
    if let Some(existing) = reg.get(&doc.0) {
        if !Arc::ptr_eq(&existing.0, &firewall.0) {
            error!("unable to register firewall: document already registered");
            commands.entity(trigger.entity).despawn();
            return;
        }
    } else {
        reg.insert(doc.0, firewall.clone());
    }
    drop(reg);

    commands
        .entity(trigger.entity)
        .insert(RegisteredFirewall(doc.0));
}

/// A prefab instance inherits a firewall from the document that spawned it.
pub fn register_instance_firewall(
    trigger: On<Insert, HsdDocId>,
    subdocs: Query<&ChildOf, (With<Hsd>, Without<Firewall>)>,
    prims: Query<&HsdChild>,
    docs: Query<&HsdDocId>,
    mut commands: Commands,
) {
    let Ok(prim) = subdocs.get(trigger.entity).map(ChildOf::parent) else {
        return;
    };
    let Ok(parent) = prims.get(prim).map(|c| c.0) else {
        return;
    };
    let Ok(parent_id) = docs.get(parent) else {
        return;
    };
    commands
        .entity(trigger.entity)
        .insert(Firewall::for_child_doc(parent_id.0));
}

pub fn deregister_firewalls(
    trigger: On<Remove, RegisteredFirewall>,
    ids: Query<&RegisteredFirewall>,
) {
    let Ok(id) = ids.get(trigger.entity) else {
        return;
    };
    FIREWALL_REGISTRY.write().remove(&id.0);
}

pub fn validate_firewall(me: &DocId, target: &DocId, channel: Channel) -> Result<(), PolicyError> {
    if me == target {
        return Ok(());
    }

    // Documents with no registered firewall (pinned space docs) are open, with
    // same-space membership as the gate instead.
    let firewall = FIREWALL_REGISTRY
        .read()
        .get(target)
        .cloned()
        .unwrap_or_else(Firewall::open);

    if let Some(whitelist) = firewall.0.read().get(&channel).cloned()
        && whitelist.permits(me)
    {
        Ok(())
    } else {
        Err(PolicyError::Firewall(format!("{channel:?}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every test here shares the one registry.
    static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn reset() {
        FIREWALL_REGISTRY.write().clear();
    }

    #[test]
    fn a_reserved_child_firewall_is_the_one_the_registry_holds() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        reset();

        let creator = DocId([1; 32]);
        let child = DocId([2; 32]);
        let reserved = reserve_child_firewall(child, creator);

        let held = FIREWALL_REGISTRY.read().get(&child).cloned().expect("held");
        assert!(
            Arc::ptr_eq(&held.0, &reserved.0),
            "the caller must attach the same Arc register_docs will compare against"
        );
        assert!(validate_firewall(&creator, &child, Channel::SceneWrite).is_ok());
        assert!(validate_firewall(&DocId([3; 32]), &child, Channel::SceneWrite).is_err());

        reset();
    }

    #[test]
    fn sealing_scene_writes_shuts_out_the_creator_too() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        reset();

        let creator = DocId([1; 32]);
        let child = DocId([4; 32]);
        let _reserved = reserve_child_firewall(child, creator);
        seal_scene_writes(child);

        assert!(validate_firewall(&creator, &child, Channel::SceneWrite).is_err());
        assert!(
            validate_firewall(&creator, &child, Channel::SceneRead).is_ok(),
            "sealing writes must not close reads"
        );

        reset();
    }

    #[test]
    fn an_unregistered_document_is_open() {
        let _guard = TEST_LOCK.lock().expect("test lock");
        reset();

        assert!(validate_firewall(&DocId([1; 32]), &DocId([9; 32]), Channel::SceneWrite).is_ok());

        reset();
    }
}
