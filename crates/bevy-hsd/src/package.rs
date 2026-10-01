//! Loading a compiled `.hsdz` package into its own namespace.

use std::collections::HashMap;

use bevy::{
    asset::{
        AssetLoader,
        LoadContext,
        io::Reader,
    },
    prelude::*,
    reflect::TypePath,
    tasks::ConditionalSendFuture,
};
use bevy_iroh::store::LocalStore;
use hsd::{
    format::package::{
        self,
        Package,
    },
    id::DocId,
    state::HsdState,
};
use iroh_docs::NamespaceId;
use unavi_util::{
    async_commands::AsyncCommands,
    async_task::spawn_async_task,
};
use wds::Store;

use crate::document::{
    Hsd,
    HsdDocId,
    HsdNamespace,
};

/// A compiled `.hsdz` package.
#[derive(Asset, TypePath)]
pub struct PackageAsset(pub Package);

#[derive(Default, TypePath)]
pub struct PackageLoader;

impl AssetLoader for PackageLoader {
    type Asset = PackageAsset;
    type Settings = ();
    type Error = anyhow::Error;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            Ok(PackageAsset(Package::decode(&bytes)?))
        })
    }

    fn extensions(&self) -> &[&str] {
        &[package::EXTENSION]
    }
}

#[derive(Component)]
pub struct ImportPackage(pub Handle<PackageAsset>);

/// Imports each loaded package into fresh namespaces.
pub(crate) fn import_packages(
    packages: Res<Assets<PackageAsset>>,
    importing: Query<(Entity, &ImportPackage)>,
    stores: Query<&LocalStore>,
    mut commands: Commands,
) {
    let Ok(store) = stores.single() else {
        return;
    };

    for (entity, import) in importing {
        let Some(asset) = packages.get(&import.0) else {
            continue;
        };

        let store = store.0.clone();
        let package = asset.0.clone();

        spawn_async_task(async move {
            if let Err(err) = import_package(store, package, entity).await {
                error!(?err, "failed to import hsd package");
            }
        });

        commands.entity(entity).remove::<ImportPackage>();
    }
}

// On failure, removes every namespace `try_import_package` minted.
// n0_future futures stay !Send on wasm and gain `Send` elsewhere.
#[cfg_attr(target_family = "wasm", expect(clippy::future_not_send))]
async fn import_package(store: Store, package: Package, entity: Entity) -> anyhow::Result<()> {
    let mut minted = Vec::new();
    let result = try_import_package(&store, package, entity, &mut minted).await;

    if result.is_err() {
        for ns in minted {
            if let Err(err) = store.remove(ns).await {
                error!(?err, %ns, "failed to remove a namespace minted by a failed import");
            }
        }
    }

    result
}

async fn try_import_package(
    store: &Store,
    package: Package,
    entity: Entity,
    minted_namespaces: &mut Vec<NamespaceId>,
) -> anyhow::Result<()> {
    // Every namespace is minted before any write, since a sub-document may
    // reference a sibling minted after it.
    let mut minted = HashMap::new();
    let mut docs = Vec::with_capacity(package.documents.len());
    for (placeholder, entries) in package.documents {
        let doc = store.create().await?;
        minted_namespaces.push(doc.id());
        minted.insert(placeholder, DocId(*doc.id().as_bytes()));
        docs.push((doc, entries));
    }

    // Served so peers can resolve references into it.
    for (doc, mut entries) in docs {
        Package::rewrite_refs(&mut entries, &minted)?;
        for (key, value) in entries {
            doc.set(key, value).await?;
        }
        doc.serve().await?;
    }

    let doc = store.create().await?;
    minted_namespaces.push(doc.id());
    let namespace = doc.id();

    let mut entries = package.entries;
    Package::rewrite_refs(&mut entries, &minted)?;
    for (key, value) in entries {
        doc.set(key, value).await?;
    }

    AsyncCommands::default()
        .push(move |world: &mut World| {
            if let Ok(mut entity) = world.get_entity_mut(entity) {
                entity.insert((
                    Hsd::new(HsdState::new()),
                    HsdDocId(DocId(*namespace.as_bytes())),
                    HsdNamespace(doc),
                ));
            }
        })
        .send()
        .await?;

    Ok(())
}
