use std::sync::Arc;

use anyhow::{bail, Result};
use capnp::{capability::Promise, Error};
use capnp_rpc::pry;
use dwn::{
    actor::{Actor, MessageBuilder},
    message::descriptor::Descriptor,
    store::{DataStore, MessageStore},
};
use tracing::{debug, error, info};
use wired_social::protocols::world_host::world_host_protocol_url;
use wired_world::world_server_capnp::world_server::{
    JoinInstanceParams, JoinInstanceResults, ListPlayersParams, ListPlayersResults, Server,
};

use crate::{
    commands::{SessionCommand, SessionMessage},
    global_context::GlobalContext,
};

pub struct WorldServer<D: DataStore, M: MessageStore> {
    pub actor: Arc<Actor<D, M>>,
    pub player_id: usize,
    pub context: Arc<GlobalContext>,
}

impl<D: DataStore + 'static, M: MessageStore + 'static> Server for WorldServer<D, M> {
    fn join_instance(
        &mut self,
        params: JoinInstanceParams,
        mut results: JoinInstanceResults,
    ) -> Promise<(), Error> {
        let params = pry!(params.get());
        let instance = pry!(params.get_instance());
        let record_id = pry!(pry!(instance.get_record_id()).to_string());

        debug!("Request to join instance: {}", record_id);

        let actor = self.actor.clone();
        let player_id = self.player_id;
        let world_host_did = self.context.world_host_did.clone();
        let context = self.context.clone();

        Promise::from_future(async move {
            let response = results.get().init_response();

            match verify_instance(actor, world_host_did, record_id.clone()).await {
                Ok(_) => {
                    debug!("Instance {} verified.", record_id);

                    context
                        .sender
                        .send(SessionMessage {
                            command: SessionCommand::JoinInstance { id: record_id },
                            player_id,
                        })
                        .map_err(|e| {
                            error!("Send failed: {}", e);
                            capnp::Error::from_kind(capnp::ErrorKind::Failed)
                        })?;

                    response.init_success().set_ok(true);
                }
                Err(e) => {
                    let e = e.to_string();
                    debug!("Instance error {}", e);
                    response.init_error(e.len() as u32).push_str(&e);
                }
            };

            Ok(())
        })
    }

    fn list_players(&mut self, _: ListPlayersParams, _: ListPlayersResults) -> Promise<(), Error> {
        info!("list_players");
        Promise::ok(())
    }
}

async fn verify_instance(
    actor: Arc<Actor<impl DataStore, impl MessageStore>>,
    world_host_did: String,
    record_id: String,
) -> Result<()> {
    let read = actor
        .read_record(record_id)
        .target(world_host_did)
        .process()
        .await?;
    debug!("Found record {}", read.record.record_id);

    let descriptor = match &read.record.descriptor {
        Descriptor::RecordsWrite(d) => d,
        _ => bail!("Invalid descriptor type"),
    };

    if descriptor.protocol != Some(world_host_protocol_url()) {
        debug!("Invalid instance protocol: {:?}", descriptor.protocol);
        bail!("Invalid descriptor")
    }

    if descriptor.protocol_path != Some("instance".to_string()) {
        debug!(
            "Invalid instance protocol path: {:?}",
            descriptor.protocol_path
        );
        bail!("Invalid descriptor")
    }

    Ok(())
}
