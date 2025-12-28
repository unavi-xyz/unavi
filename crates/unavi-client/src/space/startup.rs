use std::sync::Arc;

use bevy::prelude::*;
use wds::{
    actor::Actor,
    record::schema::{SCHEMA_HOME, SCHEMA_SPACE},
};

use crate::networking::{
    WdsActor,
    thread::{NetworkCommand, NetworkingThread},
};

pub fn join_home_space(actor: Res<WdsActor>, nt: Res<NetworkingThread>) {
    let actor = Arc::clone(&actor);
    let command_tx = nt.command_tx.clone();

    std::thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("join_home_space")
            .build()
            .expect("build tokio runtime");

        rt.block_on(async move {
            if let Err(e) = join_home_space_inner(actor, command_tx).await {
                error!("Failed to join home space: {e:?}");
            }
        });
    });
}

async fn join_home_space_inner(
    actor: Arc<Actor>,
    command_tx: flume::Sender<NetworkCommand>,
) -> anyhow::Result<()> {
    let (id, _) = actor
        .create_record(Some(vec![*SCHEMA_HOME, *SCHEMA_SPACE]))
        .await?;

    info!("Created home space: {id}");
    command_tx.send_async(NetworkCommand::Join(id)).await?;

    Ok(())
}
