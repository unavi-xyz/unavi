//! The floating-origin grid: every loaded space gets a cell on a lattice, and
//! the space the local agent stands in is kept at the world origin.

use avian3d::prelude::Position;
use bevy::{
    platform::collections::HashSet,
    prelude::*,
};
use unavi_agent::{
    LocalAgent,
    LocalAgentEntities,
};
use unavi_portal::body::PrevTranslation;

use crate::membership::Space;

/// Side of one grid cell, in metres. A space's content stays within its cell.
pub const SPACE_CELL_SIZE: f32 = 5_000.0;

/// A space's cell on the grid.
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

/// Which grid cells are taken.
#[derive(Resource, Default)]
pub struct SpaceGrid {
    used: HashSet<IVec2>,
}

impl SpaceGrid {
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

/// The space the local agent stands in, which sits at the origin.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct ActiveSpace(pub Option<Entity>);

/// Marks the local agent as not standing in any space, so its position says
/// nothing about where a space is. The client holds it while limbo pins the
/// agent to the limbo floor.
#[derive(Component)]
pub struct Spaceless;

pub fn assign_anchor(
    trigger: On<Add, Space>,
    mut allocator: ResMut<SpaceGrid>,
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
    mut allocator: ResMut<SpaceGrid>,
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

/// Positions every space on the grid lattice relative to the active space,
/// which sits at the origin; [`recenter_active_space`] shifts bodies to match.
pub fn apply_anchor_offsets(
    active: Res<ActiveSpace>,
    mut spaces: Query<(Entity, &SpaceAnchor, &mut Transform), With<Space>>,
) {
    let origin = active
        .0
        .and_then(|e| spaces.get(e).ok())
        .map_or(IVec2::ZERO, |(_, anchor, _)| anchor.grid_cell);

    for (_, anchor, mut transform) in &mut spaces {
        let rel = anchor.grid_cell - origin;
        let target = Vec3::new(
            rel.x as f32 * SPACE_CELL_SIZE,
            0.0,
            rel.y as f32 * SPACE_CELL_SIZE,
        );
        if transform.translation.distance_squared(target) > 1.0e-6 {
            transform.translation = target;
        }
    }
}

/// Promotes the space nearest the local agent's body to active, shifting
/// every body so that space sits at the origin.
pub fn recenter_active_space(
    agents: Query<&LocalAgentEntities, (With<LocalAgent>, Without<Spaceless>)>,
    spaces: Query<(Entity, &Transform), With<Space>>,
    mut bodies: Query<
        (
            Entity,
            &mut Position,
            Option<&mut Transform>,
            Option<&mut PrevTranslation>,
        ),
        Without<Space>,
    >,
    parents: Query<&ChildOf>,
    space_marker: Query<(), With<Space>>,
    position_marker: Query<(), With<Position>>,
    mut active: ResMut<ActiveSpace>,
) {
    let Ok(entities) = agents.single() else {
        return;
    };
    let body = entities.body;

    let pos = match bodies.get(body) {
        Ok((_, _, Some(transform), _)) => transform.translation,
        _ => return,
    };

    let Some((space, offset)) = spaces
        .iter()
        .min_by(|(_, a), (_, b)| {
            a.translation
                .distance_squared(pos)
                .total_cmp(&b.translation.distance_squared(pos))
        })
        .map(|(entity, transform)| (entity, transform.translation))
    else {
        return;
    };

    if active.0 == Some(space) {
        return;
    }
    active.0 = Some(space);

    let delta = -offset;

    let rides_ancestor_shift = |mut entity: Entity| {
        while let Ok(child_of) = parents.get(entity) {
            let parent = child_of.parent();
            if parent == body || space_marker.contains(parent) || position_marker.contains(parent) {
                return true;
            }
            entity = parent;
        }
        false
    };

    for (entity, mut position, transform, prev) in &mut bodies {
        if entity == body {
            continue;
        }
        position.0 += delta;
        if !rides_ancestor_shift(entity) {
            if let Some(mut transform) = transform {
                transform.translation += delta;
            }
            if let Some(mut prev) = prev
                && let Some(p) = prev.0.as_mut()
            {
                *p += delta;
            }
        }
    }

    // The agent body's `Position` is stale after a portal teleport; resync
    // from the shifted `Transform`.
    if let Ok((_, mut position, transform, prev)) = bodies.get_mut(body) {
        if let Some(mut transform) = transform {
            transform.translation += delta;
            position.0 = transform.translation;
        }
        if let Some(mut prev) = prev
            && let Some(p) = prev.0.as_mut()
        {
            *p += delta;
        }
    }
}

/// The first space loaded is active until the agent stands somewhere.
pub fn promote_first_space(
    trigger: On<Add, Space>,
    spaces: Query<(), With<Space>>,
    spaceless: Query<(), With<Spaceless>>,
    mut active: ResMut<ActiveSpace>,
) {
    // A spaceless agent's position is the limbo floor, not a claim about
    // where a space is; promotion waits until it enters one.
    if spaceless.is_empty() && active.0.is_none() && spaces.contains(trigger.entity) {
        active.0 = Some(trigger.entity);
    }
}

#[cfg(test)]
mod tests {
    use bevy::transform::TransformPlugin;
    use iroh_docs::NamespaceId;
    use unavi_portal::crossing::apply_crossings;

    use super::*;

    fn space_hash(seed: &[u8]) -> NamespaceId {
        NamespaceId::from(blake3::hash(seed).as_bytes())
    }

    fn setup() -> App {
        let mut app = App::new();
        app.add_plugins(TransformPlugin)
            .init_resource::<SpaceGrid>()
            .init_resource::<ActiveSpace>()
            .add_observer(assign_anchor)
            .add_observer(promote_first_space)
            .add_observer(release_anchor)
            .add_systems(
                PostUpdate,
                (recenter_active_space, apply_anchor_offsets)
                    .chain()
                    .after(apply_crossings)
                    .before(TransformSystems::Propagate),
            );
        app
    }

    fn spawn_space(app: &mut App, seed: &[u8]) -> Entity {
        app.world_mut()
            .spawn((
                Space(space_hash(seed).into()),
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id()
    }

    fn translation(app: &App, entity: Entity) -> Vec3 {
        app.world()
            .get::<Transform>(entity)
            .expect("transform")
            .translation
    }

    fn rel_offset(app: &App, space: Entity, origin: Entity) -> Vec3 {
        let cell = app
            .world()
            .get::<SpaceAnchor>(space)
            .expect("anchor")
            .grid_cell;
        let origin_cell = app
            .world()
            .get::<SpaceAnchor>(origin)
            .expect("anchor")
            .grid_cell;
        let rel = cell - origin_cell;
        Vec3::new(
            rel.x as f32 * SPACE_CELL_SIZE,
            0.0,
            rel.y as f32 * SPACE_CELL_SIZE,
        )
    }

    #[test]
    fn active_space_sits_at_origin() {
        let mut app = setup();
        let a = spawn_space(&mut app, b"a");
        let b = spawn_space(&mut app, b"b");

        assert_eq!(app.world().resource::<ActiveSpace>().0, Some(a));
        app.update();

        let expected = rel_offset(&app, b, a);
        assert_eq!(translation(&app, a), Vec3::ZERO);
        assert_eq!(translation(&app, b), expected);
        assert_ne!(expected, Vec3::ZERO);
    }

    #[test]
    fn a_spaceless_agent_promotes_no_space() {
        let mut app = setup();
        let tracked_head = app.world_mut().spawn_empty().id();
        let body = app
            .world_mut()
            .spawn((
                Transform::from_translation(Vec3::new(SPACE_CELL_SIZE, 0.0, SPACE_CELL_SIZE)),
                Position(Vec3::new(SPACE_CELL_SIZE, 0.0, SPACE_CELL_SIZE)),
            ))
            .id();
        app.world_mut().spawn((
            LocalAgent,
            LocalAgentEntities { body, tracked_head },
            Spaceless,
        ));

        let a = spawn_space(&mut app, b"a");
        app.update();

        assert_eq!(app.world().resource::<ActiveSpace>().0, None);
        assert_ne!(translation(&app, a), Vec3::ZERO);
    }

    #[test]
    fn recenter_shifts_world_so_occupied_space_is_origin() {
        let mut app = setup();
        let a = spawn_space(&mut app, b"a");
        let b = spawn_space(&mut app, b"b");
        app.update();

        let b_pos = translation(&app, b);
        assert_eq!(app.world().resource::<ActiveSpace>().0, Some(a));

        let local = Vec3::new(2.0, 0.0, -1.0);
        let stale = Vec3::new(7.0, 0.0, 7.0);
        let body = app
            .world_mut()
            .spawn((
                Transform::from_translation(b_pos + local),
                GlobalTransform::default(),
                PrevTranslation(Some(b_pos + local)),
                Position(stale),
            ))
            .id();
        let other_start = Vec3::new(123.0, 0.0, 0.0);
        let other = app
            .world_mut()
            .spawn((Transform::default(), Position(other_start)))
            .id();
        let tracked_head = app.world_mut().spawn_empty().id();
        app.world_mut()
            .spawn((LocalAgent, LocalAgentEntities { body, tracked_head }));

        app.update();

        assert_eq!(app.world().resource::<ActiveSpace>().0, Some(b));
        assert_eq!(translation(&app, b), Vec3::ZERO);

        let read = |e| app.world().get::<Position>(e).expect("position").0;
        assert!((translation(&app, body) - local).length() < 1.0e-4);
        assert!((read(body) - local).length() < 1.0e-4);
        assert!((read(other) - (other_start - b_pos)).length() < 1.0e-4);
    }

    #[test]
    fn root_body_transform_shifts_same_frame() {
        let mut app = setup();
        let a = spawn_space(&mut app, b"a");
        let b = spawn_space(&mut app, b"b");
        app.update();

        let b_pos = translation(&app, b);
        assert_eq!(app.world().resource::<ActiveSpace>().0, Some(a));

        let local = Vec3::new(1.0, 0.0, 2.0);
        let body = app
            .world_mut()
            .spawn((
                Transform::from_translation(b_pos + local),
                GlobalTransform::default(),
                PrevTranslation(Some(b_pos + local)),
                Position(b_pos + local),
            ))
            .id();

        // A root body's `Transform` must shift this frame, before any
        // fixed-tick writeback.
        let world = Vec3::new(50.0, 0.0, -10.0);
        let root = app
            .world_mut()
            .spawn((
                Transform::from_translation(world),
                GlobalTransform::default(),
                Position(world),
            ))
            .id();
        let tracked_head = app.world_mut().spawn_empty().id();
        app.world_mut()
            .spawn((LocalAgent, LocalAgentEntities { body, tracked_head }));

        app.update();

        assert_eq!(app.world().resource::<ActiveSpace>().0, Some(b));
        let read = |e| app.world().get::<Position>(e).expect("position").0;
        assert!((translation(&app, root) - (world - b_pos)).length() < 1.0e-4);
        assert!((read(root) - (world - b_pos)).length() < 1.0e-4);
    }

    #[test]
    fn child_body_transform_rides_parent() {
        let mut app = setup();
        spawn_space(&mut app, b"a");
        let b = spawn_space(&mut app, b"b");
        app.update();

        let b_pos = translation(&app, b);

        let local = Vec3::new(1.0, 0.0, 2.0);
        let body = app
            .world_mut()
            .spawn((
                Transform::from_translation(b_pos + local),
                GlobalTransform::default(),
                PrevTranslation(Some(b_pos + local)),
                Position(b_pos + local),
            ))
            .id();

        // A body parented to a space rides its transform, so only `Position`
        // shifts.
        let child_local = Vec3::new(3.0, 0.0, 4.0);
        let child = app
            .world_mut()
            .spawn((
                Transform::from_translation(child_local),
                GlobalTransform::default(),
                Position(b_pos + child_local),
                ChildOf(b),
            ))
            .id();
        let tracked_head = app.world_mut().spawn_empty().id();
        app.world_mut()
            .spawn((LocalAgent, LocalAgentEntities { body, tracked_head }));

        app.update();

        assert_eq!(app.world().resource::<ActiveSpace>().0, Some(b));
        let read = |e| app.world().get::<Position>(e).expect("position").0;
        assert!((translation(&app, child) - child_local).length() < 1.0e-4);
        assert!((read(child) - child_local).length() < 1.0e-4);
    }

    #[test]
    fn world_doc_under_plain_parent_shifts_local() {
        let mut app = setup();
        spawn_space(&mut app, b"a");
        let b = spawn_space(&mut app, b"b");
        app.update();

        let b_pos = translation(&app, b);

        let local = Vec3::new(1.0, 0.0, 2.0);
        let body = app
            .world_mut()
            .spawn((
                Transform::from_translation(b_pos + local),
                GlobalTransform::default(),
                PrevTranslation(Some(b_pos + local)),
                Position(b_pos + local),
            ))
            .id();

        // A prim under a plain (non-physics) container rides nothing; its local
        // `Transform` must shift directly this frame.
        let root = app
            .world_mut()
            .spawn((Transform::default(), GlobalTransform::default()))
            .id();
        let prim_local = Vec3::new(5.0, 0.0, 6.0);
        let prim = app
            .world_mut()
            .spawn((
                Transform::from_translation(prim_local),
                GlobalTransform::default(),
                Position(prim_local),
                ChildOf(root),
            ))
            .id();
        let tracked_head = app.world_mut().spawn_empty().id();
        app.world_mut()
            .spawn((LocalAgent, LocalAgentEntities { body, tracked_head }));

        app.update();

        assert_eq!(app.world().resource::<ActiveSpace>().0, Some(b));
        let read = |e| app.world().get::<Position>(e).expect("position").0;
        assert!((translation(&app, prim) - (prim_local - b_pos)).length() < 1.0e-4);
        assert!((read(prim) - (prim_local - b_pos)).length() < 1.0e-4);
    }

    #[test]
    fn nested_physics_body_rides_parent_body() {
        let mut app = setup();
        spawn_space(&mut app, b"a");
        let b = spawn_space(&mut app, b"b");
        app.update();

        let b_pos = translation(&app, b);

        let local = Vec3::new(1.0, 0.0, 2.0);
        let body = app
            .world_mut()
            .spawn((
                Transform::from_translation(b_pos + local),
                GlobalTransform::default(),
                PrevTranslation(Some(b_pos + local)),
                Position(b_pos + local),
            ))
            .id();

        let root = app
            .world_mut()
            .spawn((Transform::default(), GlobalTransform::default()))
            .id();
        let parent_local = Vec3::new(5.0, 0.0, 6.0);
        let parent_body = app
            .world_mut()
            .spawn((
                Transform::from_translation(parent_local),
                GlobalTransform::default(),
                Position(parent_local),
                ChildOf(root),
            ))
            .id();
        // Nested under another physics body: the parent's shift propagates, so
        // the local `Transform` stays put to avoid double-shifting.
        let nested_local = Vec3::new(0.5, 0.0, 0.5);
        let nested = app
            .world_mut()
            .spawn((
                Transform::from_translation(nested_local),
                GlobalTransform::default(),
                Position(parent_local + nested_local),
                ChildOf(parent_body),
            ))
            .id();
        let tracked_head = app.world_mut().spawn_empty().id();
        app.world_mut()
            .spawn((LocalAgent, LocalAgentEntities { body, tracked_head }));

        app.update();

        assert_eq!(app.world().resource::<ActiveSpace>().0, Some(b));
        let read = |e| app.world().get::<Position>(e).expect("position").0;
        assert!((translation(&app, parent_body) - (parent_local - b_pos)).length() < 1.0e-4);
        assert!((translation(&app, nested) - nested_local).length() < 1.0e-4);
        // Positions are global, so both shift uniformly.
        assert!((read(parent_body) - (parent_local - b_pos)).length() < 1.0e-4);
        assert!((read(nested) - (parent_local + nested_local - b_pos)).length() < 1.0e-4);
    }
}
