use std::{sync::Arc, time::Duration};

use dwn::{
    actor::{Actor, MessageBuilder},
    message::descriptor::{protocols::ProtocolsFilter, records::RecordsFilter},
    store::SurrealStore,
    DWN,
};
use surrealdb::{engine::local::Mem, Surreal};
use tracing_test::traced_test;
use unavi_server::{Args, Command, Storage};
use wired_protocol::protocols::world_host::{world_host_protocol_url, WORLD_HOST_PROTOCOL_VERSION};

fn local_domain(port: u16) -> String {
    format!("localhost:{}", port)
}

#[tokio::test]
#[traced_test]
async fn test_world_host() {
    let port_social = port_scanner::request_open_port().unwrap();
    let domain_social = local_domain(port_social);

    let port_world = port_scanner::request_open_port().unwrap();
    let domain_world = local_domain(port_world);

    let args_social = Args {
        debug: true,
        command: Command::Social {
            domain: domain_social.clone(),
            path: String::new(),
            port: port_social,
            storage: Storage::Memory,
        },
    };

    let args_world = Args {
        debug: true,
        command: Command::World {
            domain: domain_world,
            dwn_url: format!("http://{}", domain_social),
            path: String::new(),
            port: port_world,
            storage: Storage::Memory,
        },
    };

    let social_task = tokio::spawn(unavi_server::start(args_social));
    let world_task = tokio::spawn(unavi_server::start(args_world));
    tokio::time::sleep(Duration::from_secs(5)).await;

    assert!(logs_contain("Sync successful."));

    // Can query protocols and records using world host DID.
    let world_host_did = format!("did:web:localhost%3A{}", port_world);

    let db = Surreal::new::<Mem>(()).await.unwrap();
    let store = SurrealStore::new(db).await.unwrap();
    let dwn = Arc::new(DWN::from(store));
    let actor = Actor::new_did_key(dwn.clone()).unwrap();

    let query_protocols = actor
        .query_protocols(ProtocolsFilter {
            protocol: world_host_protocol_url(),
            versions: vec![WORLD_HOST_PROTOCOL_VERSION],
        })
        .target(world_host_did.clone())
        .send(&world_host_did)
        .await
        .unwrap();
    assert_eq!(query_protocols.status.code, 200);
    assert!(!query_protocols.entries.is_empty());

    let query_records = actor
        .query_records(RecordsFilter::default())
        .target(world_host_did.clone())
        .send(&world_host_did)
        .await
        .unwrap();
    assert_eq!(query_records.status.code, 200);
    assert!(!query_records.entries.is_empty());

    social_task.abort();
    world_task.abort();
}
