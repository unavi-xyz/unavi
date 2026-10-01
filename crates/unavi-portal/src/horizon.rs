use bevy::{
    platform::collections::HashSet,
    prelude::*,
};

use crate::{
    DevelopCamera,
    DevelopmentHorizon,
    GluedTo,
    PortalViewer,
    Seam,
    SeamActiveRender,
    SeamState,
};

const HYSTERESIS_FACTOR: f32 = 1.1;

pub fn select_developed_seams(
    budget: Res<DevelopmentHorizon>,
    viewers: Query<
        Ref<GlobalTransform>,
        (With<PortalViewer>, With<Camera3d>, Without<DevelopCamera>),
    >,
    seams: Query<
        (
            Entity,
            Ref<SeamState>,
            Ref<GlobalTransform>,
            Option<&GluedTo>,
        ),
        With<Seam>,
    >,
    actives: Query<Entity, (With<Seam>, With<SeamActiveRender>)>,
    mut commands: Commands,
) {
    let Ok(viewer) = viewers.single() else {
        return;
    };

    let inputs_changed = budget.is_changed()
        || viewer.is_changed()
        || seams
            .iter()
            .any(|(_, s, t, ..)| s.is_changed() || t.is_changed());
    if !inputs_changed {
        return;
    }

    let origin = viewer.translation();
    let max_d2 = budget.max_distance * budget.max_distance;
    let release_d2 = (budget.max_distance * HYSTERESIS_FACTOR).powi(2);

    let mut candidates: Vec<(Entity, f32)> = seams
        .iter()
        .filter_map(|(e, state, t, glued)| {
            if *state != SeamState::Open {
                return None;
            }
            // A seam glued to a document root rather than to another seam is
            // opaque, and never developed.
            if !glued.is_some_and(|g| seams.contains(g.0)) {
                return None;
            }
            let d2 = t.translation().distance_squared(origin);
            let cutoff = if actives.contains(e) {
                release_d2
            } else {
                max_d2
            };
            (d2 <= cutoff).then_some((e, d2))
        })
        .collect();
    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    candidates.truncate(budget.max_active);

    let chosen: HashSet<Entity> = candidates.iter().map(|(e, _)| *e).collect();

    for (entity, ..) in &seams {
        let want = chosen.contains(&entity);
        let has = actives.contains(entity);
        if want && !has {
            commands.entity(entity).insert(SeamActiveRender);
        } else if !want && has {
            commands.entity(entity).remove::<SeamActiveRender>();
        }
    }
}
