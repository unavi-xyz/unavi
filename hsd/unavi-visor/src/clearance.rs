//! Moves a pane out of whatever stands between it and the viewer, or under it.

use unavi_lens::DocumentId;
use wired_guest::{
    clearance,
    math::{
        Ray,
        Vec2,
        Vec3,
    },
};

use crate::wired::{
    physics::simulation::{
        RayFilter,
        raycast,
    },
    scene::document::script_document,
};

/// Casts the visor's panes against the scene, past its own colliders.
pub struct Clearance {
    /// The visor's own document, whose colliders are never in the way.
    own:    DocumentId,
    /// The user's shell, whose surfaces float in front of the viewer and are
    /// never in the way either.
    shell:  Option<DocumentId>,
    warned: bool,
}

impl Clearance {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            own:    script_document()?.id(),
            shell:  None,
            warned: false,
        })
    }

    pub const fn set_shell(&mut self, shell: Option<DocumentId>) {
        self.shell = shell;
    }

    /// Where a `size` pane wanted with its bottom centre at `at` stands as
    /// seen from `eye`. Colliders in `exclude` do not count.
    pub fn clear(&mut self, eye: Vec3, at: Vec3, size: Vec2, exclude: &[DocumentId]) -> Vec3 {
        let mut exclude_documents = vec![self.own];
        exclude_documents.extend(self.shell);
        exclude_documents.extend_from_slice(exclude);
        let filter = RayFilter {
            exclude_local_agent: true,
            exclude_documents,
        };
        clearance::clear(eye, at, size, |from, to| self.cast(from, to, &filter))
    }

    /// The distance from `from` to the first collider on the way to `to`.
    fn cast(&mut self, from: Vec3, to: Vec3, filter: &RayFilter) -> Option<f32> {
        let reach = from.distance(to);
        if reach < f32::EPSILON {
            return None;
        }
        let ray = Ray {
            origin:    from,
            direction: (to - from) / reach,
        };
        match raycast(ray, reach, filter) {
            Ok(hit) => hit.map(|hit| hit.distance),
            Err(err) => {
                if !self.warned {
                    eprintln!("visor: clearance raycast: {err:?}");
                    self.warned = true;
                }
                None
            }
        }
    }
}
