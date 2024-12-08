use dwn::Actor;

pub async fn create_world_host(actor: &Actor, connect_url: &str) {
    // TODO

    // Register protocol.
    // let definition = world_host_definition();
    //
    // let protocols = actor
    //     .query_protocols(ProtocolsFilter {
    //         protocol: definition.protocol.clone(),
    //         versions: vec![WORLD_HOST_PROTOCOL_VERSION],
    //     })
    //     .process()
    //     .await
    //     .unwrap();
    //
    // if protocols.entries.is_empty() {
    //     info!(
    //         "Initializing world host protocol v{}",
    //         WORLD_HOST_PROTOCOL_VERSION
    //     );
    //
    //     actor
    //         .register_protocol(definition.clone())
    //         .protocol_version(WORLD_HOST_PROTOCOL_VERSION)
    //         .process()
    //         .await
    //         .unwrap();
    // }
    //
    // // Set connect url.
    // let connect_records = actor
    //     .query_records(RecordsFilter {
    //         data_format: Some("text/plain".to_string()),
    //         protocol: Some(world_host_protocol_url()),
    //         protocol_version: Some(WORLD_HOST_PROTOCOL_VERSION),
    //         ..Default::default()
    //     })
    //     .process()
    //     .await
    //     .unwrap();
    //
    // if connect_records.entries.is_empty() {
    //     info!("Initializing connect URL: {}", connect_url);
    //
    //     actor
    //         .create_record()
    //         .protocol(
    //             world_host_protocol_url(),
    //             WORLD_HOST_PROTOCOL_VERSION,
    //             "connect-url".to_string(),
    //         )
    //         .data(connect_url.into())
    //         .data_format("text/plain".to_string())
    //         .published(true)
    //         .process()
    //         .await
    //         .unwrap();
    // } else {
    //     // Ensure connect url is up to date.
    //     let found = connect_records.entries.iter().find(|message| {
    //         if let Descriptor::RecordsWrite(descriptor) = &message.descriptor {
    //             descriptor.protocol_path == Some("connect-url".to_string())
    //         } else {
    //             false
    //         }
    //     });
    //
    //     if let Some(found) = found {
    //         let data = match &found.data {
    //             Some(data) => Some(data.clone()),
    //             None => {
    //                 let read = actor
    //                     .read_record(found.record_id.clone())
    //                     .process()
    //                     .await
    //                     .unwrap();
    //                 read.record.data
    //             }
    //         };
    //
    //         let matches = match &data {
    //             Some(url) => match url {
    //                 Data::Base64(encoded) => {
    //                     let url = URL_SAFE_NO_PAD.decode(encoded).unwrap();
    //                     let url = String::from_utf8_lossy(&url);
    //                     info!("Current connect URL: {}", url);
    //                     url == connect_url
    //                 }
    //                 Data::Encrypted(_) => {
    //                     warn!("Connect URL is encrypted. Cannot verify.");
    //                     true
    //                 }
    //             },
    //             None => {
    //                 warn!("Connect URL record has no data.");
    //                 false
    //             }
    //         };
    //
    //         if !matches {
    //             info!("Updating connect URL to {}", connect_url);
    //
    //             actor
    //                 .update_record(found.record_id.clone(), found.entry_id().unwrap())
    //                 .data(connect_url.into())
    //                 .data_format("text/plain".to_string())
    //                 .published(true)
    //                 .process()
    //                 .await
    //                 .unwrap();
    //         }
    //     }
    // }
}
