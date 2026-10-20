//! A beacon armed by its author unfolds into a small standing gate where it
//! comes to rest, with the beacon at the foot of its left post. Moving the
//! beacon away folds it.
//!
//! Arming is a `shared` write in the `state` group, so every peer agrees
//! whether a beacon is armed and which way its gate will face. Unfolding and
//! folding are decided on each peer from where the beacon is, the same way a
//! gate decides its seat.

use wired_guest::{
    beacon::{
        ARMED_KEY,
        BEACON_SIZE,
        DocumentId,
    },
    math::{
        Color,
        Quat,
        Transform,
        Vec3,
    },
};

use crate::{
    unavi::shapes::api::Cuboid,
    wired::{
        portal::portals,
        scene::{
            document::{
                Document,
                Layer,
            },
            properties::{
                Material,
                Portal,
                Property,
                PropertyKey,
            },
        },
    },
};

/// The authored prim the unfolded gate's portal is on.
const GATE_PRIM_NAME: &str = "gate";

const WIDTH: f32 = 1.0;
const HEIGHT: f32 = 2.0;
const POST: f32 = 0.035;

/// Metres a beacon may move in a tick and still count as lying still.
const STILL_STEP: f32 = 0.001;
/// Ticks a beacon must lie still before it unfolds.
const STILL_TICKS: u32 = 24;
/// Metres a beacon must be moved from where it unfolded to fold its gate.
const FOLD_DISTANCE: f32 = 0.15;
const UNFOLD_SECS: f32 = 0.5;
const FOLD_SECS: f32 = 0.25;

/// The yaw an armed beacon's gate faces, or `None` when disarmed.
pub fn armed(doc: &Document, body: (u64, u64)) -> Option<f32> {
    match doc.get(body, &PropertyKey::Custom(ARMED_KEY.to_owned())) {
        Some(Property::Custom((_, bytes))) => {
            let bytes: [u8; 4] = bytes.as_slice().try_into().ok()?;
            Some(f32::from_le_bytes(bytes))
        }
        _ => None,
    }
}

/// States `yaw` as the armed facing, or disarms with `None`.
pub fn arm(doc: &Document, body: (u64, u64), yaw: Option<f32>) -> anyhow::Result<()> {
    let bytes = yaw.map_or_else(Vec::new, |yaw| yaw.to_le_bytes().to_vec());
    doc.shared()
        .set(body, Property::Custom((ARMED_KEY.to_owned(), bytes)))
        .flush()?;
    Ok(())
}

/// The yaw that turns a gate's front toward someone looking along `forward`.
pub fn facing(forward: Vec3) -> f32 {
    (-forward.x).atan2(-forward.z)
}

/// Where the gate stands when a beacon at `body` unfolds facing `yaw`: its
/// left post's foot on the floor under the beacon.
fn stand(body: Vec3, yaw: f32) -> Transform {
    let rotation = Quat::from_rotation_y(yaw);
    let floor = Vec3::new(body.x, body.y - BEACON_SIZE / 2.0, body.z);
    Transform::new(
        floor + rotation * Vec3::new(WIDTH / 2.0 + POST / 2.0, 0.0, 0.0),
        rotation,
        Vec3::ONE,
    )
}

struct Body {
    /// Scaled from flat to full height to unfold.
    pivot: (u64, u64),
    /// The portal sits at the gate's centre, so it is lifted by half its
    /// height.
    lift:  Transform,
}

pub struct Deploy {
    gate:   (u64, u64),
    space:  DocumentId,
    color:  Color,
    body:   Option<Body>,
    last:   Option<Vec3>,
    still:  u32,
    /// Where the beacon was when its gate unfolded.
    anchor: Option<Vec3>,
    grow:   f32,
    shown:  Option<i16>,
}

impl Deploy {
    /// The deploy of a beacon whose document carries the authored gate prim.
    /// A beacon minted before the prim existed has none and never unfolds.
    pub fn find(doc: &Document, space: DocumentId, color: Color) -> Option<Self> {
        let gate = doc.find_by_name(GATE_PRIM_NAME).into_iter().next()?;
        Some(Self {
            gate,
            space,
            color,
            body: None,
            last: None,
            still: 0,
            anchor: None,
            grow: 0.0,
            shown: None,
        })
    }

    pub const fn is_unfolded(&self) -> bool {
        self.anchor.is_some()
    }

    /// Unfolds once an armed beacon lies still away from any socket, and folds
    /// once it is disarmed or moved. Answers whether the beacon was moved off
    /// its gate this tick, which only its author can disarm.
    pub fn update(
        &mut self,
        doc: &Document,
        at: Vec3,
        armed: Option<f32>,
        socket_near: bool,
    ) -> anyhow::Result<bool> {
        let moved = self.last.map_or(f32::INFINITY, |last| last.distance(at));
        self.still = if moved < STILL_STEP {
            self.still.saturating_add(1)
        } else {
            0
        };
        self.last = Some(at);

        let mut moved_off = false;
        match (self.anchor, armed) {
            (None, Some(yaw)) if self.still >= STILL_TICKS && !socket_near => {
                self.unfold(doc, at, yaw)?;
            }
            (Some(anchor), armed) => {
                moved_off = anchor.distance(at) > FOLD_DISTANCE;
                if armed.is_none() || moved_off {
                    self.fold(doc);
                }
            }
            _ => {}
        }
        Ok(moved_off)
    }

