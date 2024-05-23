use std::str::Utf8Error;

use bevy::log::info;
use thiserror::Error;
use wired_world::world_server_capnp::{result::Which, world_server::Client};

#[derive(Error, Debug)]
pub enum JoinInstanceError {
    #[error(transparent)]
    Capnp(#[from] capnp::Error),
    #[error("Join request denied: {0}")]
    JoinDenied(String),
    #[error(transparent)]
    Utf8(#[from] Utf8Error),
}

pub async fn join_instance(rpc: &Client, record_id: String) -> Result<(), JoinInstanceError> {
    let mut request = rpc.join_instance_request();
    request.get().init_instance().set_record_id(record_id);

    let reply = request.send().promise.await?;
    let response = reply.get()?.get_response()?;

    match response.which().unwrap() {
        Which::Success(s) => {
            let ok = s?.get_ok();
            info!("Join success: {}", ok)
        }
        Which::Error(e) => {
            let e = e?.to_str()?;
            return Err(JoinInstanceError::JoinDenied(e.to_string()));
        }
    };

    Ok(())
}
