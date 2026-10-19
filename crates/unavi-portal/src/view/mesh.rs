//! The portal plane mesh, and the open/loading/closed state it is drawn in.

use bevy::{
    camera::visibility::RenderLayers,
    prelude::*,
};

use crate::{
    destination::Destination,
    portal::{
        Portal,
        PortalSize,
        PortalState,
    },
    render_layers::PORTAL_RENDER_LAYER,
};

#[derive(Component)]
pub struct CachedSize(pub PortalSize);

pub fn ensure_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    portals: Query<
        (Entity, &PortalSize, Option<&CachedSize>),
        (With<Portal>, Or<(Changed<PortalSize>, Without<CachedSize>)>),
    >,
) {
    for (entity, size, cached) in &portals {
        if cached.is_some_and(|c| c.0 == *size) {
            continue;
        }
        let mesh = Plane3d::default()
            .mesh()
            .normal(Dir3::Z)
            .size(size.width, size.height)
            .build();
        commands.entity(entity).insert((
            Mesh3d(meshes.add(mesh)),
            RenderLayers::layer(PORTAL_RENDER_LAYER),
            CachedSize(*size),
        ));
    }
}

pub fn update_state(
    mut portals: Query<(&mut PortalState, Option<&Destination>), With<Portal>>,
    incoming: Query<(), With<crate::destination::Arrivals>>,
    doc_roots: Query<
        (),
        (
            With<bevy_hsd::document::Hsd>,
            Without<bevy_hsd::document::Unplaced>,
        ),
    >,
) {
    for (mut state, dest) in &mut portals {
        let next = match dest {
            None => PortalState::Closed,
            Some(d) if incoming.contains(d.0) || doc_roots.contains(d.0) => PortalState::Open,
            Some(_) => PortalState::Loading,
        };
        state.set_if_neq(next);
    }
}