    /// Raises or lowers the frame by `dt` seconds.
    pub fn animate(&mut self, doc: &Document, dt: f32) -> anyhow::Result<()> {
        let target = if self.anchor.is_some() { 1.0 } else { 0.0 };
        let rate = if target > self.grow {
            1.0 / UNFOLD_SECS
        } else {
            -1.0 / FOLD_SECS
        };
        self.grow = rate.mul_add(dt, self.grow).clamp(0.0, 1.0);
        self.show(doc)
    }

    fn unfold(&mut self, doc: &Document, at: Vec3, yaw: f32) -> anyhow::Result<()> {
        self.anchor = Some(at);
        let stand = stand(at, yaw);
        if self.body.is_none() {
            self.body = Some(self.build(doc)?);
        }
        let lift = self
            .body
            .as_ref()
            .map_or(Transform::IDENTITY, |body| body.lift);
        doc.local()
            .set(self.gate, Property::Transform(stand * lift))
            .set(
                self.gate,
                Property::Portal(Portal {
                    width:       WIDTH,
                    height:      HEIGHT,
                    destination: None,
                }),
            )
            .flush()?;
        if let Err(err) = portals::open(doc, self.gate, self.space) {
            eprintln!("beacon: open: {err:?}");
        }
        Ok(())
    }

    fn fold(&mut self, doc: &Document) {
        self.anchor = None;
        self.still = 0;
        if let Err(err) = portals::close(doc, self.gate) {
            eprintln!("beacon: close: {err:?}");
        }
    }

    /// Two posts and a beam of light around the portal, hung from a pivot at
    /// the floor so scaling it raises the frame.
    fn build(&self, doc: &Document) -> anyhow::Result<Body> {
        let lift = Transform::from_translation(Vec3::new(0.0, HEIGHT / 2.0, 0.0));
        let pivot = doc.create_prim(Layer::Local, Some(self.gate))?;
        let line = Material {
            base_color: Some(Color {
                r: self.color.r * 0.3,
                g: self.color.g * 0.3,
                b: self.color.b * 0.3,
                a: 1.0,
            }),
            emissive: Some(self.color),
            ..Material::default()
        };
        let mut batch = doc.local().set(
            pivot,
            Property::Transform(Transform::from_translation(Vec3::new(
                0.0,
                -HEIGHT / 2.0,
                0.0,
            ))),
        );
        for (size, at) in [
            (
                Vec3::new(POST, HEIGHT, POST),
                Vec3::new(-f32::midpoint(WIDTH, POST), HEIGHT / 2.0, 0.0),
            ),
            (
                Vec3::new(POST, HEIGHT, POST),
                Vec3::new(f32::midpoint(WIDTH, POST), HEIGHT / 2.0, 0.0),
            ),
            (
                Vec3::new(POST.mul_add(2.0, WIDTH), POST, POST),
                Vec3::new(0.0, HEIGHT + POST / 2.0, 0.0),
            ),
        ] {
            let bar = Cuboid::new(size).mesh()?;
            batch = batch
                .set(bar, Property::Parent(Some(pivot)))
                .set(bar, Property::Material(line))
                .set(bar, Property::Transform(Transform::from_translation(at)));
        }
        batch.flush()?;
        Ok(Body { pivot, lift })
    }

    fn show(&mut self, doc: &Document) -> anyhow::Result<()> {
        let Some(body) = &self.body else {
            return Ok(());
        };
        let eased = 1.0 - (1.0 - self.grow).powi(3);
        let step = (eased * 100.0).round() as i16;
        if self.shown == Some(step) {
            return Ok(());
        }
        self.shown = Some(step);
        doc.local()
            .set(
                body.pivot,
                Property::Transform(Transform::new(
                    Vec3::new(0.0, -HEIGHT / 2.0, 0.0),
                    Quat::IDENTITY,
                    Vec3::new(1.0, eased.max(1.0e-4), 1.0),
                )),
            )
            .flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_2;

    use super::*;

    #[test]
    fn a_gate_faces_whoever_threw_it() {
        let forward = Vec3::new(0.0, 0.0, -1.0);
        let yaw = facing(forward);
        let front = Quat::from_rotation_y(yaw) * Vec3::Z;
        assert!(
            front.dot(-forward) > 0.999,
            "the portal's front faces {front:?}, away from the thrower"
        );
    }

    #[test]
    fn the_beacon_stands_at_the_foot_of_the_left_post() {
        let yaw = FRAC_PI_2;
        let gate = stand(Vec3::new(2.0, 0.5, 3.0), yaw);
        let left_post = gate.transform_point(Vec3::new(-(WIDTH + POST) / 2.0, 0.0, 0.0));
        assert!(
            (left_post - Vec3::new(2.0, 0.5 - BEACON_SIZE / 2.0, 3.0)).length() < 1.0e-4,
            "left post foot at {left_post:?}"
        );
    }
}
