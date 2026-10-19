//! Portals into spaces: loading each destination, and placing linked portals
//! in the space their document belongs to.

use std::time::Duration;

use bevy::{
    ecs::system::SystemParam,
    platform::collections::{
        HashMap,
        HashSet,
    },
    prelude::*,
};
use bevy_hsd::{
    attributes::portal::PortalConfig,
    document::{
        DocIndex,
        HsdDocId,
    },
    hierarchy::descends_from,
    prim::PrimOf,
};
use unavi_policy::{
    Policy,
    permissions::HostApi,
};
use unavi_portal::portal::{
    PortalHome,
    PortalLink,
};

use crate::{
    authority::SpaceView,
    grid::ActiveSpace,
    index::Index,
    membership::{
        Space,
        SpaceId,
    },
    pinned::PinnedDoc,
};

/// Most spaces open at once through portals and not yet entered.
pub const MAX_PORTAL_SPACES: usize = 8;

/// How long a portal space may go untargeted before it is unloaded, so a
/// moment between one active space and the next does not reload it.
pub const UNTARGETED_GRACE: Duration = Duration::from_secs(5);

/// A space loaded because a portal leads to it, until the local agent enters
/// it. Counted against [`MAX_PORTAL_SPACES`], whoever wrote the portal.
#[derive(Component)]
pub struct ByPortal;

/// The portals in the active space, which are the only ones that load or hold
/// open a destination.
#[derive(SystemParam)]
pub struct ActivePortals<'w, 's> {
    active:  Res<'w, ActiveSpace>,
    portals: Query<'w, 's, (&'static PortalConfig, &'static PrimOf)>,
    parents: Query<'w, 's, &'static ChildOf>,
}

impl ActivePortals<'_, '_> {
    /// Each destination an active-space portal leads to, with the document the
    /// portal is in. Empty without an active space.
    fn destinations(&self) -> impl Iterator<Item = (SpaceId, Entity)> + '_ {
        let active = self.active.0;
        self.portals.iter().filter_map(move |(portal, prim_of)| {
            let dest = portal.0.destination.as_ref()?;
            descends_from(prim_of.0, active?, &self.parents)
                .then_some((SpaceId(dest.space), prim_of.0))
        })
    }

    fn is_limbo(&self) -> bool {
        self.active.0.is_none()
    }
}

/// Loads the destination space of every active-space portal whose space is
/// not loaded, within [`MAX_PORTAL_SPACES`].
///
/// Runs every frame, so a space that unloads while a portal still leads to it
/// is loaded again. A portal in a document a peer pinned only loads its
/// destination when that author may open portals at all. Portals in a portal
/// space load nothing, so a chain of portals cannot pull in the network.
pub fn reconcile_portal_spaces(
    active: ActivePortals,
    docs: Query<(&HsdDocId, Has<PinnedDoc>)>,
    doc_index: Res<DocIndex>,
    spaces: Res<Index<Space>>,
    by_portal: Query<(), With<ByPortal>>,
    policy: Res<Policy>,
    view: Option<Res<SpaceView>>,
    mut commands: Commands,
) {
    // A `commands.spawn` isn't visible to `spaces` until the command queue
    // flushes, so a second portal to the same destination wouldn't see the
    // first's spawn without this. It lasts one run: the next sees the space.
    let mut pending = HashSet::<SpaceId>::default();
    let mut loaded = by_portal.iter().count();

    for (id, doc_entity) in active.destinations() {
        if spaces.get(id).is_some() || pending.contains(&id) {
            continue;
        }
        if loaded >= MAX_PORTAL_SPACES {
            continue;
        }
        // A portal a peer pinned is the only one whose author gates loading.
        let source = docs.get(doc_entity).ok().map(|(doc, pinned)| {
            let root = policy.root(doc.0);
            let root_pinned = doc_index
                .get(root)
                .and_then(|e| docs.get(e).ok())
                .is_some_and(|(_, pinned)| pinned);
            (root, pinned || root_pinned)
        });
        if let Some((root, true)) = source {
            let allowed = view
                .as_deref()
                .is_some_and(|view| view.permissions(root).require(HostApi::Portal).is_ok());
            if !allowed {
                continue;
            }
        }

        pending.insert(id);
        loaded += 1;
        commands.spawn((Space(id), ByPortal));
    }
}

