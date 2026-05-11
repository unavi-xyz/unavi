use std::{collections::HashMap, sync::LazyLock};

use bevy::prelude::*;
use bevy_hsd::HsdRecordId;
use blake3::Hash;
use parking_lot::RwLock;

use crate::firewall::{Channel, Firewall};

pub static FIREWALL_REGISTRY: LazyLock<RwLock<HashMap<Hash, Firewall>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

static DEFAULT_FIREWALL: LazyLock<Firewall> = LazyLock::new(Firewall::default);

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

    if FIREWALL_REGISTRY.read().contains_key(&doc.0) {
        error!("unable to register firewall: document already registered");
        // Despawn the entity as safeguard to prevent firewall privilege leaks.
        commands.entity(trigger.entity).despawn();
        return;
    }

    FIREWALL_REGISTRY.write().insert(doc.0, firewall.clone());

    commands
        .entity(trigger.entity)
        .insert(RegisteredFirewall(doc.0));
}

pub fn deregister_firewalls(
    trigger: On<Remove, RegisteredFirewall>,
    ids: Query<&RegisteredFirewall>,
) {
    let id = ids.get(trigger.entity).expect("id");
    FIREWALL_REGISTRY.write().remove(&id.0);
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
