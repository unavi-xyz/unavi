use std::sync::LazyLock;

use bevy::prelude::*;
use bevy_hsd::HsdRecordId;
use blake3::Hash;

use crate::firewall::{Channel, Firewall};

pub static FIREWALL_REGISTRY: LazyLock<scc::HashMap<Hash, Firewall>> =
    LazyLock::new(scc::HashMap::default);

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

    if FIREWALL_REGISTRY.contains_sync(&doc.0) {
        error!("unable to register firewall: document already registered");
        // This should not be allowed to happen.
        // Despawn the entity as safeguard to prevent firewall privledge leaks.
        commands.entity(trigger.entity).despawn();
        return;
    }

    FIREWALL_REGISTRY.upsert_sync(doc.0, firewall.clone());

    commands
        .entity(trigger.entity)
        .insert(RegisteredFirewall(doc.0));
}

pub fn deregister_firewalls(
    trigger: On<Remove, RegisteredFirewall>,
    ids: Query<&RegisteredFirewall>,
) {
    let id = ids.get(trigger.entity).expect("id");
    FIREWALL_REGISTRY.remove_sync(&id.0);
}

pub fn validate_firewall(me: &Hash, target: &Hash, channel: Channel) -> anyhow::Result<()> {
    if me == target {
        return Ok(());
    }

    if let Some(whitelist) = FIREWALL_REGISTRY
        .get_sync(target)
        .as_deref()
        .unwrap_or(&*DEFAULT_FIREWALL)
        .get_sync(&channel)
        && whitelist.permits(me)
    {
        Ok(())
    } else {
        Err(anyhow::anyhow!("{channel:?} blocked by firewall"))
    }
}
