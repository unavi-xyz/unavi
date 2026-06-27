use std::{
    collections::HashMap,
    sync::{
        Arc,
        LazyLock,
    },
};

use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdChild,
    HsdRecordId,
};
use blake3::Hash;
use parking_lot::RwLock;

use crate::firewall::{
    Channel,
    Firewall,
};

pub static FIREWALL_REGISTRY: LazyLock<RwLock<HashMap<Hash, Firewall>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

static DEFAULT_FIREWALL: LazyLock<Firewall> = LazyLock::new(Firewall::closed);

#[derive(Component)]
pub struct RegisteredFirewall(Hash);

pub fn register_docs(
    trigger: On<Add, Firewall>,
    docs: Query<(&HsdRecordId, &Firewall)>,
    mut commands: Commands,
) {
    let Ok((doc, firewall)) = docs.get(trigger.entity) else {
        error!("unable to register firewall: document not found");
        return;
    };

    let mut reg = FIREWALL_REGISTRY.write();
    if let Some(existing) = reg.get(&doc.0) {
        // Child docs are pre-registered by spawn_child_doc, so allow the same
        // Arc; reject anything else as a privilege leak.
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

/// A sub-document is spawned `ChildOf` the prim that declares it; it inherits a
/// child-doc firewall keyed to that prim's document so other docs may exchange
/// events and read its scene/kv across the boundary.
pub fn register_subdoc_firewall(
    trigger: On<Insert, HsdRecordId>,
    subdocs: Query<&ChildOf, (With<Hsd>, Without<Firewall>)>,
    prims: Query<&HsdChild>,
    docs: Query<&HsdRecordId>,
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
    let id = ids.get(trigger.entity).expect("id");
    FIREWALL_REGISTRY.write().remove(&id.0);
    unavi_quota::registry::forget_document(id.0);
}

pub fn validate_firewall(me: &Hash, target: &Hash, channel: Channel) -> anyhow::Result<()> {
    if me == target {
        return Ok(());
    }

    let firewall = FIREWALL_REGISTRY
        .read()
        .get(target)
        .cloned()
        .unwrap_or_else(|| DEFAULT_FIREWALL.clone());

    if let Some(whitelist) = firewall.0.read().get(&channel).cloned()
        && whitelist.permits(me)
    {
        Ok(())
    } else {
        Err(anyhow::anyhow!("{channel:?} blocked by firewall"))
    }
}
