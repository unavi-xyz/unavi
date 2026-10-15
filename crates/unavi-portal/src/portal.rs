//! The portal itself: a plane glued to a destination, and the cached frame
//! crossing and echo systems read it through.

use bevy::{
    math::Affine3A,
    prelude::*,
};
use hsd::{
    attributes::portal::LinkId,
    id::DocId,
};

/// A plane that bodies cross and echoes/live views render through.
#[derive(Component, Default)]
#[require(PortalState, PortalSize, PortalFrame)]
pub struct Portal;

#[derive(Component, Clone, Copy, PartialEq)]
pub struct PortalSize {
    pub width:  f32,
    pub height: f32,
}

impl Default for PortalSize {
    fn default() -> Self {
        Self {
            width:  1.0,
            height: 1.0,
        }
    }
}

/// Depth of the latch slab around the portal plane; suppresses an immediate
/// re-crossing after landing.
pub const PORTAL_DEPTH: f32 = 0.05;

/// The document a portal without a link partner opens onto.
#[derive(Component, Clone, Copy)]
pub struct PortalTargetDoc(pub DocId);

/// The link pairing this portal with a portal in its target space.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct PortalLink(pub LinkId);

/// The space this portal stands in, which its link partner must target.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct PortalHome(pub DocId);

#[derive(Component, Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum PortalState {
    #[default]
    Closed,
    Loading,
    Open,
}

/// Caps how many prims a scene can promote to [`Portal`]s, enforced in
/// [`crate::config::sync_portal_config`], the single place a portal is
/// spawned from scene data.
#[derive(Resource, Clone, Copy)]
pub struct PortalLimits {
    pub max_portals: usize,
}

impl Default for PortalLimits {
    fn default() -> Self {
        Self { max_portals: 64 }
    }
}

/// A portal's world-to-local affine and footprint, recomputed only when its
/// transform or size changes so crossing and echo systems never invert a
/// matrix per body per frame.
///
/// [`update_portal_frames`] runs before Bevy's transform propagation, so it
/// reads the portal's `GlobalTransform` from the end of the *previous*
/// frame; a portal that itself moved this frame (e.g. carried by
/// `unavi-space`'s grid recenter) is one frame stale here. Portals are not
/// expected to move often, so this is left as-is rather than reordered.
#[derive(Component, Clone, Copy)]
pub struct PortalFrame {
    pub local_from_world: Affine3A,
    pub half_size:        Vec2,
    /// Radius of the smallest sphere around the portal's centre that
    /// contains its plane; a cheap distance check before the inverse
    /// transform above.
    pub bounding_radius:  f32,
}

impl Default for PortalFrame {
    fn default() -> Self {
        Self {
            local_from_world: Affine3A::IDENTITY,
            half_size:        Vec2::ZERO,
            bounding_radius:  0.0,
        }
    }
}

pub(crate) fn update_portal_frames(
    mut portals: Query<
        (&GlobalTransform, &PortalSize, &mut PortalFrame),
        (
            With<Portal>,
            Or<(Changed<GlobalTransform>, Changed<PortalSize>)>,
        ),
    >,
) {
    for (transform, size, mut frame) in &mut portals {
        *frame = PortalFrame {
            local_from_world: transform.affine().inverse(),
            half_size:        Vec2::new(size.width / 2.0, size.height / 2.0),
            bounding_radius:  Vec2::new(size.width, size.height).length() / 2.0,
        };
    }
}

/// Affine map carrying poses through a portal: into the source portal's
/// space, through the half-turn aligning the two faces, then out at the
/// destination.
#[must_use]
pub fn portal_transfer(source: &GlobalTransform, destination: &GlobalTransform) -> Affine3A {
    let flip = Affine3A::from_quat(Quat::from_rotation_y(std::f32::consts::PI));
    destination.affine() * flip * source.affine().inverse()
}
