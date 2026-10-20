//! A portal's rim colour: the identity hue of the space it opens onto.

use bevy::prelude::*;
use bevy_hsd::{
    document::HsdDocId,
    hierarchy::ancestors,
};
use hsd::id::DocId;

use crate::{
    destination::Destination,
    portal::Portal,
};

const RIM_LIGHTNESS: f32 = 0.78;
const RIM_CHROMA: f32 = 0.10;

/// The rim colour of a portal, in linear RGB.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct PortalHue(pub LinearRgba);

impl PortalHue {
    pub const NEUTRAL: LinearRgba = LinearRgba::rgb(0.7, 0.7, 0.7);

    /// The space's identity hue at the rim's own lightness and chroma.
    /// Neutral when the space is unknown.
    #[must_use]
    pub fn of_space(space: Option<DocId>) -> Self {
        space.map_or(Self(Self::NEUTRAL), |id| {
            let hue = Oklcha::from(identity_color(id)).hue;
            Self(Oklcha::new(RIM_LIGHTNESS, RIM_CHROMA, hue, 1.0).into())
        })
    }
}

/// The colour a space's beacon and halo mote wear, from the hue its id's
/// first eight bytes pick. Saturation and value are fixed, since only the hue
/// survives into the rim.
fn identity_color(space: DocId) -> Color {
    let [b0, b1, b2, b3, b4, b5, b6, b7, ..] = space.0;
    let word = u64::from_le_bytes([b0, b1, b2, b3, b4, b5, b6, b7]);
    #[expect(
        clippy::cast_precision_loss,
        reason = "a hue needs far fewer than 53 bits"
    )]
    let turn = (word as f64 / u64::MAX as f64) as f32;
    Color::hsv(turn * 360.0, 0.8, 0.9)
}

/// Sets each portal's [`PortalHue`] from the id of the space its destination
/// stands in.
pub fn update_hues(
    mut portals: Query<(Entity, &Destination, Option<&mut PortalHue>), With<Portal>>,
    docs: Query<&HsdDocId>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) {
    for (portal, destination, current) in &mut portals {
        let space = ancestors(destination.0, &parents)
            .filter(|e| docs.contains(*e))
            .last();
        let id = space.and_then(|space| docs.get(space).ok()).map(|id| id.0);
        let next = PortalHue::of_space(id);
        match current {
            Some(mut current) => {
                current.set_if_neq(next);
            }
            None => {
                commands.entity(portal).insert(next);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use hsd::id::DocId;

    use super::{
        PortalHue,
        RIM_CHROMA,
        identity_color,
    };

    #[test]
    fn a_rim_wears_its_spaces_identity_hue_at_a_fixed_chroma() {
        let rim = PortalHue::of_space(Some(DocId([7; 32])));
        let identity = Oklcha::from(identity_color(DocId([7; 32])));
        let lch = Oklcha::from(rim.0);

        assert!((lch.hue - identity.hue).abs() < 1.0, "hue drifted to {}", lch.hue);
        assert!((lch.chroma - RIM_CHROMA).abs() < 1.0e-2);
        assert_eq!(PortalHue::of_space(None).0, PortalHue::NEUTRAL);
    }
}
