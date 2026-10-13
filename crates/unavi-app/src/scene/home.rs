use std::str::FromStr;

use bevy::prelude::*;
use bevy_async::task;
use bevy_hsd::{
    document::HsdNamespace,
    package::ImportPackage,
};
use bevy_iroh::store::DataStore;
use iroh_docs::NamespaceId;
use unavi_space::{
    identity::RootDocument,
    membership::Space,
};
use unavi_store::Store;

/// A namespace to enter instead of the local home, from `--join`.
#[derive(Resource, Default)]
pub struct JoinSpace(pub Option<String>);

/// Marks the entity `join_home` spawned, until [`enter_home`] claims its
/// namespace.
#[derive(Component)]
pub struct JoiningHome;

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
    commands.spawn((ImportPackage(handle), JoiningHome));
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
