//! How the socket shows what it holds: its marks, the trace of light around
//! the doorway, the shaft and motes over an empty socket, and the ghost.
//! Every value eases toward its target, and a prim is written only when what
//! it shows has changed.

use std::f32::consts::TAU;

use wired_guest::{
    beacon::DocumentId,
    math::{
        Color,
        Quat,
        Transform,
        Vec3,
    },
};

use crate::{
    frame::{
        ORBITERS,
        Parts,
        material::{
            self,
            NEUTRAL,
        },
    },
    wired::scene::{
        document::Document,
        properties::{
            Material,
            Property,
        },
    },
};

/// Property writes gathered over one tick, applied as one batch.
type Writes = Vec<((u64, u64), Property)>;

/// Seconds for the trace to run from the socket around the doorway.
const CHARGE_SECS: f32 = 1.4;
const DISCHARGE_SECS: f32 = 0.5;
const BREATH_SECS: f32 = 4.0;
/// Per second, the rate colour and brightness close on their targets.
const EASE_RATE: f32 = 7.0;
const FADE_RATE: f32 = 4.0;
/// Radians per second.
const GHOST_TURN: f32 = 0.6;
const ORBIT_TURN: f32 = 0.45;
const ORBIT_RADIUS: f32 = 0.13;
/// Seconds for a ghost pulled from the socket to rise out and fade.
const UNPLUG_SECS: f32 = 0.45;
const UNPLUG_RISE: f32 = 0.3;
/// Fixed ticks between moves of the motes.
const ORBIT_TICKS: u32 = 2;

/// What the socket is showing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mood {
    /// Nothing near, nothing open.
    Idle,
    /// A beacon for a space of this colour is near, `closeness` from 0 at the
    /// edge of the pull to 1 at the seat.
    Near { color: Color, closeness: f32 },
    /// A beacon is seated.
    Seated(Color),
    /// The portal is open with nothing seated, as when the far side opened it.
    Ghost { space: DocumentId, color: Color },
}

struct Targets {
    color:  Color,
    /// The marks and the trace.
    level:  f32,
    charge: bool,
    shaft:  f32,
    motes:  f32,
    ghost:  f32,
}

impl Mood {
    fn targets(self, breath: f32) -> Targets {
        let wave = (TAU * breath).sin();
        match self {
            Self::Idle => Targets {
                color:  NEUTRAL,
                level:  0.08f32.mul_add(wave, 0.2),
                charge: false,
                shaft:  0.05f32.mul_add(wave, 0.14),
                motes:  1.0,
                ghost:  0.0,
            },
            Self::Near { color, closeness } => {
                let closeness = closeness.clamp(0.0, 1.0);
                Targets {
                    color,
                    level: 0.7f32.mul_add(closeness, 0.3),
                    charge: false,
                    shaft: 0.55f32.mul_add(closeness, 0.25),
                    motes: 1.0,
                    ghost: 0.0,
                }
            }
            Self::Seated(color) => Targets {
                color,
                level: 1.3,
                charge: true,
                shaft: 0.0,
                motes: 0.0,
                ghost: 0.0,
            },
            Self::Ghost { color, .. } => Targets {
                color,
                level: 0.5,
                charge: true,
                shaft: 0.12,
                motes: 0.0,
                ghost: 1.0,
            },
        }
    }
}

/// What was last written, quantized so easing toward a target stops writing
/// once the change is too small to see.
#[derive(Clone, PartialEq, Eq, Default)]
struct Shown {
    glow:  [i16; 4],
    runs:  Vec<i16>,
    shaft: i16,
    ghost: [i16; 3],
}

pub struct Look {
    parts:     Parts,
    total:     f32,
    color:     Color,
    level:     f32,
    charge:    f32,
    shaft:     f32,
    motes:     f32,
    orbit:     f32,
    ghost:     f32,
    yaw:       f32,
    /// How far a let-go ghost has risen out and faded, 0 while nothing is
    /// unplugged or while a hand still holds it.
    unplug:    f32,
    /// The pairing whose ghost was pulled out, while it is still gone.
    unplugged: Option<DocumentId>,
    /// Whether a hand took the ghost, so the gate must stop moving it.
    carried:   bool,
    /// Whether that hand still holds it, which keeps it whole until let go.
    held:      bool,
    breath:    f32,
    tick:      u32,
    shown:     Option<Shown>,
}

