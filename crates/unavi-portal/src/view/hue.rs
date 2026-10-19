//! A portal's rim colour: the hue of the space it opens onto.

use bevy::prelude::*;
use bevy_hsd::{
    attributes::sky::Sky,
    document::HsdDocId,
    hierarchy::ancestors,
    prim::PrimOf,
};
use hsd::id::DocId;

use crate::{
    destination::Destination,
    portal::Portal,
};

const RIM_LIGHTNESS: f32 = 0.78;
const RIM_CHROMA: f32 = 0.10;
/// Below this a sky reads as grey and has no hue worth keeping.
const ACHROMATIC: f32 = 0.004;

/// The rim colour of a portal, in linear RGB.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct PortalHue(pub LinearRgba);

impl PortalHue {
    pub const NEUTRAL: LinearRgba = LinearRgba::rgb(0.7, 0.7, 0.7);

    /// The sky's hue, else the space's identity hue, at the rim's own
    /// lightness and chroma. A sky's chroma is capped low, so taking it as
    /// given would leave every rim grey.
    #[must_use]
    pub fn of_space(sky: Option<Color>, space: Option<DocId>) -> Self {
        let hue = sky
            .map(Oklcha::from)
            .filter(|lch| lch.chroma >= ACHROMATIC)
            .or_else(|| space.map(|id| Oklcha::from(identity_color(id))))
            .map(|lch| lch.hue);
        hue.map_or(Self(Self::NEUTRAL), |hue| {
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

/// Sets each portal's [`PortalHue`] from the space its destination stands in:
/// the [`Sky`] on its root document, else its id.
pub fn update_hues(
    mut portals: Query<(Entity, &Destination, Option<&mut PortalHue>), With<Portal>>,
    skies: Query<(&Sky, &PrimOf)>,
    docs: Query<&HsdDocId>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) {
    for (portal, destination, current) in &mut portals {
        let space = ancestors(destination.0, &parents)
            .filter(|e| docs.contains(*e))
            .last();
        let id = space.and_then(|space| docs.get(space).ok()).map(|id| id.0);
        let sky = space.and_then(|space| {
            skies
                .iter()
                .find(|(_, prim_of)| prim_of.0 == space)
                .map(|(sky, _)| sky.0)
        });
        let next = PortalHue::of_space(sky, id);
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

    use super::PortalHue;

    #[test]
    fn a_rim_keeps_the_sky_hue_and_ignores_its_chroma() {
        let faint = Oklcha::new(0.4, 0.03, 200.0, 1.0);
        let strong = Oklcha::new(0.7, 0.2, 200.0, 1.0);

        let a = Oklcha::from(PortalHue::of_space(Some(faint.into()), None).0);
        let b = Oklcha::from(PortalHue::of_space(Some(strong.into()), None).0);

        assert!((a.hue - 200.0).abs() < 1.0, "hue drifted to {}", a.hue);
        assert!((a.chroma - b.chroma).abs() < 1.0e-3);
        assert!((a.lightness - b.lightness).abs() < 1.0e-3);
    }

    #[test]
    fn a_space_without_a_hued_sky_takes_its_identity_hue() {
        let grey = Color::srgb(0.5, 0.5, 0.5);
        let space = DocId([7; 32]);
        let skyless = PortalHue::of_space(None, Some(space));
        assert_eq!(PortalHue::of_space(Some(grey), Some(space)), skyless);
        assert!(
            Oklcha::from(skyless.0).chroma > 0.05,
            "identity rim is saturated"
        );
        assert_eq!(PortalHue::of_space(Some(grey), None).0, PortalHue::NEUTRAL);
    }
}
