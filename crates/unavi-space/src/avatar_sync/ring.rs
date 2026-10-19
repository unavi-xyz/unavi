//! Rings on the portals a remote peer crossed, which poses alone cannot show
//! because a crossing moves the body and its space in one step.

use bevy::prelude::*;
use unavi_portal::{
    portal::{
        Portal,
        PortalFrame,
        PortalState,
    },
    view::ring::start_ring,
};

use crate::{
    discovery::{
        LastSeenIn,
        Peer,
    },
    index::Index,
    link::PeerLink,
    membership::Space,
};

/// The open portal whose slab holds `world`, and the point on its plane.
fn portal_at<'a>(
    world: Vec3,
    portals: impl IntoIterator<Item = (Entity, &'a PortalFrame, &'a PortalState)>,
) -> Option<(Entity, Vec2)> {
    portals.into_iter().find_map(|(entity, frame, state)| {
        if *state != PortalState::Open {
            return None;
        }
        frame.slab_point(world).map(|point| (entity, point))
    })
}

/// Rings each portal a peer reports crossing, on whichever side of the
/// crossing is present locally.
pub fn ring_remote_crossings(
    time: Res<Time>,
    link: Option<Res<PeerLink>>,
    space_index: Res<Index<Space>>,
    peer_index: Res<Index<Peer>>,
    peers: Query<&LastSeenIn, With<Peer>>,
    spaces: Query<&GlobalTransform, With<Space>>,
    portals: Query<(Entity, &PortalFrame, &PortalState), With<Portal>>,
    mut commands: Commands,
) {
    let Some(link) = link else {
        return;
    };
    let start = time.elapsed_secs_wrapped();

    for (peer, crossing) in link.crossings().drain() {
        let Some(seen) = peer_index.get(peer).and_then(|e| peers.get(e).ok()) else {
            continue;
        };

        for (space, point) in std::iter::once(crossing.from).chain(crossing.to) {
            if !seen.contains(space) {
                continue;
            }
            let Some(transform) = space_index.get(space).and_then(|e| spaces.get(e).ok()) else {
                continue;
            };
            let world = transform.transform_point(point);
            if let Some((portal, plane)) = portal_at(world, portals.iter()) {
                start_ring(&mut commands, portal, plane, start);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::Affine3A;

    use super::*;

    fn frame_at(position: Vec3) -> PortalFrame {
        PortalFrame {
            local_from_world: Affine3A::from_translation(-position),
            half_size:        Vec2::new(1.0, 1.0),
            bounding_radius:  1.5,
        }
    }

    #[test]
    fn a_point_on_an_open_portal_maps_to_its_plane() {
        let portal = Entity::PLACEHOLDER;
        let frame = frame_at(Vec3::new(10.0, 1.0, 0.0));

        let (found, plane) = portal_at(
            Vec3::new(10.5, 1.25, 0.0),
            [(portal, &frame, &PortalState::Open)],
        )
        .expect("portal");

        assert_eq!(portal, found);
        assert!((plane - Vec2::new(0.5, 0.25)).length() < 1e-5);
    }

    #[test]
    fn a_closed_portal_or_a_point_off_its_slab_is_skipped() {
        let portal = Entity::PLACEHOLDER;
        let frame = frame_at(Vec3::ZERO);

        assert!(portal_at(Vec3::ZERO, [(portal, &frame, &PortalState::Closed)]).is_none());
        assert!(portal_at(Vec3::new(0.0, 0.0, 0.5), [(portal, &frame, &PortalState::Open)]).is_none());
        assert!(portal_at(Vec3::new(2.0, 0.0, 0.0), [(portal, &frame, &PortalState::Open)]).is_none());
    }
}
