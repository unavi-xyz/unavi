use avian3d::{
    collider_tree::{
        ColliderTreeProxy,
        ColliderTreeProxyFlags,
        ColliderTreeProxyKey,
        ColliderTreeType,
        ColliderTrees,
        MovedProxies,
        ProxyId,
    },
    collision::collider::EnlargedAabb,
    prelude::*,
};
use bevy::{
    ecs::entity::EntityHashSet,
    prelude::*,
};
use obvhs::aabb::Aabb;

/// Restores the collider tree invariant a scene can otherwise break: every
/// live collider's [`ColliderTreeProxyKey`] names a proxy that tree holds and
/// that proxy belongs to that collider.
///
/// Avian indexes its proxy arrays unchecked from these keys, so a stale key
/// panics the physics step or, when the slot has been reused, aliases another
/// collider's proxy until the next tree move evicts it. Reachable from a scene:
/// clearing a prim's `rigid_body` attribute while it keeps its collider drops
/// `ColliderOf`, and avian's handler moves the proxy to the standalone tree
/// without writing the new key back.
pub struct ColliderTreeIntegrityPlugin;

impl Plugin for ColliderTreeIntegrityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PhysicsSchedule,
            repair_collider_tree_keys
                // The step's first set, so the repair lands ahead of every
                // reader of the trees.
                .in_set(PhysicsStepSystems::First)
                // Matching avian's own tree systems, which opt out so that
                // several collision backends can coexist.
                .ambiguous_with_all(),
        );
    }
}

type TrackedColliders<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut ColliderTreeProxyKey,
        &'static EnlargedAabb,
        Option<&'static ColliderOf>,
        Option<&'static CollisionLayers>,
        Has<Sensor>,
        Has<CollisionEventsEnabled>,
        Option<&'static ActiveCollisionHooks>,
    ),
    (With<Collider>, Without<ColliderDisabled>),
>;

fn repair_collider_tree_keys(
    mut colliders: TrackedColliders,
    bodies: Query<(&RigidBody, Has<RigidBodyDisabled>)>,
    mut trees: ResMut<ColliderTrees>,
    mut moved: ResMut<MovedProxies>,
    mut broken: Local<EntityHashSet>,
    mut doomed: Local<Vec<(ColliderTreeType, ProxyId)>>,
) {
    broken.clear();
    broken.extend(
        colliders
            .iter()
            .filter(|(entity, key, ..)| !holds_own_proxy(&trees, **key, *entity))
            .map(|(entity, ..)| entity),
    );

    if broken.is_empty() {
        return;
    }

    warn!(
        count = broken.len(),
        "collider tree keys went stale; repairing before the physics step reads them",
    );

    // A stale key can outlive more than one tree move, so a collider may own
    // several proxies by now. Drop every one and mint a single replacement,
    // rather than adopt one and leak the rest as phantom colliders.
    doomed.clear();
    for tree_type in ColliderTreeType::ALL {
        let tree = trees.tree_for_type(tree_type);
        doomed.extend(
            tree.proxies
                .iter()
                .filter(|(_, proxy)| broken.contains(&proxy.collider))
                .map(|(index, _)| (tree_type, ProxyId::new(index as u32))),
        );
    }

    for (tree_type, proxy_id) in doomed.drain(..) {
        trees.tree_for_type_mut(tree_type).remove_proxy(proxy_id);
        moved.remove(&ColliderTreeProxyKey::new(proxy_id, tree_type));
    }

    for &entity in &*broken {
        let Ok((_, mut key, enlarged_aabb, collider_of, layers, is_sensor, has_events, hooks)) =
            colliders.get_mut(entity)
        else {
            continue;
        };

        let body = collider_of.map(|of| of.body);
        let (rigid_body, is_body_disabled) = match body.map(|body| bodies.get(body)) {
            Some(Ok((rigid_body, disabled))) => (Some(*rigid_body), disabled),
            _ => (None, false),
        };

        let tree_type = ColliderTreeType::from_body(rigid_body);
        let proxy = ColliderTreeProxy {
            collider: entity,
            body,
            layers: layers.copied().unwrap_or_default(),
            flags: ColliderTreeProxyFlags::new(
                is_sensor,
                is_body_disabled,
                has_events,
                hooks.copied().unwrap_or_default(),
            ),
        };

        let proxy_id = trees
            .tree_for_type_mut(tree_type)
            .add_proxy(Aabb::from(enlarged_aabb.get()), proxy);

        *key = ColliderTreeProxyKey::new(proxy_id, tree_type);
        moved.insert(*key);
    }
}

/// A placeholder key names no proxy by design, and is how avian marks a
/// collider it has deliberately left out of every tree.
fn holds_own_proxy(trees: &ColliderTrees, key: ColliderTreeProxyKey, entity: Entity) -> bool {
    if key == ColliderTreeProxyKey::PLACEHOLDER {
        return true;
    }
    trees
        .tree_for_type(key.tree_type())
        .get_proxy(key.id())
        .is_some_and(|proxy| proxy.collider == entity)
}
