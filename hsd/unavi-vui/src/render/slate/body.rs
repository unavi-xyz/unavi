//! A slate's body in the world: putting it where the script asks, keeping it
//! afloat, and drawing its face, rows and tether on it.

use std::f32::consts::TAU;

use wired_guest::math::{
    Quat,
    Transform,
    Vec3,
};

use super::{
    LIFT,
    Slate,
    float::facing,
};
use crate::{
    fit::chain,
    palette::Palette,
    render::{
        draw,
        graphs,
        pop,
    },
    slate::{
        self,
        PADDING,
        ROW_HEIGHT,
        WIDTH,
    },
    tether::{
        self,
        Pane,
    },
    tuning::Tuning,
    wired::{
        physics::simulation::{
            set_velocity,
            velocity,
        },
        scene::{
            document::Document,
            properties::{
                Collider,
                Property,
                PropertyKey,
                RigidBody,
                RigidBodyKind,
            },
        },
    },
};

/// Metres of collider through the pane.
const DEPTH: f32 = 0.01;
/// How far the face bobs, in metres, and how long one bob takes, in seconds.
const BOB_HEIGHT: f32 = 0.004;
const BOB_PERIOD: f32 = 4.5;

/// Light, and slowed quickly, so a let-go slate drifts a little and stops
/// rather than sailing off.
const BODY: RigidBody = RigidBody {
    kind:            RigidBodyKind::Dynamic,
    mass:            Some(0.3),
    friction:        Some(0.3),
    restitution:     Some(0.1),
    linear_damping:  Some(3.0),
    angular_damping: Some(5.0),
};

impl Slate {
    /// Sets a free slate's velocities so it floats facing `eye`.
    pub(super) fn steer(&mut self, doc: &Document, eye: Vec3) -> anyhow::Result<()> {
        if !self.simulated {
            return Ok(());
        }
        let Ok(pose) = doc.world_transform(self.body) else {
            return Ok(());
        };
        let moving = velocity(doc, self.body)
            .map_err(|err| anyhow::anyhow!("{err:?}"))?
            .linear;
        if let Some(steer) = self.float.steer(&pose, moving, eye) {
            set_velocity(doc, self.body, steer.linear, Some(steer.angular))
                .map_err(|err| anyhow::anyhow!("{err:?}"))?;
        }
        Ok(())
    }

