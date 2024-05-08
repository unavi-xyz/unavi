use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use dwn::{
    actor::Actor,
    message::{
        descriptor::{protocols::ProtocolsFilter, records::RecordsFilter, Descriptor},
        Data,
    },
    store::{DataStore, MessageStore},
};
use tracing::{info, warn};
use wired_protocol::protocols::world_host::{
    world_host_definition, world_host_protocol_url, WORLD_HOST_PROTOCOL_VERSION,
};

pub async fn create_world_host(
    actor: &Actor<impl DataStore, impl MessageStore>,
    connect_url: &str,
) {
    // Register protocol.
    let definition = world_host_definition();

    let protocols = actor
        .query_protocols(ProtocolsFilter {
            protocol: definition.protocol.clone(),
            versions: vec![WORLD_HOST_PROTOCOL_VERSION],
        })
        .process()
        .await
        .unwrap();

    if protocols.entries.is_empty() {
        info!(
            "Initializing world host protocol v{}",
            WORLD_HOST_PROTOCOL_VERSION
        );

        actor
            .register_protocol(definition)
            .protocol_version(WORLD_HOST_PROTOCOL_VERSION)
            .process()
            .await
            .unwrap();
    }

    // Set connect url.
    let connect_records = actor
        .query_records(RecordsFilter {
            data_format: Some("text/plain".to_string()),
            protocol: Some(world_host_protocol_url()),
            protocol_version: Some(WORLD_HOST_PROTOCOL_VERSION),
            ..Default::default()
        })
        .process()
        .await
        .unwrap();

    if connect_records.entries.is_empty() {
        info!("Initializing connect URL: {}", connect_url);

        actor
            .create_record()
            .protocol(
                world_host_protocol_url(),
                WORLD_HOST_PROTOCOL_VERSION,
                "connect-url".to_string(),
            )
            .data_format("text/plain".to_string())
            .data(connect_url.into())
            .process()
            .await
            .unwrap();
    } else {
        // Ensure connect url is up to date.
        let found = connect_records.entries.iter().find(|message| {
            if let Descriptor::RecordsWrite(descriptor) = &message.descriptor {
                descriptor.protocol_path == Some("connect-url".to_string())
            } else {
                false
            }
        });

        if let Some(found) = found {
            let matches = match &found.data {
                Some(url) => match url {
                    Data::Base64(encoded) => {
                        let url = URL_SAFE_NO_PAD.decode(encoded).unwrap();
                        let url = String::from_utf8_lossy(&url);
                        url == connect_url
                    }
                    Data::Encrypted(_) => {
                        warn!("Connect URL is encrypted. Cannot verify.");
                        true
                    }
                },
                _ => false,
            };

            if !matches {
                info!(
                    "Connect URL out of data. Updating record to {}",
                    connect_url
                );

                actor
                    .update_record(found.record_id.clone(), found.entry_id().unwrap())
                    .data(connect_url.into())
                    .process()
                    .await
                    .unwrap();
            }
        }
    }
}