/// Unloads a portal space no active-space portal has led to for
/// [`UNTARGETED_GRACE`].
///
/// The active space is never unloaded, and limbo unloads nothing: travel keeps
/// its portal spaces there, and the space limbo is waiting on may be one.
pub fn evict_portal_spaces(
    active: ActivePortals,
    time: Res<Time>,
    by_portal: Query<(Entity, &Space), With<ByPortal>>,
    mut untargeted: Local<HashMap<SpaceId, Duration>>,
    mut commands: Commands,
) {
    if active.is_limbo() {
        untargeted.clear();
        return;
    }
    let now = time.elapsed();
    let targeted = active
        .destinations()
        .map(|(id, _)| id)
        .collect::<HashSet<_>>();

    untargeted.retain(|id, _| !targeted.contains(id));
    for (entity, space) in &by_portal {
        if active.active.0 == Some(entity) || targeted.contains(&space.0) {
            continue;
        }
        let since = *untargeted.entry(space.0).or_insert(now);
        if now.saturating_sub(since) >= UNTARGETED_GRACE {
            untargeted.remove(&space.0);
            commands.entity(entity).despawn();
        }
    }
    untargeted.retain(|id, _| by_portal.iter().any(|(_, space)| space.0 == *id));
}

/// A space the local agent walked into through a portal is entered like any
/// other.
pub fn enter_portal_space(
    active: Res<ActiveSpace>,
    by_portal: Query<(), With<ByPortal>>,
    mut commands: Commands,
) {
    if let Some(entity) = active.0
        && by_portal.contains(entity)
    {
        commands.entity(entity).remove::<ByPortal>();
    }
}

