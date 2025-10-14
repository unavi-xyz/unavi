use bevy::prelude::*;
use xdid::{
    core::did::Did,
    methods::key::{DidKeyPair, PublicKey, p256::P256KeyPair},
};
use zeroize::Zeroizing;

use crate::DIRS;

#[derive(Event, Default)]
pub struct LoginEvent;

pub fn trigger_login(world: &mut World) {
    world.trigger(LoginEvent);
}

#[derive(Resource, Default)]
/// Identity of the local user.
pub struct LocalIdentity {
    pub did: Option<Did>,
}

pub fn handle_login(_: Trigger<LoginEvent>, mut identity: ResMut<LocalIdentity>) {
    let pair = match get_or_create_keypair() {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to get or create keypair: {e:?}");
            return;
        }
    };

    let did = pair.public().to_did();
    info!("Loaded identity: {did}");

    identity.did = Some(did);
}

const KEY_FILE: &str = "key.pem";

fn get_or_create_keypair() -> anyhow::Result<P256KeyPair> {
    let dir = DIRS.data_local_dir();

    let key_path = {
        let mut path = dir.to_path_buf();
        path.push(KEY_FILE);
        path
    };

    if key_path.exists() {
        let raw = std::fs::read_to_string(key_path)?;
        let pem = Zeroizing::new(raw);
        let pair = P256KeyPair::from_pkcs8_pem(pem.as_str())?;
        Ok(pair)
    } else {
        let pair = P256KeyPair::generate();
        std::fs::write(&key_path, pair.to_pkcs8_pem()?)?;
        Ok(pair)
    }
}
