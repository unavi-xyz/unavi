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
};

/// The most portals one query answers.
pub const MAX_NEAR: usize = 64;

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
}

/// Every portal in a loaded space.
#[derive(Resource, Clone, Default)]
pub struct Portals(Arc<RwLock<Vec<PortalSnapshot>>>);

impl Portals {
    /// Up to [`MAX_NEAR`] portals whose centre lies within `radius` of
    /// `centre` in the engine world, nearest first.
    #[must_use]
    pub fn near(&self, centre: Vec3, radius: f32) -> Vec<PortalSnapshot> {
        nearest(&self.0.read(), centre, radius)
    }
}

fn nearest(portals: &[PortalSnapshot], centre: Vec3, radius: f32) -> Vec<PortalSnapshot> {
    let mut found: Vec<(f32, PortalSnapshot)> = portals
        .iter()
        .map(|portal| (portal.world.translation().distance(centre), *portal))
        .filter(|(distance, _)| *distance <= radius)
        .collect();
    found.sort_by(|a, b| a.0.total_cmp(&b.0));
    found.truncate(MAX_NEAR);
    found.into_iter().map(|(_, portal)| portal).collect()
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
        ),
        With<Portal>,
    >,
    partners: Query<(), With<Portal>>,
    docs: Query<&HsdDocId>,
    shared: Res<Portals>,
) {
    let mut found: Vec<PortalSnapshot> = portals
        .iter()
        .filter_map(|(prim, of, world, size, state, target, link, resolved)| {
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
            })
        })
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

    fn portal_at(x: f32) -> PortalSnapshot {
        PortalSnapshot {
            doc:         DocId([0; 32]),
            prim:        PrimId([x as u8; 16]),
            world:       GlobalTransform::from_xyz(x, 0.0, 0.0),
            width:       1.0,
            height:      2.0,
            destination: None,
            linked:      false,
            state:       PortalState::Closed,
        }
    }

    fn xs(portals: &[PortalSnapshot]) -> Vec<f32> {
        portals
            .iter()
            .map(|portal| portal.world.translation().x)
            .collect()
    }

    #[test]
    fn near_keeps_what_is_in_range_nearest_first() {
        let all = [
            portal_at(5.0),
            portal_at(1.0),
            portal_at(20.0),
            portal_at(3.0),
        ];
        let found = nearest(&all, Vec3::ZERO, 5.0);
        assert_eq!(xs(&found), [1.0, 3.0, 5.0], "the edge is in range");
    }

    #[test]
    fn near_is_measured_from_the_centre_given() {
        let all = [portal_at(5.0), portal_at(1.0)];
        let found = nearest(&all, Vec3::new(6.0, 0.0, 0.0), 2.0);
        assert_eq!(xs(&found), [5.0]);
    }

    #[test]
    fn near_stops_at_the_cap_keeping_the_nearest() {
        let all: Vec<_> = (0..100).rev().map(|i| portal_at(i as f32)).collect();
        let found = nearest(&all, Vec3::ZERO, 1000.0);
        assert_eq!(found.len(), MAX_NEAR);
        assert_eq!(found[0].world.translation().x, 0.0);
        assert_eq!(found[MAX_NEAR - 1].world.translation().x, 63.0);
    }

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
