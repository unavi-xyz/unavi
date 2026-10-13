use bevy::prelude::*;
use hsd::attributes::portal::PortalAttr;

use crate::attributes::apply_simple;

/// Longest side of a portal, in metres.
const MAX_SIDE: f64 = 64.0;

/// A prim's portal destination and size.
#[derive(Component, Debug, Clone, Copy)]
pub struct PortalConfig(pub PortalAttr);

pub(crate) fn apply(
    commands: &mut Commands,
    prim: Entity,
    payload: Option<&[u8]>,
) -> Result<(), postcard::Error> {
    apply_simple::<PortalAttr, PortalConfig>(commands, prim, payload, admitted)
}

/// A portal whose sides are not each finite, positive and within
/// [`MAX_SIDE`] has no geometry to draw, and is refused.
fn admitted(attr: PortalAttr) -> Option<PortalConfig> {
    let bounded = |side: f64| side.is_finite() && side > 0.0 && side <= MAX_SIDE;
    (bounded(attr.size_x) && bounded(attr.size_y)).then_some(PortalConfig(attr))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sized(side: f64) -> PortalAttr {
        PortalAttr {
            size_x: side,
            size_y: 1.0,
            ..default()
        }
    }

    #[test]
    fn a_portal_without_drawable_sides_is_refused() {
        assert!(admitted(sized(2.0)).is_some());
        for side in [f64::NAN, f64::INFINITY, 0.0, -1.0, MAX_SIDE * 2.0] {
            assert!(
                admitted(sized(side)).is_none(),
                "side {side} must be refused"
            );
        }
    }
}
