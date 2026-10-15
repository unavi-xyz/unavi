//! Fitting an icon's real geometry into the shell that draws it.

use wired_guest::math::{
    Quat,
    Transform,
    Vec3,
};

use crate::wired::scene::{
    document::Document,
    properties::{
        Property,
        PropertyKey,
        VertexAttribute,
    },
};

/// Two xforms in a chain: `local` posed beneath `parent`.
#[must_use]
pub fn chain(parent: &Transform, local: &Transform) -> Transform {
    Transform {
        translation: parent.translation + parent.rotation * (parent.scale * local.translation),
        rotation:    parent.rotation * local.rotation,
        scale:       parent.scale * local.scale,
    }
}

/// The box `root`'s whole tree fills, in `root`'s own frame.
///
/// `root`'s own xform is skipped — the caller replaces it when it places the
/// result — and every piece beneath it is walked and posed in turn, so this
/// costs one stream read per piece rather than a vertex upload.
pub fn measure(doc: &Document, root: (u64, u64)) -> anyhow::Result<(Vec3, Vec3)> {
    let identity = Transform {
        translation: Vec3::ZERO,
        rotation:    Quat::IDENTITY,
        scale:       Vec3::ONE,
    };
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);
    gather(doc, root, &identity, &mut min, &mut max)?;
    if min.x > max.x {
        return Ok((Vec3::ZERO, Vec3::ZERO));
    }
    Ok((min, max))
}

fn gather(
    doc: &Document,
    prim: (u64, u64),
    parent: &Transform,
    min: &mut Vec3,
    max: &mut Vec3,
) -> anyhow::Result<()> {
    if let Some(Property::MeshVertices(positions)) =
        doc.get(prim, &PropertyKey::MeshVertices(VertexAttribute::Position))
    {
        for vertex in positions.values.as_chunks::<3>().0 {
            let point = parent.transform_point(Vec3::new(vertex[0], vertex[1], vertex[2]));
            *min = min.min(point);
            *max = max.max(point);
        }
    }
    for child in doc.children(prim) {
        let local = match doc.get(child, &PropertyKey::Transform) {
            Some(Property::Transform(transform)) => transform,
            _ => Transform {
                translation: Vec3::ZERO,
                rotation:    Quat::IDENTITY,
                scale:       Vec3::ONE,
            },
        };
        gather(doc, child, &chain(parent, &local), min, max)?;
    }
    Ok(())
}

/// A box's centre and the uniform scale that makes its diagonal span `fraction`
/// of a shell's diameter, so any icon reads the same size inside its shell
/// however it was authored.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fit {
    pub center: Vec3,
    /// Diagonal multiplier; the shell's own scale rides on top of it.
    pub scale:  f32,
}

/// Fits a measured box: its diagonal becomes `fraction` of a shell's diameter.
/// A box with no geometry keeps scale one and centre zero, so an empty icon is
/// left where the shell's own scale puts it.
#[must_use]
pub fn fit(min: Vec3, max: Vec3, fraction: f32) -> Fit {
    let center = (max + min) * 0.5;
    let diagonal = (max - min).length();
    let scale = if diagonal > 0.0 {
        fraction * 2.0 / diagonal
    } else {
        1.0
    };
    Fit { center, scale }
}

#[cfg(test)]
mod tests {
    use wired_guest::math::Quat;

    use super::*;

    #[test]
    fn a_centred_box_needs_no_translation() {
        let fit = fit(Vec3::splat(-1.0), Vec3::splat(1.0), 0.5);
        assert_eq!(fit.center, Vec3::ZERO);
    }

    /// A box's diagonal is its corner-to-corner span; after fitting, the span
    /// is `fraction` of a shell's diameter and a corner sits `fraction` of the
    /// shell's radius from the centre.
    #[test]
    fn the_fitted_box_spans_the_requested_fraction() {
        let min = Vec3::ZERO;
        let max = Vec3::new(1.0, 1.0, 1.0);
        let fit = fit(min, max, 0.5);
        assert!(fit.scale.mul_add((max - min).length(), -1.0).abs() < 1.0e-6);
        let corner = (max - fit.center) * fit.scale;
        assert!((corner.length() - 0.5).abs() < 1.0e-6);
    }

    #[test]
    fn an_empty_box_is_left_alone() {
        let fit = fit(Vec3::ZERO, Vec3::ZERO, 0.5);
        assert_eq!(fit.center, Vec3::ZERO);
        assert!((fit.scale - 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn chain_applies_a_child_beneath_its_parent() {
        let parent = Transform {
            translation: Vec3::new(0.0, 1.0, 0.0),
            rotation:    Quat::IDENTITY,
            scale:       Vec3::splat(2.0),
        };
        let local = Transform {
            translation: Vec3::new(1.0, 0.0, 0.0),
            rotation:    Quat::IDENTITY,
            scale:       Vec3::splat(1.0),
        };
        let chained = chain(&parent, &local);
        assert_eq!(chained.translation, Vec3::new(2.0, 1.0, 0.0));
        assert_eq!(
            chained.transform_point(Vec3::ZERO),
            Vec3::new(2.0, 1.0, 0.0)
        );
    }
}
