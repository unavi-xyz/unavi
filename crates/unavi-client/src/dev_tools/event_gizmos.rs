use std::sync::LazyLock;

use bevy::{
    color::palettes::tailwind,
    math::Isometry3d,
    prelude::*,
};
use parking_lot::Mutex;
use unavi_script::debug::{
    EMIT_OBSERVER,
    spatial_receptors,
};

const EVENT_LIFETIME: f32 = 0.8;
const PENDING_CAP: usize = 256;

struct EmittedSpatialEvent {
    position: Vec3,
    radius:   f32,
}

static PENDING: LazyLock<Mutex<Vec<EmittedSpatialEvent>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

pub fn install_emit_observer() {
    *EMIT_OBSERVER.write() = Some(Box::new(|channel, position, radius| {
        debug!("debug-event: emit {channel} @ {position:?} r={radius}");
        let mut q = PENDING.lock();
        if q.len() >= PENDING_CAP {
            return;
        }
        q.push(EmittedSpatialEvent { position, radius });
    }));
}

#[derive(Resource, Default)]
pub struct EventPings(Vec<(EmittedSpatialEvent, f32)>);

pub fn draw_receptors(mut gizmos: Gizmos) {
    for r in spatial_receptors() {
        gizmos
            .sphere(
                Isometry3d::from_translation(r.position),
                r.radius,
                tailwind::SKY_400,
            )
            .resolution(24);
    }
}

pub fn update_event_pings(time: Res<Time>, mut pings: ResMut<EventPings>) {
    let delta = time.delta_secs();
    pings.0.retain_mut(|(_, age)| {
        *age += delta;
        *age < EVENT_LIFETIME
    });
    let drained = std::mem::take(&mut *PENDING.lock());
    pings.0.extend(drained.into_iter().map(|ev| (ev, 0.0)));
}

pub fn draw_event_pings(pings: Res<EventPings>, mut gizmos: Gizmos) {
    for (ev, age) in &pings.0 {
        let t = (age / EVENT_LIFETIME).clamp(0.0, 1.0);
        let mut color: Srgba = tailwind::AMBER_400;
        color.alpha = (1.0 - t).powi(2);
        gizmos
            .sphere(Isometry3d::from_translation(ev.position), ev.radius, color)
            .resolution(24);
    }
}
