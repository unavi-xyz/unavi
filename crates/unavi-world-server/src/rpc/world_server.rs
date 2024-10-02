use std::sync::Arc;

use anyhow::{bail, Result};
use capnp::capability::Promise;
use capnp_rpc::pry;
use dwn::{
    actor::{Actor, MessageBuilder},
    message::descriptor::Descriptor,
};
use tracing::{debug, error};
use wired_social::protocols::world_host::world_host_protocol_url;
use wired_world::world_server_capnp::world_server::{
    JoinParams, JoinResults, LeaveParams, LeaveResults, PlayerParams, PlayerResults, PlayersParams,
    PlayersResults, Server, TickrateParams, TickrateResults,
};

use crate::{
    global_context::GlobalContext,
    update_loop::{IncomingCommand, IncomingEvent, TICKRATE},
};

pub struct WorldServer {
    pub actor: Arc<Actor>,
    pub player_id: usize,
    pub context: Arc<GlobalContext>,
}

impl Server for WorldServer {
    fn join(&mut self, params: JoinParams, mut results: JoinResults) -> Promise<(), capnp::Error> {
        let params = pry!(params.get());
        let record_id = pry!(pry!(params.get_record_id()).to_string());

        debug!("Request to join world: {}", record_id);

        let actor = self.actor.clone();
        let context = self.context.clone();
        let player_id = self.player_id;
        let world_host_did = self.context.world_host_did.clone();

        Promise::from_future(async move {
            let mut success = results.get().init_success();

            match verify_world(actor, world_host_did, record_id.clone()).await {
                Ok(_) => {
                    debug!("World {} verified.", record_id);

                    context
                        .sender
                        .send(IncomingEvent {
                            command: IncomingCommand::JoinWorld { id: record_id },
                            player_id,
                        })
                        .map_err(|e| {
                            error!("Send failed: {}", e);
                            capnp::Error::from_kind(capnp::ErrorKind::Failed)
                        })?;

                    success.set_success(());
                }
                Err(e) => {
                    let e = e.to_string();
                    debug!("World error {}", e);
                    success.init_error(e.len() as u32).push_str(&e);
                }
            };

            Ok(())
        })
    }

    fn leave(&mut self, params: LeaveParams, _: LeaveResults) -> Promise<(), capnp::Error> {
        let params = pry!(params.get());
        let record_id = pry!(pry!(params.get_record_id()).to_string());

        let context = self.context.clone();
        let player_id = self.player_id;

        Promise::from_future(async move {
            context
                .sender
                .send(IncomingEvent {
                    command: IncomingCommand::LeaveWorld { id: record_id },
                    player_id,
                })
                .map_err(|e| {
                    error!("Send failed: {}", e);
                    capnp::Error::from_kind(capnp::ErrorKind::Failed)
                })?;

            Ok(())
        })
    }

    fn players(&mut self, _: PlayersParams, _: PlayersResults) -> Promise<(), capnp::Error> {
        todo!();
    }

    fn player(&mut self, _: PlayerParams, _: PlayerResults) -> Promise<(), capnp::Error> {
        todo!();
    }

    fn tickrate(
        &mut self,
        _: TickrateParams,
        mut results: TickrateResults,
    ) -> Promise<(), capnp::Error> {
        results.get().set_tickrate(TICKRATE);
        Promise::ok(())
    }
}

/// Verifies the provided `record_id` is a valid world.
async fn verify_world(actor: Arc<Actor>, world_host_did: String, record_id: String) -> Result<()> {
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
        debug!("Invalid world protocol: {:?}", descriptor.protocol);
        bail!("Invalid descriptor")
    }

    if descriptor.protocol_path != Some("world".to_string()) {
        debug!(
            "Invalid world protocol path: {:?}",
            descriptor.protocol_path
        );
        bail!("Invalid descriptor")
    }

    Ok(())
}
