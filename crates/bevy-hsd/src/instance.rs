use std::{sync::Arc, time::Duration};

use async_channel::Receiver;
use bevy::prelude::*;
use bevy_wds::{
    LocalActor, LocalBlobs,
    record::write::{SchemaDef, WriteRecord},
};
use blake3::Hash;
use loro_surgeon::Reconcile;
use unavi_util::{async_commands::AsyncCommands, async_task::spawn_async_task};
use wds::{Blobs, actor::Actor};
use wired_schemas::SCHEMA_HSD;

use crate::{
    HsdDoc, HsdRecordId,
    asset::{BlobAsset, HsdAsset},
};

const DEFAULT_TTL: Duration = Duration::from_hours(7 * 24);

#[derive(Component)]
pub struct InstanceHsd(pub Handle<HsdAsset>);

#[derive(Component)]
pub struct InstancingHsd;

pub(crate) fn instance_hsd(
    hsds: Res<Assets<HsdAsset>>,
    blobs: Res<Assets<BlobAsset>>,
    to_spawn: Query<(Entity, &InstanceHsd)>,
    actors: Query<(&LocalActor, &LocalBlobs)>,
    mut commands: Commands,
) {
    'hsd: for (entity, instance) in to_spawn {
        let Some(asset) = hsds.get(&instance.0) else {
            continue;
        };

        let Ok((actor, local_blobs)) = actors.single() else {
            continue;
        };

        let mut blob_assets = Vec::new();

        for blob_handle in asset.deps.values() {
            let Some(blob) = blobs.get(blob_handle) else {
                // Wait for all blobs to load.
                continue 'hsd;
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
        commands.trigger(write);

        spawn_async_task(async move {
            if let Err(err) = spawn_hsd_record(actor, local_blobs, rx, blob_assets, entity).await {
                error!(?err, "Failed to spawn HSD record");
                let _ = cancel.send(());
            }
        });

        commands.entity(entity).remove::<InstanceHsd>();
    }
}

async fn spawn_hsd_record(
    actor: Actor,
    local_blobs: Blobs,
    rx: Receiver<Hash>,
    blob_assets: Vec<Vec<u8>>,
    entity: Entity,
) -> anyhow::Result<()> {
    for blob in blob_assets {
        local_blobs.add_slice(&blob).await?;
    }

    let record_id = rx.recv().await?;
    let doc = actor.read(record_id).send().await?;

    AsyncCommands::default()
        .push(move |world: &mut World| {
            world
                .entity_mut(entity)
                .insert((HsdDoc(Arc::new(doc)), HsdRecordId(record_id)));
        })
        .send()
        .await?;

    Ok(())
}