/// Keeps each linked portal's [`PortalHome`] on the space its document is
/// in, which is unknown until the document registers.
pub fn sync_portal_home(
    portals: Query<(Entity, &PrimOf, Option<&PortalHome>), With<PortalLink>>,
    docs: Query<&HsdDocId>,
    view: Option<Res<SpaceView>>,
    mut commands: Commands,
) {
    let Some(view) = view else {
        return;
    };
    for (portal, child, current) in &portals {
        let home = docs
            .get(child.0)
            .ok()
            .and_then(|doc| view.space_of(doc.0))
            .map(SpaceId::doc);
        match (home, current) {
            (Some(home), Some(cur)) if cur.0 == home => {}
            (Some(home), _) => {
                commands.entity(portal).insert(PortalHome(home));
            }
            (None, Some(_)) => {
                commands.entity(portal).remove::<PortalHome>();
            }
            (None, None) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use hsd::attributes::portal::{
        PortalAttr,
        PortalDestination,
    };

    use super::*;

    /// An app standing in a plain space entity.
    fn app() -> App {
        let mut app = App::new();
        app.init_resource::<Policy>()
            .init_resource::<DocIndex>()
            .init_resource::<Index<Space>>()
            .init_resource::<Time>()
            .add_systems(Update, (reconcile_portal_spaces, evict_portal_spaces));
        let active = app.world_mut().spawn_empty().id();
        app.insert_resource(ActiveSpace(Some(active)));
        app
    }

    fn active(app: &App) -> Entity {
        app.world().resource::<ActiveSpace>().0.expect("active")
    }

    fn advance(app: &mut App, secs: u64) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(secs));
    }

    fn by_portal_space(app: &mut App, id: SpaceId) -> Entity {
        app.world_mut().spawn((Space(id), ByPortal)).id()
    }

    /// A portal to `space`, in a document under `space_entity`.
    fn portal_in(app: &mut App, space_entity: Entity, space: SpaceId) -> Entity {
        let doc = app.world_mut().spawn(ChildOf(space_entity)).id();
        app.world_mut()
            .spawn((
                PortalConfig(PortalAttr {
                    destination: Some(PortalDestination {
                        space: space.0,
                        link:  None,
                    }),
                    size_x:      1.0,
                    size_y:      1.0,
                }),
                PrimOf(doc),
            ))
            .id()
    }

    fn portal_to(app: &mut App, space: SpaceId) -> Entity {
        let active = active(app);
        portal_in(app, active, space)
    }

    fn spaces_of(app: &mut App, id: SpaceId) -> usize {
        app.world_mut()
            .query::<&Space>()
            .iter(app.world())
            .filter(|space| space.0 == id)
            .count()
    }

    #[test]
    fn a_despawned_destination_loads_again() {
        let mut app = app();
        let id = SpaceId([1; 32]);
        portal_to(&mut app, id);

        app.update();
        assert_eq!(spaces_of(&mut app, id), 1);

        let entity = app
            .world()
            .resource::<Index<Space>>()
            .get(id)
            .expect("indexed");
        app.world_mut().entity_mut(entity).despawn();
        app.update();
        assert_eq!(spaces_of(&mut app, id), 1);
        assert!(app.world().resource::<Index<Space>>().get(id).is_some());
    }

    #[test]
    fn portals_to_one_destination_load_it_once() {
        let mut app = app();
        let id = SpaceId([2; 32]);
        portal_to(&mut app, id);
        portal_to(&mut app, id);

        app.update();
        assert_eq!(spaces_of(&mut app, id), 1);
        app.update();
        assert_eq!(spaces_of(&mut app, id), 1);
    }

    #[test]
    fn destinations_load_within_the_cap() {
        let mut app = app();
        for seed in 0..=MAX_PORTAL_SPACES as u8 {
            portal_to(&mut app, SpaceId([seed + 10; 32]));
        }

        app.update();
        let count = app
            .world_mut()
            .query_filtered::<(), With<ByPortal>>()
            .iter(app.world())
            .count();
        assert_eq!(count, MAX_PORTAL_SPACES);
    }

    #[test]
    fn a_portal_in_a_portal_space_loads_nothing() {
        let mut app = app();
        let near = SpaceId([20; 32]);
        let far = SpaceId([21; 32]);
        portal_to(&mut app, near);
        app.update();

        let near_entity = app
            .world()
            .resource::<Index<Space>>()
            .get(near)
            .expect("near loaded");
        portal_in(&mut app, near_entity, far);
        app.update();
        app.update();

        assert_eq!(spaces_of(&mut app, far), 0);
    }

    #[test]
    fn limbo_loads_nothing() {
        let mut app = app();
        let id = SpaceId([22; 32]);
        portal_to(&mut app, id);
        app.insert_resource(ActiveSpace(None));

        app.update();

        assert_eq!(spaces_of(&mut app, id), 0);
    }

    #[test]
    fn an_untargeted_portal_space_unloads_after_the_grace() {
        let mut app = app();
        let id = SpaceId([23; 32]);
        by_portal_space(&mut app, id);

        app.update();
        advance(&mut app, UNTARGETED_GRACE.as_secs() - 1);
        app.update();
        assert_eq!(spaces_of(&mut app, id), 1, "kept within the grace");

        advance(&mut app, 1);
        app.update();
        assert_eq!(spaces_of(&mut app, id), 0, "unloaded after the grace");
    }

    #[test]
    fn a_retargeted_portal_space_restarts_its_grace() {
        let mut app = app();
        let id = SpaceId([24; 32]);
        by_portal_space(&mut app, id);

        app.update();
        advance(&mut app, 4);
        let portal = portal_to(&mut app, id);
        app.update();
        app.world_mut().entity_mut(portal).despawn();
        app.update();
        advance(&mut app, 4);
        app.update();
        assert_eq!(spaces_of(&mut app, id), 1, "grace restarted on retarget");

        advance(&mut app, 1);
        app.update();
        assert_eq!(spaces_of(&mut app, id), 0);
    }

    #[test]
    fn limbo_keeps_untargeted_portal_spaces() {
        let mut app = app();
        let id = SpaceId([25; 32]);
        by_portal_space(&mut app, id);
        app.insert_resource(ActiveSpace(None));

        app.update();
        advance(&mut app, 60);
        app.update();

        assert_eq!(spaces_of(&mut app, id), 1);
    }

    #[test]
    fn the_active_space_is_never_evicted() {
        let mut app = app();
        let id = SpaceId([26; 32]);
        let entity = by_portal_space(&mut app, id);
        app.insert_resource(ActiveSpace(Some(entity)));

        app.update();
        advance(&mut app, 60);
        app.update();

        assert_eq!(spaces_of(&mut app, id), 1);
    }
}
