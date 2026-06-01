use bevy::prelude::*;

use crate::{
    Portal,
    PortalActiveRender,
    PortalCamera,
    PortalRenderBudget,
    PortalState,
    PortalTargetDoc,
    PortalTargetReceptor,
};

pub fn select_active_portals(
    budget: Res<PortalRenderBudget>,
    cameras: Query<&GlobalTransform, (With<Camera3d>, Without<PortalCamera>)>,
    portals: Query<
        (
            Entity,
            &PortalState,
            &GlobalTransform,
            Has<PortalTargetDoc>,
            Has<PortalTargetReceptor>,
        ),
        With<Portal>,
    >,
    actives: Query<Entity, (With<Portal>, With<PortalActiveRender>)>,
    mut commands: Commands,
) {
    let Some(viewer) = cameras.iter().next() else {
        return;
    };
    let origin = viewer.translation();
    let max_d2 = budget.max_distance * budget.max_distance;

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
            (d2 <= max_d2).then_some((e, d2))
        })
        .collect();
    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    candidates.truncate(budget.max_active);

    let chosen: bevy::platform::collections::HashSet<Entity> =
        candidates.iter().map(|(e, _)| *e).collect();

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
