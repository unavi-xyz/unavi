use bevy::prelude::*;
use didkit::{DIDMethod, Source, JWK};

pub struct DidPlugin;

impl Plugin for DidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, generate_did_key);
    }
}

#[derive(Resource)]
pub struct UserDID {
    pub did_key: String,
    _key: JWK,
}

fn generate_did_key(mut commands: Commands) {
    let key = match JWK::generate_ed25519() {
        Ok(key) => key,
        Err(err) => {
            error!("Failed to generate JWK key: {}", err);
            return;
        }
    };

    let source = Source::Key(&key);

    let did = match did_method_key::DIDKey.generate(&source) {
        Some(did) => did,
        None => {
            error!("Failed to generate DID");
            return;
        }
    };

    info!("User DID: {}", did);

    commands.insert_resource(UserDID {
        did_key: did,
        _key: key,
    });
}
