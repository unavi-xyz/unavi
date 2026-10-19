//! A portal crossing by the local agent, recorded for the avatar stream.

use bevy::prelude::*;
use postcard::experimental::max_size::MaxSize;
use serde::{
    Deserialize,
    Serialize,
};
use unavi_agent::{
    LocalAgent,
    LocalAgentEntities,
};
use unavi_portal::{
    crossing::Crossed,
    portal::Portal,
};

use crate::{
    link::codec::{
        self,
        f32_vec3::F32Vec3,
    },
    membership::{
        Space,
        SpaceId,
    },
};

/// A point on a portal's plane, in its space's frame.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, MaxSize)]
pub struct CrossingPoint {
    pub space: SpaceId,
    pub point: F32Vec3,
}

impl CrossingPoint {
    /// The point in its space's frame, if finite and inside the space's cell.
    #[must_use]
    pub fn checked(self) -> Option<(SpaceId, Vec3)> {
        codec::position(self.point.into()).map(|point| (self.space, point))
    }
}

/// Where a crossing left the source space and entered the destination. A
/// one-way crossing has no destination point.
#[derive(Clone, Copy, Debug)]
pub struct Crossing {
    pub from: CrossingPoint,
    pub to:   Option<CrossingPoint>,
}

/// A peer's [`Crossing`], checked, with points in their spaces' frames.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReceivedCrossing {
    pub from: (SpaceId, Vec3),
    pub to:   Option<(SpaceId, Vec3)>,
}

/// A [`Crossing`] and its place in the order of crossings.
#[derive(Clone, Copy, Debug)]
pub struct NumberedCrossing {
    pub seq:      u32,
    pub crossing: Crossing,
}

/// The local agent's latest crossing.
#[derive(Resource, Default)]
pub struct LastCrossing(Option<NumberedCrossing>);

impl LastCrossing {
    #[must_use]
    pub const fn latest(&self) -> Option<NumberedCrossing> {
        self.0
    }

    fn record(&mut self, crossing: Crossing) {
        let seq = self.0.map_or(0, |last| last.seq.wrapping_add(1));
        self.0 = Some(NumberedCrossing { seq, crossing });
    }
}

/// Decides when a stream writes the latest crossing: once per crossing, and
/// never one that happened before the stream's first pose.
#[derive(Default)]
pub struct CrossingGate {
    seeded:  bool,
    written: Option<u32>,
}

impl CrossingGate {
    pub fn due(&mut self, latest: Option<NumberedCrossing>) -> Option<Crossing> {
        let seq = latest.map(|n| n.seq);
        if !self.seeded {
            self.seeded = true;
            self.written = seq;
            return None;
        }
        if seq == self.written {
            return None;
        }
        self.written = seq;
        latest.map(|n| n.crossing)
    }
}

/// Records the local body's crossings, with each point in its portal's space.
pub fn record_crossing(
    event: On<Crossed>,
    agent: Query<&LocalAgentEntities, With<LocalAgent>>,
    portals: Query<&GlobalTransform, With<Portal>>,
    spaces: Query<(&Space, &GlobalTransform)>,
    parents: Query<&ChildOf>,
    mut last: ResMut<LastCrossing>,
) {
    let Ok(entities) = agent.single() else {
        return;
    };
    if entities.body != event.entity {
        return;
    }
    let point_in_space = |portal: Entity, point: Vec2| {
        let world = portals
            .get(portal)
            .ok()?
            .transform_point(point.extend(0.0));
        let owner = parents
            .iter_ancestors(portal)
            .find(|ancestor| spaces.contains(*ancestor))?;
        let (space, transform) = spaces.get(owner).ok()?;
        Some(CrossingPoint {
            space: space.id(),
            point: transform.affine().inverse().transform_point3(world).into(),
        })
    };

    let Some(from) = point_in_space(event.portal, event.point) else {
        return;
    };
    let to = portals
        .contains(event.destination)
        .then(|| {
            // The half-turn gluing two faces mirrors plane-local x.
            point_in_space(event.destination, Vec2::new(-event.point.x, event.point.y))
        })
        .flatten();
    last.record(Crossing { from, to });
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: SpaceId = SpaceId([1; 32]);
    const B: SpaceId = SpaceId([2; 32]);

    fn point(space: SpaceId, x: f32) -> CrossingPoint {
        CrossingPoint {
            space,
            point: Vec3::new(x, 0.0, 0.0).into(),
        }
    }

    fn crossing(x: f32) -> Crossing {
        Crossing {
            from: point(A, x),
            to:   None,
        }
    }

    #[test]
    fn a_crossing_before_the_first_pose_is_never_written() {
        let mut last = LastCrossing::default();
        last.record(crossing(1.0));

        let mut gate = CrossingGate::default();
        assert!(gate.due(last.latest()).is_none());
        assert!(gate.due(last.latest()).is_none());
    }

    #[test]
    fn each_later_crossing_is_written_once() {
        let mut last = LastCrossing::default();
        let mut gate = CrossingGate::default();
        assert!(gate.due(last.latest()).is_none());

        last.record(crossing(1.0));
        let written = gate.due(last.latest()).expect("written");
        assert_eq!(A, written.from.space);
        assert!(gate.due(last.latest()).is_none());

        last.record(Crossing {
            from: point(B, 2.0),
            to:   Some(point(A, -2.0)),
        });
        let written = gate.due(last.latest()).expect("written");
        assert_eq!(B, written.from.space);
        assert!(gate.due(last.latest()).is_none());
    }

    #[test]
    fn a_stream_started_with_no_crossing_writes_the_first() {
        let mut last = LastCrossing::default();
        let mut gate = CrossingGate::default();
        assert!(gate.due(None).is_none());

        last.record(crossing(1.0));
        assert!(gate.due(last.latest()).is_some());
    }

    #[test]
    fn a_point_outside_the_cell_is_rejected() {
        assert!(point(A, 1.0).checked().is_some());
        assert!(point(A, f32::NAN).checked().is_none());
        assert!(point(A, f32::MAX).checked().is_none());
    }
}