    pub(super) fn draw(
        &mut self,
        doc: &Document,
        eye: &Transform,
        palette: &Palette,
        delta: f32,
    ) -> anyhow::Result<()> {
        let progress = pop::step(self.progress, self.shown, delta);
        let moved = (progress - self.progress).abs() > f32::EPSILON;
        self.progress = progress;
        if progress <= 0.0 {
            return self.put_away(doc);
        }

        let size = slate::size(self.rows.len());
        let mut batch = doc.local();
        let fresh = !self.simulated;
        if fresh {
            self.simulated = true;
            self.stale = true;
            // Shown again where it was put away, unless the script says where.
            if self.put.is_none() {
                self.put = self
                    .pose
                    .map(|pose| pose.translation + Vec3::Y * (size.y / 2.0));
            }
            batch = batch
                .set(self.body, Property::RigidBody(BODY))
                .set(self.body, Property::GravityScale(0.0));
        }
        // A pose written this frame reads back only from the next, so a move
        // is taken as written.
        let teleported = self.put.is_some();
        let pose = match self.put.take() {
            Some(top) => {
                let centre = top - Vec3::Y * (size.y / 2.0);
                let pose = Transform {
                    translation: centre,
                    rotation:    facing(centre, eye.translation),
                    scale:       Vec3::ONE,
                };
                batch = batch.set(self.body, Property::Transform(pose));
                self.float.rest_at(centre);
                Some(pose)
            }
            None => doc.world_transform(self.body).ok().or(self.pose),
        };
        self.pose = pose;

        self.bob = (self.bob + delta) % BOB_PERIOD;
        let face = Transform {
            translation: Vec3::Y
                * BOB_HEIGHT.mul_add((self.bob / BOB_PERIOD * TAU).sin(), size.y / 2.0),
            rotation:    Quat::IDENTITY,
            scale:       Vec3::splat(pop::scale(progress)),
        };
        batch = batch.set(self.face, Property::Transform(face));
        self.face_pose = pose.map(|pose| chain(&pose, &face));

        // A body entering the simulation this frame starts still anyway.
        if teleported && !fresh {
            batch.flush()?;
            batch = doc.local();
            // A move starts it at rest, not carrying on as it was going.
            if let Err(err) = set_velocity(doc, self.body, Some(Vec3::ZERO), Some(Vec3::ZERO)) {
                eprintln!("vui: slate stop: {err:?}");
            }
        }
        if moved {
            graphs::set_pane_open(doc, self.pane, pop::front(progress))?;
        }
        if !self.stale && !moved {
            batch.flush()?;
            return Ok(());
        }
        self.stale = false;

        // Resized only when its rows change how tall it is, since a new
        // collider is a new shape to the simulation.
        if self.collider_rows != Some(self.rows.len()) {
            self.collider_rows = Some(self.rows.len());
            batch = batch.set(
                self.body,
                Property::Collider(Collider::Cuboid(Vec3::new(size.x, size.y, DEPTH))),
            );
        }
        batch = batch.set(
            self.pane,
            Property::Transform(Transform {
                translation: Vec3::ZERO,
                rotation:    Quat::IDENTITY,
                scale:       Vec3::new(size.x, size.y, 1.0),
            }),
        );
        batch = match self.hovered {
            Some(index) => batch
                .set(
                    self.hover,
                    Property::Transform(Transform {
                        translation: Vec3::new(0.0, slate::row_centre(index), LIFT / 2.0),
                        rotation:    Quat::IDENTITY,
                        scale:       Vec3::new(WIDTH - PADDING, ROW_HEIGHT * 0.9, LIFT / 2.0),
                    }),
                )
                .set(
                    self.hover,
                    Property::Material(draw::ink(
                        self.accent.unwrap_or(palette.accent),
                        0.18 * progress,
                    )),
                ),
            None => batch.set(self.hover, Property::Transform(draw::hidden())),
        };

        batch = self.draw_rows(batch, palette, pop::front(progress) * size.y);
        batch.flush()?;
        Ok(())
    }

    /// Takes a slate that has shrunk away out of the simulation and out of
    /// sight, so a hidden slate is nothing to bump into.
    fn put_away(&mut self, doc: &Document) -> anyhow::Result<()> {
        self.face_pose = None;
        if !self.simulated {
            return Ok(());
        }
        self.simulated = false;
        self.collider_rows = None;
        self.float = super::float::Float::default();
        doc.local()
            .clear(self.body, PropertyKey::RigidBody)
            .clear(self.body, PropertyKey::GravityScale)
            .clear(self.body, PropertyKey::Collider)
            .set(self.body, Property::Transform(draw::hidden()))
            .flush()?;
        Ok(())
    }

    /// Draws the line from the pane's side to what it describes, popping
    /// with the pane, or puts it away.
    pub(super) fn draw_tether(
        &self,
        doc: &Document,
        palette: &Palette,
        tuning: &Tuning,
    ) -> anyhow::Result<()> {
        let tether = self.face_pose.zip(self.tether_to).map(|(face, to)| {
            let size = slate::size(self.rows.len());
            let pane = Pane {
                centre:     face.translation
                    - face.rotation * Vec3::Y * (size.y / 2.0 * face.scale.y),
                rotation:   face.rotation,
                half_width: size.x / 2.0 * face.scale.x,
            };
            tether::tether(&pane, to, tuning)
        });
        let color = self.accent.unwrap_or(palette.tether);
        self.tether.update(doc, tether, color, palette.tether_glow)
    }
}