fn approach(from: f32, to: f32, rate: f32, dt: f32) -> f32 {
    let k = 1.0 - (-rate * dt).exp();
    (to - from).mul_add(k, from)
}

fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

fn quantize(value: f32, steps: f32) -> i16 {
    (value * steps).round() as i16
}

/// How much of each run of `lengths` is drawn once `covered` metres of the
/// whole trace are, each from 0 to 1.
fn coverage(lengths: impl Iterator<Item = f32>, covered: f32) -> Vec<f32> {
    let mut start = 0.0;
    lengths
        .map(|length| {
            let drawn = ((covered - start) / length).clamp(0.0, 1.0);
            start += length;
            drawn
        })
        .collect()
}

impl Look {
    pub fn new(parts: Parts) -> Self {
        let total = parts.trace.iter().map(|run| run.length).sum();
        Self {
            parts,
            total,
            color: NEUTRAL,
            level: 0.0,
            charge: 0.0,
            shaft: 0.0,
            motes: 0.0,
            orbit: 0.0,
            ghost: 0.0,
            yaw: 0.0,
            unplug: 0.0,
            unplugged: None,
            carried: false,
            held: false,
            breath: 0.0,
            tick: 0,
            shown: None,
        }
    }

    /// Lifts the ghost out of the socket and fades it. `grabbed` when a hand
    /// took it: the ghost stays whole in that hand until [`Look::release`], and
    /// the gate leaves its pose alone from then on. Otherwise the gate lifts it
    /// out itself. `space` names the pairing that is now gone.
    pub const fn unplug(&mut self, space: DocumentId, grabbed: bool) {
        if self.unplugged.is_some() {
            self.carried |= grabbed;
            self.held |= grabbed;
            return;
        }
        self.unplugged = Some(space);
        self.carried = grabbed;
        self.held = grabbed;
    }

    /// Lets go: a carried ghost starts to dissolve. Does nothing when no ghost
    /// was pulled.
    pub const fn release(&mut self) {
        self.held = false;
    }

    /// Whether a pairing has been unplugged and its ghost is on its way out.
    pub const fn is_unplugged(&self) -> bool {
        self.unplugged.is_some()
    }

    pub fn update(&mut self, doc: &Document, mood: Mood, dt: f32) -> anyhow::Result<()> {
        self.step(mood, dt);
        let covered = ease_out(self.charge) * self.total;
        let runs = coverage(self.parts.trace.iter().map(|run| run.length), covered);
        let lift = ease_out(self.unplug);
        let opacity = self.ghost * (1.0 - lift);
        let next = Shown {
            glow:  [
                quantize(self.color.r, 64.0),
                quantize(self.color.g, 64.0),
                quantize(self.color.b, 64.0),
                quantize(self.level, 20.0),
            ],
            runs:  runs.iter().map(|&drawn| quantize(drawn, 100.0)).collect(),
            shaft: quantize(self.shaft, 40.0),
            ghost: [
                quantize(opacity, 32.0),
                quantize(self.yaw, 24.0),
                quantize(lift, 50.0),
            ],
        };

        let last = self.shown.take();
        let mut writes = Vec::new();
        self.write_marks(last.as_ref(), &next, &mut writes);
        self.write_trace(last.as_ref(), &next, &runs, &mut writes);
        self.write_shaft(last.as_ref(), &next, &mut writes);
        self.write_motes(last.is_none(), &mut writes);
        self.write_ghost(last.as_ref(), &next, lift, opacity, &mut writes);
        self.shown = Some(next);

        if writes.is_empty() {
            return Ok(());
        }
        writes
            .into_iter()
            .fold(doc.local(), |batch, (prim, property)| {
                batch.set(prim, property)
            })
            .flush()?;
        Ok(())
    }

    /// Whether the ghost can be taken hold of: shown, and not already pulled
    /// out.
    pub fn ghost_touchable(&self) -> bool {
        self.ghost > 0.5 && self.unplugged.is_none()
    }

    /// Whether any of the ghost is still on screen.
    pub fn ghost_visible(&self) -> bool {
        self.ghost > 0.01 && self.unplug < 1.0
    }

