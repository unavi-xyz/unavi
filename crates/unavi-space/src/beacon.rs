use std::{
    sync::Arc,
    time::Duration,
};

use bevy::prelude::*;
use bevy_iroh::endpoint::IrohEndpoint;
use bevy_wds::{
    LocalActor,
    record::write::{
        SchemaDef,
        WriteRecord,
    },
};
use time::OffsetDateTime;
use unavi_util::async_task::spawn_async_task;
use wired_records::{
    beacon::BeaconRecord,
    byte_array::ByteArray,
    did::HydratedDid,
};
use wired_schemas::SCHEMA_BEACON;

use crate::Space;

const BEACON_TTL: Duration = Duration::from_mins(2);

pub fn publish_beacons(
    time: Res<Time>,
    spaces: Query<&Space>,
    actor: Query<&LocalActor>,
    endpoint: Query<&IrohEndpoint>,
    mut last: Local<Duration>,
    mut commands: Commands,
) {
    if spaces.is_empty() {
        return;
    }

    let now = time.elapsed();
    if !last.is_zero() && now.saturating_sub(*last) < BEACON_TTL {
        return;
    }
    *last = now;

    let Ok(actor) = actor.single() else {
        return;
    };
    let Ok(endpoint) = endpoint.single() else {
        return;
    };

    let did = actor.0.identity().did().clone();
    let endpoint_id = endpoint.0.id();

    for space in spaces {
        let did = did.clone();
        let space = space.0;

        let (mut event, rx, cancel) = WriteRecord::new(None);
        event.ttl = Some(BEACON_TTL);
        event.public = true;
        event.schemas = vec![SchemaDef {
            container: "beacon".into(),
            schema:    (&*SCHEMA_BEACON).into(),
            f:         Arc::new(move |doc| {
                let beacon = BeaconRecord {
                    did:      HydratedDid(did.clone()),
                    endpoint: ByteArray(loro_surgeon::bytes::ByteArray(*endpoint_id.as_bytes())),
                    expires:  (OffsetDateTime::now_utc() + BEACON_TTL).unix_timestamp(),
                    space:    space.into(),
                };
                beacon.save(doc)?;
                Ok(())
            }),
        }];

        spawn_async_task(async move {
            if let Ok(id) = rx.recv().await {
                info!(%id, "Published beacon");
            }
            drop(cancel);
        });

        commands.trigger(event);
    }
}
