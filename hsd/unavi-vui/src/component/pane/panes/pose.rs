//! A pane in the world: where it stands or floats, and its collider,
//! prims and tether drawn there each frame.

use std::f32::consts::TAU;

use wired_guest::math::{
    Transform,
    Vec3,
};

use super::{
    Mount,
    Mounted,
    PaneId,
    Panes,
};
use crate::{
    machinery::{
        float::facing,
        standing::{
            eased,
            turned_toward,
        },
    },
    part::{
        draw,
        pop,
        tether::{
            Pane as Body,
            {
                self,
            },
        },
    },
    tokens::{
        palette::Palette,
        tuning::Tuning,
    },
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
/// How far a floating pane bobs, in metres, and how long one bob takes, in
/// seconds.
const BOB_HEIGHT: f32 = 0.004;
const BOB_PERIOD: f32 = 4.5;

/// Light, and slowed quickly, so a let-go pane drifts a little and stops
/// rather than sailing off.
const BODY: RigidBody = RigidBody {
    kind:            RigidBodyKind::Dynamic,
    mass:            Some(0.3),
    friction:        Some(0.3),
    restitution:     Some(0.1),
    linear_damping:  Some(3.0),
    angular_damping: Some(5.0),
};

impl Panes {
    /// Eases each pane in or out, poses it, and draws it and its tether.
    pub fn update(
        &mut self,
        doc: &Document,
        eye: &Transform,
        palette: &Palette,
        tuning: &Tuning,
        delta: f32,
    ) -> anyhow::Result<()> {
        self.eye = Some(eye.translation);
        for mounted in self.slots.iter_mut().flatten() {
            mounted.draw(doc, eye, palette, tuning, delta)?;
        }
        Ok(())
    }

    /// Sets each floating pane's velocities so it stays afloat facing the
    /// viewer.
    pub(super) fn steer(&mut self, doc: &Document) {
        let Some(eye) = self.eye else {
            return;
        };
        for mounted in self.slots.iter_mut().flatten() {
            if let Err(err) = mounted.steer(doc, eye) {
                eprintln!("vui: pane float: {err:?}");
            }
        }
    }

    /// Where a floating pane is now, so a consumer can anchor to it.
    #[must_use]
    pub fn position(&self, id: PaneId) -> Option<Vec3> {
        self.slots
            .get(id.0)?
            .as_ref()?
            .pose
            .map(|pose| pose.translation)
    }
}

impl Mounted {
    fn steer(&mut self, doc: &Document, eye: Vec3) -> anyhow::Result<()> {
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

    fn draw(
        &mut self,
        doc: &Document,
        eye: &Transform,
        palette: &Palette,
        tuning: &Tuning,
        delta: f32,
    ) -> anyhow::Result<()> {
        let em = self.em(tuning);
        let tuning = &Tuning {
            pane_em: em,
            ..*tuning
        };
        let size = self.pane.size();
        let metres = size * em;
        let visible = self.shown && self.mount.is_some();

        let origin = match self.mount {
            Some(mount) if visible => self.pose_for(doc, mount, eye, tuning, metres, delta)?,
            _ => {
                if self.prims.progress() <= 0.0 {
                    self.put_away(doc)?;
                }
                Vec3::new(0.0, metres.y / 2.0, 0.0)
            }
        };

        if self.runs.is_none() {
            self.runs = Some(self.pane.runs());
        }
        let shown = self
            .runs
            .as_ref()
            .filter(|_| visible)
            .map(|runs| (runs, size));
        self.prims
            .update(doc, shown, origin, palette, tuning, delta)?;

        let tether = match self.tether_to {
            Some(to) if self.prims.progress() > 0.0 => self.pose.map(|pose| {
                let body = Body {
                    centre:     pose.translation,
                    rotation:   pose.rotation,
                    half_width: metres.x / 2.0 * pop::scale(self.prims.progress()),
                };
                tether::tether(&body, to, tuning)
            }),
            None | Some(_) => None,
        };
        self.tether.update(
            doc,
            tether,
            self.hue.unwrap_or(palette.tether),
            palette.tether_glow,
        )
    }

    /// Poses the body for `mount` and returns where the pane's top centre is
    /// in the body's frame.
    fn pose_for(
        &mut self,
        doc: &Document,
        mount: Mount,
        eye: &Transform,
        tuning: &Tuning,
        metres: wired_guest::math::Vec2,
        delta: f32,
    ) -> anyhow::Result<Vec3> {
        let mut batch = doc.local();
        let mut bob = 0.0;
        match mount {
            Mount::Standing { at } => {
                if self.simulated {
                    self.simulated = false;
                    self.float = super::Float::default();
                    batch = batch
                        .clear(self.body, PropertyKey::RigidBody)
                        .clear(self.body, PropertyKey::GravityScale);
                }
                let target = Transform {
                    translation: at + Vec3::Y * (metres.y / 2.0),
                    rotation:    turned_toward(at, eye),
                    scale:       Vec3::ONE,
                };
                let drawn = eased(self.drawn, target, tuning.stand_speed, delta);
                self.drawn = Some(drawn);
                self.pose = Some(drawn);
                if let Some(pose) = self.written.diff(drawn) {
                    batch = batch.set(self.body, Property::Transform(pose));
                }
            }
            Mount::Floating { at } => {
                let fresh = !self.simulated;
                if fresh {
                    self.simulated = true;
                    self.drawn = None;
                    self.written.clear();
                    batch = batch
                        .set(self.body, Property::RigidBody(BODY))
                        .set(self.body, Property::GravityScale(0.0));
                }
                let placed = self.put.take();
                let pose = match placed {
                    Some(centre) => {
                        let pose = Transform {
                            translation: centre,
                            rotation:    facing(centre, eye.translation),
                            scale:       Vec3::ONE,
                        };
                        batch = batch.set(self.body, Property::Transform(pose));
                        self.float.rest_at(centre);
                        Some(pose)
                    }
                    None if fresh => {
                        let pose = Transform {
                            translation: at,
                            rotation:    facing(at, eye.translation),
                            scale:       Vec3::ONE,
                        };
                        batch = batch.set(self.body, Property::Transform(pose));
                        self.float.rest_at(at);
                        Some(pose)
                    }
                    None => doc.world_transform(self.body).ok().or(self.pose),
                };
                self.pose = pose;
                if placed.is_some() && !fresh {
                    batch.flush()?;
                    batch = doc.local();
                    if let Err(err) =
                        set_velocity(doc, self.body, Some(Vec3::ZERO), Some(Vec3::ZERO))
                    {
                        eprintln!("vui: pane stop: {err:?}");
                    }
                }
                self.bob = (self.bob + delta) % BOB_PERIOD;
                bob = BOB_HEIGHT * (self.bob / BOB_PERIOD * TAU).sin();
            }
        }
        if self.collider.diff(metres).is_some() {
            batch = batch.set(
                self.body,
                Property::Collider(Collider::Cuboid(Vec3::new(metres.x, metres.y, DEPTH))),
            );
        }
        batch.flush()?;
        self.placed = true;
        self.bob_offset = bob;
        Ok(Vec3::new(0.0, metres.y / 2.0 + bob, 0.0))
    }

    /// Takes a pane that has shrunk away out of the simulation and out of
    /// sight, so a hidden pane is nothing to bump into.
    fn put_away(&mut self, doc: &Document) -> anyhow::Result<()> {
        self.pose = None;
        self.drawn = None;
        if !self.placed {
            return Ok(());
        }
        self.placed = false;
        self.collider.clear();
        self.written.clear();
        self.simulated = false;
        self.float = super::Float::default();
        doc.local()
            .clear(self.body, PropertyKey::RigidBody)
            .clear(self.body, PropertyKey::GravityScale)
            .clear(self.body, PropertyKey::Collider)
            .set(self.body, Property::Transform(draw::hidden()))
            .flush()?;
        Ok(())
    }
}
