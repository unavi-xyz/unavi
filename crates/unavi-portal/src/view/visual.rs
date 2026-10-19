//! What a portal's surface shows, and keeping its material and cameras in
//! step with it.

use bevy::prelude::*;

use crate::{
    destination::Destination,
    portal::{
        Portal,
        PortalState,
    },
    view::{
        Viewed,
        live::{
            despawn_live,
            install_live,
        },
        material::PortalMaterial,
        still::StillCapture,
    },
};

#[cfg(test)] mod tests;

/// The portal's surface. Every visible state draws the rim.
#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalVisual {
    /// Closed. Nothing is drawn.
    #[default]
    Hidden,
    /// A fill rising behind the veil, while the destination loads or its
    /// still is taken.
    Loading,
    /// The live view through to the link partner.
    Live,
    /// The arrival still behind the veil, for a one-way portal.
    Still,
    /// The veil alone, for a linked portal outside the view budget.
    Veil,
}

impl PortalVisual {
    /// The `mode` uniform `portal.wgsl` branches on.
    #[must_use]
    pub const fn mode(self) -> u32 {
        match self {
            Self::Hidden | Self::Live => 0,
            Self::Still => 1,
            Self::Loading => 2,
            Self::Veil => 3,
        }
    }
}

/// The [`PortalVisual`] last installed on a portal. Written only once the
/// install succeeds, so a frame without a camera to mirror retries.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct AppliedVisual(PortalVisual);

pub fn choose_visual(
    mut portals: Query<
        (
            &PortalState,
            Has<Viewed>,
            Option<&Destination>,
            Option<&StillCapture>,
            &mut PortalVisual,
        ),
        With<Portal>,
    >,
    linked: Query<(), With<Portal>>,
    time: Res<Time>,
) {
    let now = time.elapsed_secs();
    for (state, viewed, destination, still, mut visual) in &mut portals {
        let one_way = destination.is_some_and(|d| !linked.contains(d.0));
        let next = match state {
            PortalState::Closed => PortalVisual::Hidden,
            PortalState::Loading => PortalVisual::Loading,
            PortalState::Open if viewed => PortalVisual::Live,
            PortalState::Open if one_way => match still {
                Some(still) if still.is_ready(now) => PortalVisual::Still,
                _ => PortalVisual::Loading,
            },
            PortalState::Open => PortalVisual::Veil,
        };
        visual.set_if_neq(next);
    }
}

pub fn apply_visual(
    portals: Query<
        (
            Entity,
            &PortalVisual,
            Option<&AppliedVisual>,
            Option<&MeshMaterial3d<PortalMaterial>>,
        ),
        With<Portal>,
    >,
    mut materials: ResMut<Assets<PortalMaterial>>,
    mut commands: Commands,
) {
    for (portal, visual, applied, material) in &portals {
        let visual = *visual;
        if applied.is_some_and(|a| a.0 == visual) {
            continue;
        }

        if visual == PortalVisual::Hidden {
            commands
                .entity(portal)
                .insert((Visibility::Hidden, AppliedVisual(visual)));
            commands.queue(move |world: &mut World| despawn_live(world, portal));
            continue;
        }

        if material.is_none() {
            let handle = materials.add(PortalMaterial::default());
            commands.entity(portal).insert(MeshMaterial3d(handle));
        }
        commands.queue(move |world: &mut World| install(world, portal, visual));
    }
}

fn install(world: &mut World, portal: Entity, visual: PortalVisual) {
    if world.get_entity(portal).is_err() {
        return;
    }
    if visual == PortalVisual::Live {
        if !install_live(world, portal) {
            return;
        }
    } else {
        despawn_live(world, portal);
    }

    let still = world
        .get::<StillCapture>(portal)
        .filter(|_| visual == PortalVisual::Still)
        .map(|still| still.atlas.clone());
    let material = world
        .get::<MeshMaterial3d<PortalMaterial>>(portal)
        .map(|m| m.0.id());
    if let Some(id) = material
        && let Some(mut material) = world.resource_mut::<Assets<PortalMaterial>>().get_mut(id)
    {
        material.still = still;
    }

    world
        .entity_mut(portal)
        .insert((Visibility::Inherited, AppliedVisual(visual)));
}
