//! Tells the page's loader the app has started drawing, so it can lift its
//! cover.

use bevy::prelude::*;

/// Frames to let pass after startup, so the loader's cover lifts onto a drawn
/// scene instead of the first blank frames.
const READY_FRAMES: u32 = 3;

const READY_EVENT: &str = "unavi-ready";

pub struct ReadyPlugin;

impl Plugin for ReadyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FramesUntilReady(READY_FRAMES))
            .add_systems(
                Update,
                signal_ready.run_if(resource_exists::<FramesUntilReady>),
            );
    }
}

#[derive(Resource)]
struct FramesUntilReady(u32);

fn signal_ready(mut frames: ResMut<FramesUntilReady>, mut commands: Commands) {
    frames.0 = frames.0.saturating_sub(1);
    if frames.0 > 0 {
        return;
    }
    commands.remove_resource::<FramesUntilReady>();

    let Some(window) = web_sys::window() else {
        warn!("No window to signal readiness on");
        return;
    };
    let event = match web_sys::Event::new(READY_EVENT) {
        Ok(event) => event,
        Err(err) => {
            warn!(?err, "Failed to create the ready event");
            return;
        }
    };
    if let Err(err) = window.dispatch_event(&event) {
        warn!(?err, "Failed to dispatch the ready event");
    }
}
