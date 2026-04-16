#![expect(unused)]

use std::time::Duration;

use bevy::prelude::*;

use crate::networking::object::publish::Grabbed;

/// Pin lifetime for an entity with a state-driven lifetime.
///
/// - Our own spawned objects: `Permanent`
/// - Foreign objects (from peer state): `Timed` with `FOREIGN_TTL`, refreshed
///   on grab or collision with a locally-owned object
#[derive(Component, Clone, Debug)]
pub enum ObjectPin {
    Permanent,
    Timed { expires_at: Duration },
}

impl ObjectPin {
    pub const FOREIGN_TTL: Duration = Duration::from_mins(5);
    pub const EXTENDED_TTL: Duration = Duration::from_mins(30);

    pub fn foreign(now: Duration) -> Self {
        Self::Timed {
            expires_at: now + Self::FOREIGN_TTL,
        }
    }

    pub fn refresh(&mut self, now: Duration) {
        if let Self::Timed { expires_at } = self {
            *expires_at = now + Self::EXTENDED_TTL;
        }
    }

    pub fn expired(&self, now: Duration) -> bool {
        match self {
            Self::Permanent => false,
            Self::Timed { expires_at } => now >= *expires_at,
        }
    }
}

/// Despawn entities whose `Timed` pin has expired.
pub fn tick_pins(mut commands: Commands, time: Res<Time>, pins: Query<(Entity, &ObjectPin)>) {
    let now = time.elapsed();
    for (entity, pin) in &pins {
        if pin.expired(now) {
            commands.entity(entity).despawn();
        }
    }
}

/// Refresh pin TTL when an entity gains the `Grabbed` component.
pub fn refresh_pins_on_grab(
    time: Res<Time>,
    mut pins: Query<&mut ObjectPin>,
    newly_grabbed: Query<Entity, Added<Grabbed>>,
) {
    let now = time.elapsed();
    for entity in &newly_grabbed {
        if let Ok(mut pin) = pins.get_mut(entity) {
            pin.refresh(now);
        }
    }
}
