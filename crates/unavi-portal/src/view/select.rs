use bevy::{
    platform::collections::HashSet,
    prelude::*,
};

use crate::{
    body::PortalViewer,
    destination::Destination,
    portal::{
        Portal,
        PortalState,
    },
    view::{
        ViewBudget,
        Viewed,
        camera::PortalViewCamera,
    },
};

const HYSTERESIS_FACTOR: f32 = 1.1;

pub fn select_viewed_portals(
    budget: Res<ViewBudget>,
    viewers: Query<
        Ref<GlobalTransform>,
        (
            With<PortalViewer>,
            With<Camera3d>,
            Without<PortalViewCamera>,
        ),
    >,
    portals: Query<
        (
            Entity,
            Ref<PortalState>,
            Ref<GlobalTransform>,
            Option<&Destination>,
        ),
        With<Portal>,
    >,
    actives: Query<Entity, (With<Portal>, With<Viewed>)>,
    mut commands: Commands,
) {
    let Ok(viewer) = viewers.single() else {
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
        .filter_map(|(e, state, t, destination)| {
            if *state != PortalState::Open {
                return None;
            }
            // A portal glued to a document root rather than to another
            // portal is opaque, and never viewed live.
            if !destination.is_some_and(|d| portals.contains(d.0)) {
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
    candidates.sort_by(|a, b| a.1.total_cmp(&b.1));
    candidates.truncate(budget.max_active);

    let chosen: HashSet<Entity> = candidates.iter().map(|(e, _)| *e).collect();

    for (entity, ..) in &portals {
        let want = chosen.contains(&entity);
        let has = actives.contains(entity);
        if want && !has {
            commands.entity(entity).insert(Viewed);
        } else if !want && has {
            commands.entity(entity).remove::<Viewed>();
        }
    }
}
