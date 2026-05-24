use std::{sync::Arc, time::Duration};

use anyhow::Context;
use async_channel::Receiver;
use bevy::{
    asset::{AssetLoader, AsyncReadExt, LoadContext, io::Reader},
    platform::collections::HashMap,
    prelude::*,
    reflect::TypePath,
    tasks::{BoxedFuture, ConditionalSendFuture},
};
use bevy_wds::{
    LocalActor, LocalBlobs,
    record::write::{SchemaDef, WriteRecord},
};
use blake3::Hash;
use hsd::{
    attributes::collider::ColliderAttr,
    file::{HsdFile, HsdFilePrim},
};
use loro::LoroDoc;
use unavi_util::{async_commands::AsyncCommands, async_task::spawn_async_task};
use wds::{Blobs, actor::Actor};
use wired_schemas::SCHEMA_HSD;

use crate::{Hsd, HsdRecordId};

const DEFAULT_TTL: Duration = Duration::from_hours(7 * 24);

#[derive(Asset, TypePath)]
pub struct HsdAsset {
    pub file: Arc<HsdFile>,
    pub deps: HashMap<Hash, Handle<BlobAsset>>,
}

#[derive(Asset, Debug, Deref, DerefMut, TypePath)]
pub struct BlobAsset(pub Vec<u8>);

#[derive(Default, TypePath)]
pub struct HsdLoader;

impl AssetLoader for HsdLoader {
    type Asset = HsdAsset;
    type Settings = ();
    type Error = anyhow::Error;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut s = String::new();
            reader.read_to_string(&mut s).await?;
            let file = Arc::new(HsdFile::from_ron(&s)?);

            let mut deps = HashMap::new();
            collect_blob_deps(&file, &mut |hash| {
                let path = load_context
                    .path()
                    .path()
                    .parent()
                    .expect("asset parent dir")
                    .join(Hash::from_bytes(hash).to_string());
                deps.entry(Hash::from_bytes(hash))
                    .or_insert_with(|| load_context.load(path));
            });

            Ok(HsdAsset { file, deps })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["hsd"]
    }
}

#[derive(Default, TypePath)]
pub struct BlobLoader;

impl AssetLoader for BlobLoader {
    type Asset = BlobAsset;
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
            Ok(BlobAsset(bytes))
        })
    }

    fn extensions(&self) -> &[&str] {
        &[]
    }
}

fn collect_blob_deps(file: &HsdFile, push: &mut impl FnMut([u8; 32])) {
    for prim in &file.0 {
        walk_prim(prim, push);
    }
}

fn walk_prim(prim: &HsdFilePrim, push: &mut impl FnMut([u8; 32])) {
    let a = &prim.attributes;
    if let Some(asset) = &a.asset {
        push(asset.0.0);
    }
    if let Some(script) = &a.script {
        push(script.0.0);
    }
    if let Some(img) = &a.image {
        push(img.data.0);
    }
    if let Some(mesh) = &a.mesh {
        for b in mesh.attributes.values() {
            push(b.0);
        }
        if let Some(idx) = &mesh.indices {
            push(idx.0);
        }
    }
    if let Some(c) = &a.collider {
        match c {
            ColliderAttr::ConvexHull(b) => push(b.0),
            ColliderAttr::Trimesh { indices, vertices } => {
                push(indices.0);
                push(vertices.0);
            }
            _ => {}
        }
    }
    for child in &prim.children {
        walk_prim(child, push);
    }
}

pub struct OnLoadCtx {
    pub doc: LoroDoc,
    pub entity: Entity,
    pub record_id: Hash,
}

pub type OnLoadFn =
    Box<dyn FnOnce(OnLoadCtx) -> BoxedFuture<'static, anyhow::Result<()>> + Send + Sync>;

#[derive(Component)]
pub struct LoadHsd {
    pub handle: Handle<HsdAsset>,
    pub extra_schemas: Option<Vec<SchemaDef>>,
    pub on_load: Option<OnLoadFn>,
}

pub fn instance_hsd(
    hsds: Res<Assets<HsdAsset>>,
    blobs: Res<Assets<BlobAsset>>,
    loading: Query<(Entity, &mut LoadHsd)>,
    actors: Query<(&LocalActor, &LocalBlobs)>,
    mut commands: Commands,
) {
    'loading: for (entity, mut load) in loading {
        let Some(asset) = hsds.get(&load.handle) else {
            continue;
        };

        let Ok((actor, local_blobs)) = actors.single() else {
            continue;
        };

        let mut blob_assets = Vec::new();
        for blob_handle in asset.deps.values() {
            let Some(blob) = blobs.get(blob_handle) else {
                continue 'loading;
            };
            blob_assets.push(blob.0.clone());
        }

        let actor = actor.0.clone();
        let local_blobs = local_blobs.0.clone();
        let file = Arc::clone(&asset.file);

        let (mut write, rx, cancel) = WriteRecord::new(None);
        write.ttl = Some(DEFAULT_TTL);
        write.schemas = vec![SchemaDef {
            container: "hsd".into(),
            schema: (&*SCHEMA_HSD).into(),
            f: Arc::new(move |doc| {
                file.load_into_doc(doc)?;
                Ok(())
            }),
        }];
        write
            .schemas
            .extend(load.extra_schemas.take().unwrap_or_default());
        commands.trigger(write);

        let on_load = load.on_load.take();

        spawn_async_task(async move {
            if let Err(err) = recv_doc(actor, local_blobs, rx, blob_assets, entity, on_load).await {
                error!(?err, "Failed to receive HSD record");
                let _ = cancel.send(());
            }
        });

        commands.entity(entity).remove::<LoadHsd>();
    }
}

async fn recv_doc(
    actor: Actor,
    local_blobs: Blobs,
    rx: Receiver<Hash>,
    blob_assets: Vec<Vec<u8>>,
    entity: Entity,
    on_load: Option<OnLoadFn>,
) -> anyhow::Result<()> {
    for blob in blob_assets {
        local_blobs.add_slice(&blob).await?;
    }

    let record_id = rx.recv().await?;
    let doc = actor.read(record_id).send().await?;

    if let Some(on_load) = on_load {
        on_load(OnLoadCtx {
            doc,
            entity,
            record_id,
        })
        .await
        .context("on_load")?;
    }

    Ok(())
}

#[must_use]
pub fn on_load_spawn_doc(ctx: OnLoadCtx) -> BoxedFuture<'static, anyhow::Result<()>> {
    Box::pin(async move {
        AsyncCommands::default()
            .push(move |world: &mut World| {
                world
                    .entity_mut(ctx.entity)
                    .insert((Hsd(Arc::new(ctx.doc)), HsdRecordId(ctx.record_id)));
            })
            .send()
            .await?;
        Ok(())
    })
}
