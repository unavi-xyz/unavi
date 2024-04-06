use bevy::prelude::*;
use bevy_async_task::AsyncTaskRunner;
use didkit::{Document, ResolutionInputMetadata, ResolutionMetadata, DID_METHODS};
use dwn::{actor::Actor, store::SurrealStore};
use surrealdb::engine::local::Db;
use thiserror::Error;

const REGISTRY_DID: &str = "did:web:localhost%3A3000";

pub struct DidPlugin;

impl Plugin for DidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_world);
    }
}

#[derive(Resource)]
pub struct User {
    pub actor: Actor<SurrealStore<Db>, SurrealStore<Db>>,
}

fn create_world(mut task_runner: AsyncTaskRunner<()>) {
    task_runner.start(async {
        match resolve_did(REGISTRY_DID).await {
            Ok(doc) => {
                info!("Resolved registry: {:#?}", doc);
            }
            Err(err) => {
                error!("Failed to create world: {}", err);
            }
        }
    });
}

#[derive(Error, Debug)]
enum ResolveDidError {
    #[error("Failed to parse DID: {0}")]
    DidParse(&'static str),
    #[error("Failed to resolve DID: {0}")]
    Resolution(String),
}

async fn resolve_did(did: &str) -> Result<Document, ResolveDidError> {
    match DID_METHODS
        .get_method(did)
        .map_err(ResolveDidError::DidParse)?
        .to_resolver()
        .resolve(did, &ResolutionInputMetadata::default())
        .await
    {
        (
            ResolutionMetadata {
                error: Some(err), ..
            },
            _,
            _,
        ) => Err(ResolveDidError::Resolution(err)),
        (_, Some(doc), _) => Ok(doc),
        _ => Err(ResolveDidError::Resolution("Unexpected result".to_string())),
    }
}
