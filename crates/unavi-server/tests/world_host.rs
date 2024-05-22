use std::{sync::Arc, time::Duration};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use capnp_rpc::{rpc_twoparty_capnp::Side, twoparty::VatNetwork, RpcSystem};
use dwn::{
    actor::{Actor, MessageBuilder},
    message::{descriptor::records::RecordsFilter, Data},
    store::SurrealStore,
    DWN,
};
use surrealdb::{engine::local::Mem, Surreal};
use tokio::task::LocalSet;
use tracing_test::traced_test;
use wired_social::{
    protocols::world_host::{world_host_protocol_url, WORLD_HOST_PROTOCOL_VERSION},
    schemas::{common::RecordLink, instance::Instance},
};
use wired_world::world_server_capnp::{result::Which, world_server::Client};
use wtransport::ClientConfig;
use xwt_futures_io::{read::ReadCompat, write::WriteCompat};

mod utils;

#[tokio::test]
#[traced_test]
async fn test_world_host() {
    let utils::TestServer {
        domain_social,
        domain_world,
        port_world,
        task_social,
        task_world,
        ..
    } = utils::setup_test_server().await;

    // DID document is available.
    let response = reqwest::get(format!("http://{}/.well-known/did.json", domain_world))
        .await
        .unwrap();
    let json: serde_json::Value = response.json().await.unwrap();
    let root = json.as_object().unwrap();
    let service = root.get("service").unwrap().as_array().unwrap();
    let service_endpoint = service[0]
        .as_object()
        .unwrap()
        .get("serviceEndpoint")
        .unwrap()
        .as_str()
        .unwrap();
    assert_eq!(service_endpoint, format!("http://{}", domain_social));

    // Query connect URL using world host DID.
    let world_host_did = format!("did:web:localhost%3A{}", port_world);

    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store = SurrealStore::new(db).await.unwrap();
    let dwn = Arc::new(DWN::from(store));
    let actor = Actor::new_did_key(dwn.clone()).unwrap();

    let query_records = actor
        .query_records(RecordsFilter {
            protocol: Some(world_host_protocol_url()),
            protocol_version: Some(WORLD_HOST_PROTOCOL_VERSION),
            ..Default::default()
        })
        .target(world_host_did.clone())
        .send(&world_host_did)
        .await
        .unwrap();
    assert_eq!(query_records.status.code, 200);
    assert_eq!(query_records.entries.len(), 1);

    let message = &query_records.entries[0];

    let read = actor
        .read_record(message.record_id.clone())
        .target(world_host_did.clone())
        .send(&world_host_did)
        .await
        .unwrap();

    let connect_url = match &read.record.data {
        Some(Data::Base64(s)) => String::from_utf8(URL_SAFE_NO_PAD.decode(s).unwrap()).unwrap(),
        Some(_) => unreachable!(),
        None => panic!("No data"),
    };

    // Create an instance.
    let world_record_id = actor
        .create_record()
        .published(true)
        .send(&world_host_did)
        .await
        .unwrap()
        .record_id;

    let instance_record_id = actor
        .create_record()
        .published(true)
        .protocol(
            world_host_protocol_url(),
            WORLD_HOST_PROTOCOL_VERSION,
            "instance".to_string(),
        )
        .data(
            serde_json::to_vec(&Instance {
                world: RecordLink {
                    did: actor.did.clone(),
                    record_id: world_record_id,
                },
            })
            .unwrap(),
        )
        .data_format("application/json".to_string())
        .target(world_host_did.clone())
        .send(&world_host_did)
        .await
        .unwrap()
        .record_id;

    // Open WebTransport connection and join the instance.
    LocalSet::default()
        .run_until(async move {
            unavi_networking::handler::handle_instance_session(&connect_url, instance_record_id)
                .await
                .unwrap();
        })
        .await;

    if task_social.is_finished() {
        panic!("Server finished")
    }

    if task_world.is_finished() {
        panic!("Server finished")
    }

    task_social.abort();
    task_world.abort();
}
