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
    HsdNamespace,
};
use iroh_docs::NamespaceId;
use parking_lot::RwLock;

use crate::{
    error::ScriptError,
    firewall::{
        Channel,
        Firewall,
    },
};

pub static FIREWALL_REGISTRY: LazyLock<RwLock<HashMap<NamespaceId, Firewall>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

#[derive(Component)]
pub struct RegisteredFirewall(NamespaceId);

pub fn register_docs(
    trigger: On<Add, Firewall>,
    docs: Query<(&HsdNamespace, &Firewall)>,
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

pub fn register_subdoc_firewall(
    trigger: On<Insert, HsdNamespace>,
    subdocs: Query<&ChildOf, (With<Hsd>, Without<Firewall>)>,
    prims: Query<&HsdChild>,
    docs: Query<&HsdNamespace>,
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

pub fn validate_firewall(
    me: &NamespaceId,
    target: &NamespaceId,
    channel: Channel,
) -> anyhow::Result<()> {
    if me == target {
        return Ok(());
    }

    // Documents with no registered firewall (e.g. pinned space docs) fall back
    // to the open default; same-space membership remains the gate.
    let firewall = FIREWALL_REGISTRY
        .read()
        .get(target)
        .cloned()
        .unwrap_or_default();

    if let Some(whitelist) = firewall.0.read().get(&channel).cloned()
        && whitelist.permits(me)
    {
        Ok(())
    } else {
        Err(ScriptError::firewall(format!("{channel:?}")).into())
    }
}
