//! The slab a bed of keys stands on.

use std::cell::Cell;

use wired_guest::math::{
    Quat,
    Transform,
    Vec3,
};

use super::Bodies;
use crate::{
    material::graphs,
    part::{
        draw,
        mesh,
        shadow::Shadow,
        view::BedView,
    },
    wired::scene::{
        document::Document,
        properties::Property,
    },
};

/// How far behind the surface's plane the slab stands, past the back of the
/// thickest key.
const BACK: f32 = 0.014;

/// The bed's slab prim, made hidden and given its mesh and graph when a bed
/// is first drawn on it.
pub(super) struct BedPrim {
    prim:  (u64, u64),
    /// Whether the slab has its mesh and graph yet.
    ready: Cell<bool>,
    pose:  Shadow<Option<Transform>>,
}

impl BedPrim {
    pub(super) const fn new(prim: (u64, u64)) -> Self {
        Self {
            prim,
            ready: Cell::new(false),
            pose: Shadow::new(),
        }
    }

    /// Takes back what was written, for a frame whose edits did not land.
    pub(super) fn forget(&self) {
        self.ready.set(false);
        self.pose.clear();
    }
}

/// Where the slab stands and how big it is: the bed's extents, scaled by how
/// far it has arrived.
fn pose(bed: BedView) -> Transform {
    Transform {
        translation: Vec3::new(0.0, 0.0, -BACK),
        rotation:    Quat::IDENTITY,
        scale:       Vec3::new(
            bed.extents.x * 2.0 * bed.bloom,
            bed.extents.y * 2.0 * bed.bloom,
            1.0,
        ),
    }
}

impl Bodies {
    /// Draws the slab under the keys, or puts it away for a level that has
    /// none or a surface that is down.
    pub(super) fn apply_bed(&self, doc: &Document, bed: Option<BedView>) -> anyhow::Result<()> {
        let wanted = bed.filter(|_| self.shown.get()).map(pose);
        let Some(pose) = self.bed.pose.diff(wanted) else {
            return Ok(());
        };
        let carve = wanted.is_some() && !self.bed.ready.get();
        let mut batch = doc.local();
        if carve {
            batch = draw::mesh(&mesh::face())
                .into_iter()
                .fold(batch, |batch, property| batch.set(self.bed.prim, property));
        }
        batch = batch.set(
            self.bed.prim,
            Property::Transform(pose.unwrap_or_else(draw::hidden)),
        );
        if let Err(err) = batch.flush() {
            self.bed.forget();
            return Err(err.into());
        }
        if carve {
            self.bed.ready.set(true);
            graphs::slab::set_slab(doc, self.bed.prim)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use wired_guest::math::Vec2;

    use super::*;

    #[test]
    fn the_slab_is_the_bed_s_extents_scaled_by_how_far_it_has_arrived() {
        let whole = pose(BedView {
            extents: Vec2::new(0.165, 0.165),
            bloom:   1.0,
        });
        assert!((whole.scale.x - 0.33).abs() < 1.0e-6);
        let half = pose(BedView {
            extents: Vec2::new(0.165, 0.165),
            bloom:   0.5,
        });
        assert!((half.scale.y - 0.165).abs() < 1.0e-6);
        assert!(
            whole.translation.z < 0.0,
            "behind the keys, not through them"
        );
    }
}
