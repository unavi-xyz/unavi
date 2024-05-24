use std::str::Utf8Error;

use bevy::log::info;
use thiserror::Error;
use wired_world::world_server_capnp::{success::Which, world_server::Client};

#[derive(Error, Debug)]
pub enum JoinInstanceError {
    #[error(transparent)]
    Capnp(#[from] capnp::Error),
    #[error(transparent)]
    NotInSchema(#[from] capnp::NotInSchema),
    #[error("Join request denied: {0}")]
    JoinDenied(String),
    #[error(transparent)]
    Utf8(#[from] Utf8Error),
}

pub async fn join_instance(rpc: &Client, record_id: String) -> Result<(), JoinInstanceError> {
    let mut request = rpc.join_request();
    request.get().set_record_id(record_id);

    let reply = request.send().promise.await?;
    let success = reply.get()?.get_success()?;

    match success.which()? {
        Which::Success(_) => {
            info!("Join successful!");
        }
        Which::Error(e) => {
            let e = e?.to_str()?;
            return Err(JoinInstanceError::JoinDenied(e.to_string()));
        }
    };

    Ok(())
}
