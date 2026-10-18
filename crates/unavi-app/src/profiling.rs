//! Frame-time sampling for a build with the `profiling` feature.

use std::cmp::Ordering;

use bevy::{
    diagnostic::{
        Diagnostic,
        DiagnosticsStore,
        EntityCountDiagnosticsPlugin,
        FrameTimeDiagnosticsPlugin,
    },
    prelude::*,
};

/// Seconds between summaries of the collected window.
const REPORT_SECS: f32 = 5.0;

pub struct ProfilingPlugin;

impl Plugin for ProfilingPlugin {
    fn build(&self, app: &mut App) {
        // A wasm build samples only when the page URL carries `?profile=1`; a
        // native build samples whenever the feature is compiled in.
        let enabled = cfg_select! {
            target_family = "wasm" => profile_param(),
            _ => true,
        };
        if !enabled {
            return;
        }
        app.add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            EntityCountDiagnosticsPlugin::default(),
        ))
        .add_systems(Update, report);
    }
}

#[cfg(target_family = "wasm")]
fn profile_param() -> bool {
    web_sys::window()
        .and_then(|window| window.location().search().ok())
        .and_then(|search| web_sys::UrlSearchParams::new_with_str(&search).ok())
        .is_some_and(|params| params.get("profile").as_deref() == Some("1"))
}

fn report(
    diagnostics: Res<DiagnosticsStore>,
    time: Res<Time>,
    mut window: Local<Vec<f64>>,
    mut since_report: Local<f32>,
) {
    if let Some(ms) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(Diagnostic::value)
    {
        window.push(ms);
    }

    *since_report += time.delta_secs();
    if *since_report < REPORT_SECS || window.is_empty() {
        return;
    }
    *since_report = 0.0;

    let entities = diagnostics
        .get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT)
        .and_then(Diagnostic::value)
        .map_or(0, |count| count as u64);

    let mut sorted = window.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

    info!(
        frames = sorted.len(),
        p50 = percentile(&sorted, 0.50),
        p95 = percentile(&sorted, 0.95),
        max = sorted.last().copied().unwrap_or_default(),
        entities,
        "frame time (ms)"
    );
    window.clear();
}

/// The value at `quantile` of an ascending slice.
fn percentile(sorted: &[f64], quantile: f64) -> f64 {
    let last = sorted.len().saturating_sub(1);
    let index = (last as f64 * quantile).round() as usize;
    sorted.get(index).copied().unwrap_or_default()
}