    /// Eases every value one tick toward what `mood` asks for.
    fn step(&mut self, mood: Mood, dt: f32) {
        self.breath = (self.breath + dt / BREATH_SECS).fract();
        self.tick = self.tick.wrapping_add(1);
        let targets = mood.targets(self.breath);

        self.color = Color {
            r: approach(self.color.r, targets.color.r, EASE_RATE, dt),
            g: approach(self.color.g, targets.color.g, EASE_RATE, dt),
            b: approach(self.color.b, targets.color.b, EASE_RATE, dt),
            a: 1.0,
        };
        self.level = approach(self.level, targets.level, EASE_RATE, dt);
        self.charge = if targets.charge {
            (self.charge + dt / CHARGE_SECS).min(1.0)
        } else {
            (self.charge - dt / DISCHARGE_SECS).max(0.0)
        };
        self.shaft = approach(self.shaft, targets.shaft, FADE_RATE, dt);
        self.motes = approach(self.motes, targets.motes, FADE_RATE, dt);
        self.orbit = ORBIT_TURN.mul_add(dt, self.orbit) % TAU;
        // A ghost in a hand stays whole until it is let go, even though the
        // pairing it stood for is already gone.
        let ghost = if self.held { 1.0 } else { targets.ghost };
        self.ghost = approach(self.ghost, ghost, FADE_RATE, dt);
        if self.ghost > 0.01 {
            self.yaw = GHOST_TURN.mul_add(dt, self.yaw) % TAU;
        }
        if self.unplugged.is_some() && !self.held {
            self.unplug = if self.unplug == 0.0 {
                f32::EPSILON
            } else {
                (self.unplug + dt / UNPLUG_SECS).min(1.0)
            };
        }
        // A pulled ghost stays gone for the rest of its pairing, even if this
        // peer has not yet seen the portal close. It comes back only once it
        // has faded and the pairing it was pulled from is no longer shown, or
        // when the socket shows a different pairing.
        let elsewhere = matches!(mood, Mood::Ghost { space, .. } if Some(space) != self.unplugged);
        let gone = self.ghost <= 0.01 && !matches!(mood, Mood::Ghost { .. });
        if self.unplugged.is_some() && (elsewhere || gone) {
            self.unplug = 0.0;
            self.unplugged = None;
            self.carried = false;
            self.held = false;
            self.ghost = 0.0;
            // The prim is wherever the hand left it, so restate all of it.
            self.shown = None;
        }
    }

    fn glow(&self) -> Material {
        material::glow(self.color, self.level)
    }

    fn write_marks(&self, last: Option<&Shown>, next: &Shown, writes: &mut Writes) {
        if last.map(|s| s.glow) == Some(next.glow) {
            return;
        }
        let glow = self.glow();
        for &prim in self.parts.marks.iter().chain(&self.parts.orbiters) {
            writes.push((prim, Property::Material(glow)));
        }
    }

    fn write_trace(&self, last: Option<&Shown>, next: &Shown, runs: &[f32], writes: &mut Writes) {
        let glow_changed = last.map(|s| s.glow) != Some(next.glow);
        for (index, run) in self.parts.trace.iter().enumerate() {
            let was = last.map_or(-1, |s| s.runs[index]);
            let now = next.runs[index];
            if now > 0 && (glow_changed || was <= 0) {
                writes.push((run.line, Property::Material(self.glow())));
            }
            if now != was {
                // An undrawn run is shrunk on every axis, not only along its
                // own, or its end shows as a square of light at the corner.
                let scale = if now > 0 {
                    Vec3::ONE + run.axis.abs() * runs[index].mul_add(run.length, -1.0)
                } else {
                    Vec3::splat(1.0e-4)
                };
                writes.push((
                    run.pivot,
                    Property::Transform(Transform::new(run.from, Quat::IDENTITY, scale)),
                ));
            }
        }
    }

    fn write_shaft(&self, last: Option<&Shown>, next: &Shown, writes: &mut Writes) {
        let was_on = last.is_some_and(|s| s.shaft > 0);
        if (next.shaft > 0 || was_on)
            && last.map(|s| (s.shaft, s.glow)) != Some((next.shaft, next.glow))
        {
            writes.push((
                self.parts.shaft,
                Property::Material(material::haze(self.color, self.shaft)),
            ));
        }
    }

    fn write_motes(&self, first: bool, writes: &mut Writes) {
        if !self.tick.is_multiple_of(ORBIT_TICKS) || (self.motes <= 0.01 && !first) {
            return;
        }
        for (index, &orbiter) in self.parts.orbiters.iter().enumerate() {
            let index = index as f32;
            let turn = index.mul_add(TAU / ORBITERS as f32, self.orbit);
            let bob = 0.03 * self.orbit.mul_add(1.7, index).sin();
            let height = index.mul_add(0.22, 0.25) + bob;
            writes.push((
                orbiter,
                Property::Transform(Transform::new(
                    Vec3::new(ORBIT_RADIUS * turn.cos(), height, ORBIT_RADIUS * turn.sin()),
                    Quat::from_rotation_y(turn),
                    Vec3::splat(self.motes.max(1.0e-4)),
                )),
            ));
        }
    }

