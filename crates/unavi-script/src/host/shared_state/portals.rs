//! The portals in loaded spaces, for scripts to read without waiting on the
//! world.

use std::sync::Arc;

use bevy::prelude::*;
use bevy_hsd::{
    document::HsdDocId,
    prim::{
        Prim,
        PrimOf,
    },
};
use hsd::id::{
    DocId,
    PrimId,
};
use parking_lot::RwLock;
use unavi_portal::{
    destination::Destination,
    portal::{
        Portal,
        PortalLink,
        PortalSize,
        PortalState,
        PortalTargetDoc,
    },
    view::hue::PortalHue,
};

use crate::host::shared_state::near::nearest;

/// A portal as the engine placed it this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PortalSnapshot {
    pub doc:         DocId,
    pub prim:        PrimId,
    /// In the engine world.
    pub world:       GlobalTransform,
    pub width:       f32,
    pub height:      f32,
    pub destination: Option<DocId>,
    pub linked:      bool,
    pub state:       PortalState,
    pub hue:         LinearRgba,
}

/// Every portal in a loaded space.
#[derive(Resource, Clone, Default)]
pub struct Portals(Arc<RwLock<Vec<PortalSnapshot>>>);

impl Portals {
    /// Up to [`super::near::MAX_NEAR`] portals whose centre lies within
    /// `radius` of `centre` in the engine world, nearest first.
    #[must_use]
    pub fn near(&self, centre: Vec3, radius: f32) -> Vec<PortalSnapshot> {
        nearest(&self.0.read(), centre, radius, |portal| {
            portal.world.translation()
        })
    }
}

/// Copies every portal out of the world.
pub fn snapshot_portals(
    portals: Query<
        (
            &Prim,
            &PrimOf,
            &GlobalTransform,
            &PortalSize,
            &PortalState,
            Option<&PortalTargetDoc>,
            Option<&PortalLink>,
            Option<&Destination>,
            Option<&PortalHue>,
        ),
        With<Portal>,
    >,
    partners: Query<(), With<Portal>>,
    docs: Query<&HsdDocId>,
    shared: Res<Portals>,
) {
    let mut found: Vec<PortalSnapshot> = portals
        .iter()
        .filter_map(
            |(prim, of, world, size, state, target, link, resolved, hue)| {
                Some(PortalSnapshot {
                    doc:         docs.get(of.0).ok()?.0,
                    prim:        prim.0,
                    world:       *world,
                    width:       size.width,
                    height:      size.height,
                    destination: target.map(|target| target.0),
                    // A portal opening onto another portal is half of a pair;
                    // until the destination resolves, a stated link is the best
                    // answer.
                    linked:      resolved.map_or_else(
                        || link.is_some(),
                        |destination| partners.contains(destination.0),
                    ),
                    state:       *state,
                    hue:         hue.map_or(PortalHue::NEUTRAL, |hue| hue.0),
                })
            },
        )
        .collect();
    // A stable order, so an unchanged scene compares equal and writes
    // nothing.
    found.sort_by_key(|portal| (portal.doc.0, portal.prim.0));
    if *shared.0.read() != found {
        *shared.0.write() = found;
    }
}

#[cfg(test)]
mod tests {
    use hsd::attributes::portal::LinkId;

    use super::*;

    #[test]
    fn a_portal_in_a_loaded_document_is_snapshotted() {
        let mut app = App::new();
        app.init_resource::<Portals>()
            .add_systems(Update, snapshot_portals);
        let shared = app.world().resource::<Portals>().clone();

        let doc_id = DocId([3; 32]);
        let doc = app.world_mut().spawn(HsdDocId(doc_id)).id();
        let target = DocId([9; 32]);
        let portal = app
            .world_mut()
            .spawn((
                Prim(PrimId([4; 16])),
                PrimOf(doc),
                Portal,
                PortalSize {
                    width:  2.0,
                    height: 3.0,
                },
                PortalState::Open,
                PortalTargetDoc(target),
                GlobalTransform::from_xyz(1.0, 2.0, 3.0),
            ))
            .id();
        app.update();

        let found = shared.near(Vec3::new(1.0, 2.0, 3.0), 1.0);
        assert_eq!(found.len(), 1);
        let found = found[0];
        assert_eq!((found.doc, found.prim), (doc_id, PrimId([4; 16])));
        assert_eq!((found.width, found.height), (2.0, 3.0));
        assert_eq!(found.destination, Some(target));
        assert!(!found.linked, "no link partner is stated");
        assert_eq!(found.state, PortalState::Open);
        assert_eq!(found.hue, PortalHue::NEUTRAL, "no hue yet reads neutral");

        let rim = LinearRgba::rgb(0.2, 0.4, 0.6);
        app.world_mut().entity_mut(portal).insert(PortalHue(rim));
        app.update();
        assert_eq!(shared.near(Vec3::ZERO, 100.0)[0].hue, rim);

        app.world_mut()
            .entity_mut(portal)
            .insert(PortalLink(LinkId([1; 16])));
        app.update();
        assert!(
            shared.near(Vec3::ZERO, 100.0)[0].linked,
            "a stated link counts before it resolves"
        );

        app.world_mut().entity_mut(portal).insert(Destination(doc));
        app.update();
        assert!(
            !shared.near(Vec3::ZERO, 100.0)[0].linked,
            "a document root is no partner"
        );

        let other = app.world_mut().spawn(Portal).id();
        app.world_mut()
            .entity_mut(portal)
            .insert(Destination(other));
        app.update();
        assert!(
            shared.near(Vec3::ZERO, 100.0)[0].linked,
            "a portal is a partner"
        );

        app.world_mut().entity_mut(portal).despawn();
        app.update();
        assert_eq!(shared.near(Vec3::ZERO, 100.0), [], "the portal is gone");
    }
}
