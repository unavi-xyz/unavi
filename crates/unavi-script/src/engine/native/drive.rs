use std::{
    sync::{
        Arc,
        atomic::{
            AtomicUsize,
            Ordering,
        },
    },
    time::{
        Duration,
        Instant,
    },
};

use bevy::prelude::*;
use unavi_util::async_commands::pump_async_commands;

// ponytail: fixed budget window; wire to the monitor refresh rate if pacing
// ever needs to be tighter than last-frame delta.
const BUDGET_MAX: Duration = Duration::from_millis(5);
const BUDGET_MIN: Duration = Duration::from_millis(1);
const FRAME_RESERVE: Duration = Duration::from_millis(2);
const FRAME_MIN: Duration = Duration::from_millis(8);
const FRAME_MAX: Duration = Duration::from_millis(33);

/// Time to wait for the frame's scripts to finish, shrinking as the frame's
/// remaining budget is consumed so a slow frame is not made slower.
pub fn script_budget(time: &Time<Real>) -> Duration {
    let target = time.delta().clamp(FRAME_MIN, FRAME_MAX);
    let elapsed = time.last_update().map(|t| t.elapsed()).unwrap_or_default();
    budget(target, elapsed)
}

fn budget(target: Duration, elapsed: Duration) -> Duration {
    target
        .saturating_sub(elapsed)
        .saturating_sub(FRAME_RESERVE)
        .clamp(BUDGET_MIN, BUDGET_MAX)
}

/// Blocks the main thread until every script spawned this frame reports done or
/// the budget elapses, pumping the async command queue so awaiting host calls
/// can make progress instead of deadlocking against the blocked main thread.
pub fn wait_for_scripts(world: &mut World, outstanding: &Arc<AtomicUsize>, budget: Duration) {
    if outstanding.load(Ordering::Acquire) == 0 {
        return;
    }
    let deadline = Instant::now() + budget;
    while outstanding.load(Ordering::Acquire) > 0 {
        pump_async_commands(world);
        if outstanding.load(Ordering::Acquire) == 0 || Instant::now() >= deadline {
            break;
        }
        std::thread::yield_now();
    }
    pump_async_commands(world);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_shrinks_with_elapsed_frame_time() {
        let ms = Duration::from_millis;
        // Fresh frame: capped at the max window.
        assert_eq!(budget(ms(16), ms(0)), BUDGET_MAX);
        // Late in the frame: only the slice up to the reserve remains.
        assert_eq!(budget(ms(16), ms(12)), ms(2));
        // Frame already overrun: floored, never negative.
        assert_eq!(budget(ms(16), ms(50)), BUDGET_MIN);
    }
}
