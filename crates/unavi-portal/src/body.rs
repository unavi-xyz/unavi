//! Bodies that cross portals, and the state that crossing and echoing need
//! on them.

use bevy::prelude::*;

/// A body that teleports when it crosses a [`Portal`](crate::portal::Portal)
/// plane, and casts an echo while straddling one.
#[derive(Component)]
#[require(PortalLatch, PrevTranslation, EchoBody)]
pub struct PortalBody;

/// Casts portal echoes but is never locally teleported across a portal.
/// Bodies whose crossings are driven externally, such as network-replicated
/// avatars, carry it alone.
#[derive(Component, Default)]
#[require(EchoRadius)]
pub struct EchoBody;

/// Marker for the camera whose position drives portal view-budget selection.
#[derive(Component)]
pub struct PortalViewer;

/// Whether a body is still within some portal's landing slab after a
/// crossing. Held until the body clears every slab, so the slab it just
/// landed in does not immediately re-fire.
#[derive(Component, Default)]
pub struct PortalLatch(pub bool);

/// The body's translation last frame.
///
/// Seeded on the first sighting so a body spawned exactly at the origin is
/// not mistaken for an unset sentinel (a `Vec3::ZERO` sentinel would read
/// that as a false crossing).
#[derive(Component, Default)]
pub struct PrevTranslation(pub Option<Vec3>);

/// Cached bounding-sphere radius of a body's subtree around its own origin.
///
/// Recomputed by [`crate::echo::update_echo_radius`] when any entity in the
/// body's subtree gains or changes an
/// [`Aabb`](bevy::camera::primitives::Aabb), or gains or loses children, so
/// [`crate::echo::maintain_echoes`] never walks the whole hierarchy every
/// frame. A descendant that disappears by despawning, instead of just
/// losing its `Aabb` while it stays, is not observed; the radius then lags
/// until some other change touches the subtree.
#[derive(Component, Default, Clone, Copy)]
pub struct EchoRadius(pub f32);
