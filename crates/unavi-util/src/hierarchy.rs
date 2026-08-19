use bevy::prelude::*;

/// Longest chain a walk will follow before giving up.
///
/// Scene hierarchies here are prims under documents under spaces, so nothing
/// legitimate comes close. Reaching it means a `ChildOf` cycle, which without
/// a cap is an unbounded loop on whichever thread the walk runs on.
const MAX_DEPTH: usize = 256;

/// Walks `entity` and then its ancestors, nearest first.
pub fn ancestors<'a>(
    entity: Entity,
    parents: &'a Query<&ChildOf>,
) -> impl Iterator<Item = Entity> + 'a {
    let mut current = Some(entity);
    let mut depth = 0;
    std::iter::from_fn(move || {
        let at = current?;
        depth += 1;
        if depth > MAX_DEPTH {
            warn!(?entity, "parent chain exceeded {MAX_DEPTH}; cycle?");
            current = None;
            return None;
        }
        current = parents.get(at).ok().map(ChildOf::parent);
        Some(at)
    })
}

/// Whether `ancestor` is `entity` or stands above it.
#[must_use]
pub fn descends_from(entity: Entity, ancestor: Entity, parents: &Query<&ChildOf>) -> bool {
    ancestors(entity, parents).any(|at| at == ancestor)
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;

    use super::*;

    fn chain(world: &mut World, len: usize) -> Vec<Entity> {
        let mut made = Vec::new();
        for _ in 0..len {
            let entity = world.spawn_empty().id();
            if let Some(parent) = made.last() {
                world.entity_mut(entity).insert(ChildOf(*parent));
            }
            made.push(entity);
        }
        made
    }

    #[test]
    fn a_walk_reaches_the_root_nearest_first() {
        let mut world = World::new();
        let made = chain(&mut world, 3);
        let leaf = made[2];
        let expected = vec![made[2], made[1], made[0]];

        let walked = world
            .run_system_once(move |parents: Query<&ChildOf>| {
                ancestors(leaf, &parents).collect::<Vec<_>>()
            })
            .expect("ran");

        assert_eq!(walked, expected);
    }

    #[test]
    fn a_cycle_terminates() {
        let mut world = World::new();
        let made = chain(&mut world, 3);
        let (root, leaf) = (made[0], made[2]);
        world.entity_mut(root).insert(ChildOf(leaf));

        let walked = world
            .run_system_once(move |parents: Query<&ChildOf>| ancestors(leaf, &parents).count())
            .expect("ran");

        assert_eq!(walked, MAX_DEPTH);
    }

    #[test]
    fn an_entity_descends_from_itself_and_its_parents() {
        let mut world = World::new();
        let made = chain(&mut world, 3);
        let (root, mid, leaf) = (made[0], made[1], made[2]);

        let (from_root, from_leaf, from_self) = world
            .run_system_once(move |parents: Query<&ChildOf>| {
                (
                    descends_from(leaf, root, &parents),
                    descends_from(root, leaf, &parents),
                    descends_from(mid, mid, &parents),
                )
            })
            .expect("ran");

        assert!(from_root, "a leaf descends from its root");
        assert!(!from_leaf, "a root does not descend from its leaf");
        assert!(from_self);
    }
}