    fn write_ghost(
        &self,
        last: Option<&Shown>,
        next: &Shown,
        lift: f32,
        opacity: f32,
        writes: &mut Writes,
    ) {
        let was_on = last.is_some_and(|s| s.ghost[0] > 0);
        if next.ghost[0] <= 0 && !was_on {
            return;
        }
        if last.map(|s| (s.ghost[0], s.glow)) != Some((next.ghost[0], next.glow)) {
            writes.push((
                self.parts.ghost,
                Property::Material(material::ghost(self.color, opacity)),
            ));
        }
        if !self.carried && last.map(|s| s.ghost) != Some(next.ghost) {
            writes.push((
                self.parts.ghost,
                Property::Transform(Transform::new(
                    Vec3::new(0.0, UNPLUG_RISE * lift, 0.0),
                    Quat::from_rotation_y(self.yaw),
                    Vec3::splat(0.3f32.mul_add(self.ghost, 0.7)),
                )),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPACE_A: DocumentId = (1, 0, 0, 0);
    const SPACE_B: DocumentId = (2, 0, 0, 0);
    const COLOR: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    fn look() -> Look {
        Look::new(Parts {
            marks:    Vec::new(),
            trace:    Vec::new(),
            shaft:    (0, 0),
            orbiters: [(0, 0); ORBITERS],
            ghost:    (1, 1),
        })
    }

    const GHOST: Mood = Mood::Ghost {
        space: SPACE_A,
        color: COLOR,
    };

    #[test]
    fn the_trace_fills_its_runs_in_order() {
        let drawn = coverage([1.0, 2.0, 1.0].into_iter(), 2.0);
        assert_eq!(drawn, vec![1.0, 0.5, 0.0]);
    }

    #[test]
    fn a_ghost_becomes_touchable_only_once_it_is_shown() {
        let mut look = look();
        look.step(GHOST, 0.05);
        assert!(!look.ghost_touchable(), "touchable before it faded in");
        look.step(GHOST, 0.5);
        assert!(look.ghost_touchable());
    }

    #[test]
    fn a_taken_ghost_stays_whole_in_the_hand_until_let_go() {
        let mut look = look();
        look.step(GHOST, 0.5);
        look.unplug(SPACE_A, true);
        for _ in 0..20 {
            look.step(Mood::Idle, 0.1);
        }
        assert!(look.ghost_visible(), "the held ghost dissolved in the hand");
        assert!(
            !look.ghost_touchable(),
            "a pulled ghost cannot be pulled again"
        );

        look.release();
        for _ in 0..20 {
            look.step(Mood::Idle, 0.1);
        }
        assert!(!look.ghost_visible(), "the let-go ghost did not dissolve");
    }

    #[test]
    fn a_let_go_ghost_stays_gone_while_its_pairing_is_still_shown() {
        let mut look = look();
        look.step(GHOST, 0.5);
        look.unplug(SPACE_A, true);
        look.release();
        for _ in 0..20 {
            look.step(GHOST, 0.1);
        }
        assert!(!look.ghost_visible(), "the unplugged ghost came back");
        assert!(!look.ghost_touchable());
    }

    #[test]
    fn a_let_go_ghost_comes_back_for_a_different_pairing() {
        let mut look = look();
        look.step(GHOST, 0.5);
        look.unplug(SPACE_A, true);
        look.release();
        for _ in 0..20 {
            look.step(GHOST, 0.1);
        }
        let other = Mood::Ghost {
            space: SPACE_B,
            color: COLOR,
        };
        look.step(other, 0.5);
        look.step(other, 0.5);
        assert!(look.ghost_touchable(), "a new pairing kept the ghost gone");
    }

    #[test]
    fn a_blinking_pairing_does_not_reset_the_unplug() {
        let mut look = look();
        look.step(GHOST, 0.5);
        look.unplug(SPACE_A, true);
        look.release();
        look.step(Mood::Idle, 0.2);
        look.step(GHOST, 0.2);
        for _ in 0..10 {
            look.step(GHOST, 0.1);
        }
        assert!(!look.ghost_visible(), "a blinking pairing reset the unplug");
    }
}
