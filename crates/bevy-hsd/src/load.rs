use std::{sync::Arc, time::Duration};

use async_channel::Receiver;
use bevy::{prelude::*, tasks::BoxedFuture};
use bevy_wds::{
    LocalActor, LocalBlobs,
    record::write::{SchemaDef, WriteRecord},
};
use blake3::Hash;
use loro::LoroDoc;
use loro_surgeon::Reconcile;
use unavi_util::{async_commands::AsyncCommands, async_task::spawn_async_task};
use wds::{Blobs, actor::Actor};
use wired_schemas::SCHEMA_HSD;

use crate::{
    HsdDoc, HsdRecordId,
    asset::{BlobAsset, HsdAsset},
};

const DEFAULT_TTL: Duration = Duration::from_hours(7 * 24);

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

pub(crate) fn instance_hsd(
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
                // Wait for all blobs to load.
                continue 'loading;
            };

            blob_assets.push(blob);
        }

        let actor = actor.0.clone();
        let blob_assets = blob_assets
            .into_iter()
            .map(|b| b.0.clone())
            .collect::<Vec<_>>();
        let asset_doc = asset.doc.clone();
        let local_blobs = local_blobs.0.clone();

        let (mut write, rx, cancel) = WriteRecord::new(None);
        write.ttl = Some(DEFAULT_TTL);
        write.schemas = vec![SchemaDef {
            container: "hsd".into(),
            schema: (&*SCHEMA_HSD).into(),
            f: Arc::new(move |doc| {
                let map = doc.get_map("hsd");
                asset_doc.reconcile(&map)?;
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
        });
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
                    .insert((HsdDoc(Arc::new(ctx.doc)), HsdRecordId(ctx.record_id)));
            })
            .send()
            .await?;
        Ok(())
    })
}
