use bevy::{
    platform::collections::HashSet,
    prelude::*,
};

use crate::{
    Portal,
    PortalActiveRender,
    PortalCamera,
    PortalRenderBudget,
    PortalState,
    PortalTargetDoc,
    PortalTargetReceptor,
    PortalViewer,
};

const HYSTERESIS_FACTOR: f32 = 1.1;

pub fn select_active_portals(
    budget: Res<PortalRenderBudget>,
    viewers: Query<
        (Ref<GlobalTransform>, Entity),
        (With<PortalViewer>, With<Camera3d>, Without<PortalCamera>),
    >,
    portals: Query<
        (
            Entity,
            Ref<PortalState>,
            Ref<GlobalTransform>,
            Has<PortalTargetDoc>,
            Has<PortalTargetReceptor>,
        ),
        With<Portal>,
    >,
    actives: Query<Entity, (With<Portal>, With<PortalActiveRender>)>,
    mut commands: Commands,
) {
    let Some((viewer, _)) = viewers.iter().min_by_key(|(_, e)| *e) else {
        return;
    };

    let inputs_changed = budget.is_changed()
        || viewer.is_changed()
        || portals
            .iter()
            .any(|(_, s, t, ..)| s.is_changed() || t.is_changed());
    if !inputs_changed {
        return;
    }

    let origin = viewer.translation();
    let max_d2 = budget.max_distance * budget.max_distance;
    let release_d2 = (budget.max_distance * HYSTERESIS_FACTOR).powi(2);

    let mut candidates: Vec<(Entity, f32)> = portals
        .iter()
        .filter_map(|(e, state, t, has_space, has_receptor)| {
            if *state != PortalState::Open {
                return None;
            }
            // Opaque portals (without a receptor) are cheap to render.
            if has_space && !has_receptor {
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

    for (entity, ..) in &portals {
        let want = chosen.contains(&entity);
        let has = actives.contains(entity);
        if want && !has {
            commands.entity(entity).insert(PortalActiveRender);
        } else if !want && has {
            commands.entity(entity).remove::<PortalActiveRender>();
        }
    }
}
