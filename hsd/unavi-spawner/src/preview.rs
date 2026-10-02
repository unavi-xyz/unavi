//! A spinning cube above the artifact, previewing the shape about to be
//! spawned. Carried by `wired:agent/local.attach` rather than a prim whose
//! transform this script copies from the camera every frame.

use std::cell::Cell;

use wired_guest::math::{
    Color,
    Quat,
    Transform,
    Vec3,
};

use crate::{
    palette,
    unavi::shapes::api::Cuboid,
    wired::{
        agent::local::{
            Attachment,
            attach,
        },
        scene::{
            document::{
                Document,
                script_document,
            },
            properties::Property,
        },
    },
};

const PREVIEW_SIZE: f32 = 0.07;
const ABOVE: f32 = 0.13;
const SPIN: f32 = 1.4;

/// A document of its own, so `attach` carries exactly this cube and nothing
/// else the script may write.
pub struct Preview {
    doc:      Document,
    cube:     (u64, u64),
    spin:     Cell<f32>,
    color:    Cell<Option<Color>>,
    /// The `(offset, scale)` last sent to `attach`, so a pose that has not
    /// moved does not pay for another host round trip every frame.
    attached: Cell<Option<(Vec3, f32)>>,
}

impl Preview {
    pub fn new(offset: Vec3, scale: f32) -> anyhow::Result<Self> {
        let doc = script_document()?;
        let cube = Cuboid::new(Vec3::splat(PREVIEW_SIZE)).mesh()?;

        let preview = Self {
            doc,
            cube,
            spin: Cell::new(0.0),
            color: Cell::new(None),
            attached: Cell::new(None),
        };
        preview.reattach(offset, scale);
        Ok(preview)
    }

    pub fn update(&self, offset: Vec3, scale: f32, color: Color, delta: f32) {
        if self.color.get() != Some(color) {
            self.color.set(Some(color));
            if let Err(err) = self
                .doc
                .local()
                .set(self.cube, Property::Material(palette::preview(color)))
                .flush()
            {
                eprintln!("spawner: preview material: {err:?}");
            }
        }

        self.reattach(offset, scale);

        let spin = delta.mul_add(SPIN, self.spin.get());
        self.spin.set(spin);
        if let Err(err) = self
            .doc
            .local()
            .set(
                self.cube,
                Property::Transform(Transform {
                    translation: Vec3::new(0.0, ABOVE, 0.0),
                    rotation:    Quat::new(0.0, (spin * 0.5).sin(), 0.0, (spin * 0.5).cos()),
                    scale:       Vec3::ONE,
                }),
            )
            .flush()
        {
            eprintln!("spawner: spin preview: {err:?}");
        }
    }

    /// Re-anchors the preview to the camera, an async round trip to the
    /// host, skipped when `offset`/`scale` already match the last call.
    fn reattach(&self, offset: Vec3, scale: f32) {
        if self.attached.get() == Some((offset, scale)) {
            return;
        }
        if let Err(err) = attach(
            &self.doc,
            Attachment::Camera,
            Transform {
                translation: offset,
                rotation:    Quat::IDENTITY,
                scale:       Vec3::splat(scale),
            },
        ) {
            eprintln!("spawner: attach preview: {err:?}");
            return;
        }
        self.attached.set(Some((offset, scale)));
    }
}
