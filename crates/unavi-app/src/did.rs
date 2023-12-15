use bevy::prelude::*;
use didkit::DIDMethod;

pub struct DidPlugin;

impl Plugin for DidPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UserDID>()
            .add_systems(Startup, set_user_did);
    }
}

#[derive(Default, Resource)]
pub struct UserDID(pub String);

fn set_user_did(mut commands: Commands) {
    let key = match didkit::JWK::generate_ed25519() {
        Ok(key) => key,
        Err(err) => {
            error!("Failed to generate JWK key: {}", err);
            return;
        }
    };

    let source = didkit::Source::Key(&key);

    let did = match did_method_key::DIDKey.generate(&source) {
        Some(did) => did,
        None => {
            error!("Failed to generate DID");
            return;
        }
    };

    info!("User DID: {}", did);

    commands.insert_resource(UserDID(did));
}
