//! Runs script instances: instantiation, and the lifecycle calls each tick.

use std::time::Duration;

use bevy::prelude::*;

mod log;

#[cfg(not(target_family = "wasm"))] mod native;
#[cfg(target_family = "wasm")] mod web;

/// How often `fixed-update` runs.
pub const FIXED_INTERVAL: Duration = Duration::from_millis(50);

pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        cfg_select! {
            target_family = "wasm" => {
                app.add_plugins(web::WebEnginePlugin);
            }
            _ => {
                app.add_plugins(native::NativeEnginePlugin);
            }
        }
    }
}

/// Which lifecycle export a call runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TickKind {
    Init,
    Update,
    FixedUpdate,
}

/// The time a lifecycle call carries.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tick {
    pub dt:    f32,
    pub time:  f64,
    pub index: u64,
}

/// One export's call history.
#[derive(Default)]
struct Clock {
    last:  Option<Duration>,
    index: u64,
}

impl Clock {
    fn next(&mut self, now: Duration) -> Tick {
        let dt = self
            .last
            .map_or(Duration::ZERO, |last| now.saturating_sub(last));
        self.last = Some(now);
        let index = self.index;
        self.index += 1;
        Tick {
            dt: dt.as_secs_f32(),
            time: now.as_secs_f64(),
            index,
        }
    }
}

/// When a script's `update` and `fixed-update` last ran.
#[derive(Component, Default)]
pub struct TickClocks {
    update:    Clock,
    fixed:     Clock,
    /// When the next `fixed-update` is due. A call that runs late pulls the
    /// next one earlier, by up to one interval, so the rate holds on average.
    fixed_due: Option<Duration>,
}

impl TickClocks {
    /// Whether `fixed-update` is due at `now`.
    #[must_use]
    pub fn fixed_due(&self, now: Duration) -> bool {
        self.fixed_due.is_none_or(|due| now >= due)
    }

    /// Advances `kind`'s clock to `now`. `init` carries the time alone.
    pub fn next(&mut self, kind: TickKind, now: Duration) -> Tick {
        match kind {
            TickKind::Init => Tick {
                dt:    0.0,
                time:  now.as_secs_f64(),
                index: 0,
            },
            TickKind::Update => self.update.next(now),
            TickKind::FixedUpdate => {
                let due = self.fixed_due.unwrap_or(now) + FIXED_INTERVAL;
                self.fixed_due = Some(due.max(now));
                self.fixed.next(now)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tick_carries_the_time_since_the_last_of_its_kind() {
        let mut clocks = TickClocks::default();
        let ms = Duration::from_millis;
        assert_eq!(clocks.next(TickKind::Update, ms(100)).dt, 0.0);
        let tick = clocks.next(TickKind::Update, ms(116));
        assert!((tick.dt - 0.016).abs() < 1.0e-6);
        assert_eq!(tick.index, 1);
        assert_eq!(
            clocks.next(TickKind::FixedUpdate, ms(116)).index,
            0,
            "each export counts its own calls"
        );
    }

    #[test]
    fn fixed_update_waits_out_its_interval() {
        let mut clocks = TickClocks::default();
        assert!(clocks.fixed_due(Duration::ZERO));
        let _ = clocks.next(TickKind::FixedUpdate, Duration::from_millis(10));
        assert!(!clocks.fixed_due(Duration::from_millis(40)));
        assert!(clocks.fixed_due(Duration::from_millis(60)));
        let _ = clocks.next(TickKind::FixedUpdate, Duration::from_millis(70));
        assert!(
            clocks.fixed_due(Duration::from_millis(110)),
            "a call 10 ms late brings the next one 10 ms closer"
        );
    }
}
