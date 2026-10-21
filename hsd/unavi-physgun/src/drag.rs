//! An element moved by editing where it is rather than by physics: held
//! where the aim ray grabbed it, at a distance the stick or wheel changes,
//! snapped to the space's grid unless freed, and kept on release.

use wired_guest::math::{
    Quat,
    Transform,
    Vec3,
};

use crate::{
    element::Element,
    pose_in_parent,
    snap::{
        TURN,
        to_grid,
    },
    unavi::author::api::Edit,
    wired::scene::{
        document::Layer,
        properties::{
            Property,
            PropertyKey,
        },
    },
};

/// Nearest and furthest a dragged element is held, in metres.
const MIN_DISTANCE: f32 = 0.5;
const MAX_DISTANCE: f32 = 50.0;
/// The smallest scale divided by, so a flattened element does not put an
/// infinity into its transform.
const MIN_SCALE: f32 = 1.0e-4;

pub struct Drag {
    element:    Element,
    edit:       Edit,
    /// Where the ray met the element, in its own frame, so that point stays
    /// under the ray.
    grab_local: Vec3,
    distance:   f32,
    rotation:   Quat,
    scale:      Vec3,
    /// 15° steps turned since the grab.
    turns:      i32,
    /// Shared, so everyone present sees it move, or local where the
    /// document has no shared layer.
    layer:      Layer,
}

impl Drag {
    /// Starts moving `element`, grabbed at `point` along `aim`. A `created`
    /// element is new, so undoing the drag removes it. A refusal hands the
    /// element back, so a new one can be taken out again.
    pub fn begin(
        element: Element,
        point: Vec3,
        aim: &Transform,
        label: &str,
        created: bool,
    ) -> Result<Self, (anyhow::Error, Element)> {
        let started = Self::start(&element, label, created);
        let (body, edit) = match started {
            Ok(started) => started,
            Err(err) => return Err((err, element)),
        };
        let layer = match element.doc.get(element.prim, &PropertyKey::Transform) {
            Some(current) => match element.doc.shared().set(element.prim, current).flush() {
                Ok(()) => Layer::Shared,
                Err(err) => {
                    eprintln!("build: moving for this peer only: {err:?}");
                    Layer::Local
                }
            },
            None => Layer::Local,
        };
        Ok(Self {
            layer,
            grab_local: (body.rotation.inverse() * (point - body.translation))
                / body.scale.max(Vec3::splat(MIN_SCALE)),
            distance: (point - aim.translation).length().clamp(MIN_DISTANCE, MAX_DISTANCE),
            rotation: body.rotation,
            scale: body.scale,
            turns: 0,
            element,
            edit,
        })
    }

    fn start(element: &Element, label: &str, created: bool) -> anyhow::Result<(Transform, Edit)> {
        let body = element.pose()?;
        let edit = Edit::new(label)?;
        if created {
            edit.track_creation(&element.doc, element.prim)?;
        } else {
            edit.track(&element.doc, &[(element.prim, PropertyKey::Transform)]);
        }
        Ok((body, edit))
    }

    /// Where the aim ray holds the element, which the beam ends at.
    #[must_use]
    pub fn point(&self, aim: &Transform) -> Vec3 {
        aim.translation + aim.forward() * self.distance
    }

    pub fn push(&mut self, metres: f32) {
        self.distance = (self.distance + metres).clamp(MIN_DISTANCE, MAX_DISTANCE);
    }

    pub const fn turn(&mut self, steps: i32) {
        self.turns += steps;
    }

    /// Moves the element to follow `aim`. With `grid`, the space's root in the
    /// tool's frame, it settles on the space's grid.
    pub fn update(&self, aim: &Transform, grid: Option<&Transform>) -> anyhow::Result<()> {
        let rotation = Quat::from_rotation_y(self.turns as f32 * TURN) * self.rotation;
        let point = aim.translation + aim.forward() * self.distance;
        let mut pose = Transform::new(
            point - rotation * (self.grab_local * self.scale),
            rotation,
            self.scale,
        );
        if let Some(grid) = grid {
            let mut in_space = grid.inverse() * pose;
            in_space.translation = to_grid(in_space.translation);
            pose = *grid * in_space;
        }

        let local = pose_in_parent(&self.element.doc, self.element.prim, pose)
            .ok_or_else(|| anyhow::anyhow!("the element's parent is not placed"))?;
        let batch = match self.layer {
            Layer::Shared => self.element.doc.shared(),
            Layer::Local => self.element.doc.local(),
        };
        batch.set(self.element.prim, Property::Transform(local)).flush()?;
        Ok(())
    }

    /// Keeps where it was left.
    pub fn end(self) -> anyhow::Result<()> {
        self.edit.commit()?;
        Ok(())
    }
}
