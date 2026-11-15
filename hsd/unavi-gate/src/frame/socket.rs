//! The black stone and the socket in its top, all beneath the socket prim: the
//! recess and its rim, the square of light around it, the cross on its faces,
//! the shaft and motes over it, and the ghost.

use wired_guest::{
    beacon::BEACON_SIZE,
    math::{
        Transform,
        Vec3,
    },
};

use super::{
    ORBITERS,
    SEAT_FLOOR,
    SHAFT_FOOT,
    STONE_WIDTH,
    block,
    material,
    place,
    route,
    span,
    trim_span,
};
use crate::{
    Batch,
    unavi::shapes::api::Cuboid,
    wired::scene::{
        document::{
            Document,
            Layer,
        },
        properties::{
            Material,
            Property,
        },
    },
};

/// The recess a beacon sits in.
const RECESS: f32 = BEACON_SIZE + 0.035;
const RIM_HEIGHT: f32 = 0.05;
/// Seat point up to the top of the rim.
pub const RIM_TOP: f32 = RIM_HEIGHT - SEAT_FLOOR;

/// A band of black glass a little proud of the stone, under its rim.
const COLLAR: f32 = 0.03;
const COLLAR_PROUD: f32 = 0.008;

const INLAY: f32 = 0.008;
const INLAY_PROUD: f32 = 0.003;
const CROSS_ARM: f32 = 0.07;
const CROSS_BAR: f32 = 0.014;
/// Seat point down to the cross on the stone's faces.
pub const CROSS_DROP: f32 = 0.16;
/// Seat point down to the foot of the cross, where the trace starts.
pub const CROSS_FOOT: f32 = CROSS_DROP + CROSS_ARM / 2.0;

const SHAFT_WIDTH: f32 = 0.014;
const SHAFT_HEIGHT: f32 = 1.6;
const ORBITER_SIZE: f32 = 0.016;

pub struct SocketParts {
    pub marks:    Vec<(u64, u64)>,
    pub shaft:    (u64, u64),
    pub orbiters: [(u64, u64); ORBITERS],
    pub ghost:    (u64, u64),
}

pub fn build<'a>(
    doc: &Document,
    batch: Batch<'a>,
    socket: (u64, u64),
    seat: Vec3,
    dark: Material,
) -> anyhow::Result<(Batch<'a>, SocketParts)> {
    let batch = stone(batch, socket, seat)?;
    let (batch, marks) = marks(doc, batch, socket, dark)?;

    let shaft = Cuboid::new(Vec3::new(SHAFT_WIDTH, SHAFT_HEIGHT, SHAFT_WIDTH)).mesh()?;
    let mut batch = place(
        batch,
        shaft,
        socket,
        Vec3::new(0.0, SHAFT_FOOT + SHAFT_HEIGHT / 2.0, 0.0),
        material::haze(material::NEUTRAL, 0.0),
    );
    let mut orbiters = [(0, 0); ORBITERS];
    for orbiter in &mut orbiters {
        *orbiter = Cuboid::new(Vec3::splat(ORBITER_SIZE)).mesh()?;
        batch = place(batch, *orbiter, socket, Vec3::ZERO, dark).set(
            *orbiter,
            Property::Transform(Transform::new(
                Vec3::ZERO,
                wired_guest::math::Quat::IDENTITY,
                Vec3::splat(1.0e-4),
            )),
        );
    }
    let ghost = Cuboid::new(Vec3::splat(BEACON_SIZE * 0.92)).mesh()?;
    let batch = place(
        batch,
        ghost,
        socket,
        Vec3::ZERO,
        material::ghost(material::NEUTRAL, 0.0),
    );
    Ok((
        batch,
        SocketParts {
            marks,
            shaft,
            orbiters,
            ghost,
        },
    ))
}

/// The black stone from its foot up to the recess floor, its collar, and the
/// rim around the recess. Positions are in the socket's frame.
fn stone(batch: Batch<'_>, socket: (u64, u64), seat: Vec3) -> anyhow::Result<Batch<'_>> {
    let black = material::black();
    let half = STONE_WIDTH / 2.0;
    let foot = route::foot(seat) - seat.y;
    let mut batch = span(
        batch,
        socket,
        Vec3::new(-half, foot, -half),
        Vec3::new(half, -SEAT_FLOOR, half),
        black,
    )?;
    let collar = half + COLLAR_PROUD;
    batch = trim_span(
        batch,
        socket,
        Vec3::new(-collar, -SEAT_FLOOR - COLLAR, -collar),
        Vec3::new(collar, -SEAT_FLOOR, collar),
        black,
    )?;
    let wall = (STONE_WIDTH - RECESS) / 2.0;
    let rim_y = f32::midpoint(RIM_TOP, -SEAT_FLOOR);
    let rim_at = f32::midpoint(RECESS, wall);
    for side in [-1.0_f32, 1.0] {
        batch = block(
            batch,
            socket,
            Vec3::new(wall, RIM_HEIGHT, STONE_WIDTH),
            Vec3::new(side * rim_at, rim_y, 0.0),
            black,
        )?;
        batch = block(
            batch,
            socket,
            Vec3::new(RECESS, RIM_HEIGHT, wall),
            Vec3::new(0.0, rim_y, side * rim_at),
            black,
        )?;
    }
    Ok(batch)
}

/// The square of light around the recess, and the beacon's cross on the
/// stone's front and back faces. Answers every bar.
fn marks<'a>(
    doc: &Document,
    mut batch: Batch<'a>,
    socket: (u64, u64),
    dark: Material,
) -> anyhow::Result<(Batch<'a>, Vec<(u64, u64)>)> {
    let mut bars = Vec::with_capacity(8);
    let edge = RECESS / 2.0 + INLAY;
    let length = INLAY.mul_add(3.0, RECESS);
    let y = RIM_TOP + INLAY_PROUD / 2.0;
    for side in [-1.0_f32, 1.0] {
        for (size, at) in [
            (
                Vec3::new(INLAY, INLAY_PROUD, length),
                Vec3::new(side * edge, y, 0.0),
            ),
            (
                Vec3::new(length, INLAY_PROUD, INLAY),
                Vec3::new(0.0, y, side * edge),
            ),
        ] {
            let bar = Cuboid::new(size).mesh()?;
            batch = place(batch, bar, socket, at, dark);
            bars.push(bar);
        }
    }

    for side in [-1.0_f32, 1.0] {
        let cross = doc.create_prim(Layer::Local, Some(socket))?;
        let face = side * (STONE_WIDTH / 2.0 + INLAY_PROUD / 2.0);
        batch = batch.set(
            cross,
            Property::Transform(Transform::from_translation(Vec3::new(
                0.0,
                -CROSS_DROP,
                face,
            ))),
        );
        for size in [
            Vec3::new(CROSS_ARM, CROSS_BAR, INLAY_PROUD),
            Vec3::new(CROSS_BAR, CROSS_ARM, INLAY_PROUD),
        ] {
            let bar = Cuboid::new(size).mesh()?;
            batch = place(batch, bar, cross, Vec3::ZERO, dark);
            bars.push(bar);
        }
    }
    Ok((batch, bars))
}
