use bevy::{
    platform::collections::HashSet,
    prelude::*,
};
use bevy_hsd::{
    Hsd,
    attributes::spawn::SpawnPoint,
};
use unavi_agent::LocalAgent;
use unavi_portal::teleport::PortalTeleport;

use crate::{
    Space,
    membership::SpaceOwner,
    spawn::pick_spawn,
};

#[derive(Component, Default)]
pub struct DocTraveler;

pub const SPACE_CELL_SIZE: f32 = 10_000.0;

#[derive(Component, Debug, Clone, Copy)]
pub struct SpaceAnchor {
    pub grid_cell: IVec2,
}

impl SpaceAnchor {
    #[must_use]
    pub fn offset(&self) -> Vec3 {
        Vec3::new(
            self.grid_cell.x as f32 * SPACE_CELL_SIZE,
            0.0,
            self.grid_cell.y as f32 * SPACE_CELL_SIZE,
        )
    }
}

#[derive(Resource, Default)]
pub struct SpaceGridAllocator {
    used: HashSet<IVec2>,
}

impl SpaceGridAllocator {
    pub fn allocate(&mut self) -> IVec2 {
        for radius in 1_i32.. {
            for x in -radius..=radius {
                for y in -radius..=radius {
                    if x.abs() != radius && y.abs() != radius {
                        continue;
                    }
                    let cell = IVec2::new(x, y);
                    if self.used.insert(cell) {
                        return cell;
                    }
                }
            }
        }
        unreachable!()
    }

    pub fn release(&mut self, cell: IVec2) {
        self.used.remove(&cell);
    }
}

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct ActiveSpace(pub Option<Entity>);

pub fn assign_anchor(
    trigger: On<Add, Space>,
    mut allocator: ResMut<SpaceGridAllocator>,
    mut commands: Commands,
) {
    let cell = allocator.allocate();
    commands
        .entity(trigger.entity)
        .insert(SpaceAnchor { grid_cell: cell });
}

pub fn release_anchor(
    trigger: On<Remove, Space>,
    anchors: Query<&SpaceAnchor>,
    active: Option<ResMut<ActiveSpace>>,
    mut allocator: ResMut<SpaceGridAllocator>,
) {
    if let Ok(anchor) = anchors.get(trigger.entity) {
        allocator.release(anchor.grid_cell);
    }
    if let Some(mut active) = active
        && active.0 == Some(trigger.entity)
    {
        active.0 = None;
    }
}

pub fn apply_anchor_offsets(
    active: Res<ActiveSpace>,
    mut spaces: Query<(Entity, &SpaceAnchor, &mut Transform), With<Space>>,
) {
    for (entity, anchor, mut transform) in &mut spaces {
        let target = if active.0 == Some(entity) {
            Vec3::ZERO
        } else {
            anchor.offset()
        };
        if transform.translation.distance_squared(target) > 1.0e-6 {
            transform.translation = target;
        }
    }
}

pub fn reparent_doc_traveler(
    trigger: On<PortalTeleport>,
    travelers: Query<(), (With<Hsd>, With<DocTraveler>)>,
    parents: Query<&ChildOf>,
    spaces: Query<(), With<Space>>,
    mut transforms: Query<&mut Transform>,
    mut commands: Commands,
) {
    let event = trigger.event();
    if !travelers.contains(event.entity) {
        return;
    }

    let mut current = event.destination;
    let dest_space = loop {
        if spaces.contains(current) {
            break Some(current);
        }
        match parents.get(current) {
            Ok(child_of) => current = child_of.parent(),
            Err(_) => break None,
        }
    };

    let Some(space) = dest_space else {
        return;
    };

    commands
        .entity(event.entity)
        .insert((ChildOf(space), SpaceOwner(space)));
    if let Ok(mut transform) = transforms.get_mut(event.entity) {
        transform.translation = Vec3::ZERO;
        transform.rotation = Quat::IDENTITY;
    }
}

pub fn promote_active_on_teleport(
    trigger: On<PortalTeleport>,
    local_agents: Query<(), With<LocalAgent>>,
    spaces_marker: Query<(), With<Space>>,
    spaces: Query<&GlobalTransform, With<Space>>,
    spawn_points: Query<(&SpawnPoint, &GlobalTransform, &ChildOf)>,
    parents: Query<&ChildOf>,
    mut transforms: Query<&mut Transform>,
    mut active: ResMut<ActiveSpace>,
) {
    let event = trigger.event();
    if !local_agents.contains(event.entity) {
        return;
    }
    if !spaces_marker.contains(event.destination) {
        return;
    }
    if active.0 == Some(event.destination) {
        return;
    }
    active.0 = Some(event.destination);
    if let Ok(mut transform) = transforms.get_mut(event.entity) {
        // The newly active space is being snapped to the world origin this
        // frame, so spawn coordinates are taken in the space's local frame.
        transform.translation =
            pick_spawn(event.destination, &spawn_points, &parents, &spaces).unwrap_or(Vec3::ZERO);
    }
}

pub fn promote_first_space(
    trigger: On<Add, Space>,
    spaces: Query<(), With<Space>>,
    mut active: ResMut<ActiveSpace>,
) {
    if active.0.is_none() && spaces.contains(trigger.entity) {
        active.0 = Some(trigger.entity);
    }
}
