use std::str::FromStr;

use bevy::prelude::*;
use bevy_async::{
    AsyncWorld,
    task,
};
use bevy_hsd::{
    document::{
        Hsd,
        HsdDocId,
        HsdNamespace,
        Minted,
    },
    package::{
        ImportPackage,
        PackageAsset,
    },
};
use bevy_iroh::store::DataStore;
use hsd::{
    id::DocId,
    state::HsdState,
};
use iroh_docs::NamespaceId;
use unavi_space::{
    identity::RootDocument,
    membership::{
        Space,
        SpaceId,
    },
};
use unavi_store::{
    Document,
    Store,
};

/// A namespace to enter instead of the local home, from `--join`.
#[derive(Resource, Default)]
pub struct JoinSpace(pub Option<String>);

/// Marks the entity `join_home` spawned, until [`enter_home`] claims its
/// namespace.
#[derive(Component)]
pub struct JoiningHome;

/// The space this node minted as its home. Travel drops and re-reads spaces as
/// bare entities, which lose the [`Minted`] claim that makes this node the
/// author of its own home; [`remint_home`] restores it.
#[derive(Resource)]
pub struct HomeSpace(pub SpaceId);

/// Marks a re-read home as minted by this node again once its document id is
/// known, which [`Minted`] needs in order to record the authorship.
///
/// Run by `drive_travel`, the system that re-reads spaces, every tick.
pub fn remint_home(
    home: Option<&HomeSpace>,
    unminted: &Query<(Entity, &Space), (With<HsdDocId>, Without<Minted>)>,
    commands: &mut Commands,
) {
    let Some(home) = home else {
        return;
    };
    for (entity, space) in unminted {
        if space.0 == home.0 {
            commands.entity(entity).insert(Minted);
        }
    }
}

pub fn join_startup_space(
    join: Res<JoinSpace>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let Some(raw) = join.0.as_deref() else {
        join_home(&asset_server, &mut commands);
        return;
    };

    match NamespaceId::from_str(raw) {
        Ok(ns) => {
            info!(%ns, "Joining space");
            commands.spawn(Space(ns.into()));
        }
        Err(err) => {
            error!(?err, raw, "Invalid --join namespace, falling back to home");
            join_home(&asset_server, &mut commands);
        }
    }
}

pub fn join_home(asset_server: &AssetServer, commands: &mut Commands) {
    let handle = asset_server.load("hsd/unavi_default_home.hsdz");
    commands.spawn(ResolveHome(handle));
}

/// A bundled home not yet decided: load the recorded one if it is still held,
/// otherwise import the default into fresh namespaces.
#[derive(Component)]
pub struct ResolveHome(pub Handle<PackageAsset>);

/// Resolves [`ResolveHome`] once the store and root document exist.
///
/// Startup runs before the identity task has installed either, so this waits
/// for them rather than guessing. A recorded home whose document is no longer
/// held falls back to a fresh import.
pub fn resolve_home(
    pending: Query<(Entity, &ResolveHome)>,
    store: Option<Res<DataStore>>,
    root: Option<Res<RootDocument>>,
    async_world: Res<AsyncWorld>,
    mut commands: Commands,
) {
    let (Some(store), Some(root)) = (store, root) else {
        return;
    };
    for (entity, resolve) in &pending {
        commands.entity(entity).despawn();
        let store = store.0.clone();
        let root = root.0;
        let handle = resolve.0.clone();
        let world = async_world.clone();
        task::spawn(async move {
            let loaded = load_recorded_home(&store, root).await;
            let sent = world
                .commands()
                .push(move |world: &mut World| match loaded {
                    Some((ns, doc)) => {
                        info!(%ns, "Loading saved home");
                        world.insert_resource(HomeSpace(ns.into()));
                        world.spawn((
                            Space(ns.into()),
                            Hsd::new(HsdState::new()),
                            HsdDocId(DocId(*ns.as_bytes())),
                            HsdNamespace(doc),
                            Minted,
                        ));
                    }
                    None => {
                        world.spawn((ImportPackage(handle), JoiningHome));
                    }
                })
                .send()
                .await;
            if let Err(err) = sent {
                warn!(?err, "the world is gone; dropping home resolution");
            }
        });
    }
}

/// The recorded home namespace and its held document, when both exist.
async fn load_recorded_home(store: &Store, root: NamespaceId) -> Option<(NamespaceId, Document)> {
    let doc = store.held(root).await.ok()??;
    let bytes = doc.get(HOME_KEY).await.ok()??;
    let (_version, rest) = postcard::take_from_bytes::<u32>(&bytes).ok()?;
    let bytes: &[u8; 32] = rest.get(..32)?.try_into().ok()?;
    let ns = NamespaceId::from(bytes);
    let held = store.held(ns).await.ok()??;
    Some((ns, held))
}

/// Enters a `join_home` entity as [`Space`] once its namespace resolves, and
/// records it as home.
pub fn enter_home(
    trigger: On<Add, HsdNamespace>,
    joining: Query<&HsdNamespace, With<JoiningHome>>,
    root: Option<Res<RootDocument>>,
    store: Option<Res<DataStore>>,
    mut commands: Commands,
) {
    let Ok(namespace) = joining.get(trigger.entity) else {
        return;
    };
    let ns = namespace.0.id();
    info!(%ns, "Joining home");

    commands
        .entity(trigger.entity)
        .insert(Space(ns.into()))
        .remove::<JoiningHome>();
    commands.insert_resource(HomeSpace(ns.into()));

    if let (Some(root), Some(store)) = (root, store) {
        task::spawn(record_home(store.0.clone(), root.0, ns));
    }
}

/// Key under a DID's root doc naming the space it comes home to.
const HOME_KEY: &str = "home";

/// Version prefix on the entry, so a reader cannot misread its bytes as a
/// namespace.
const HOME_VERSION: u32 = 0;

/// Writes down which space is home, so a shell can offer to travel back to it;
/// without this entry a script has no way to learn it.
async fn record_home(store: Store, root: NamespaceId, ns: NamespaceId) {
    let mut value = match postcard::to_stdvec(&HOME_VERSION) {
        Ok(value) => value,
        Err(err) => {
            warn!(?err, "could not encode the home space ref");
            return;
        }
    };
    value.extend_from_slice(ns.as_bytes());

    let recorded = async {
        let doc = store
            .held(root)
            .await?
            .ok_or(unavi_store::Error::NotHeld(root))?;
        doc.set(HOME_KEY, value).await
    };
    match recorded.await {
        Ok(_) => info!(%ns, "Recorded home space"),
        Err(err) => warn!(?err, "could not record the home space"),
    }
}
