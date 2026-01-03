use bevy::prelude::*;
use wds::record::schema::{SCHEMA_HOME, SCHEMA_SPACE};

use crate::networking::{
    WdsActors,
    thread::{NetworkCommand, NetworkingThread},
};

pub fn join_home_space(actors: Res<WdsActors>, nt: Res<NetworkingThread>) {
    let actors = actors.clone();
    let command_tx = nt.command_tx.clone();

    std::thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("join_home_space")
            .build()
            .expect("build tokio runtime");

        rt.block_on(async move {
            if let Err(e) = join_home_space_inner(actors, command_tx).await {
                error!("Failed to join home space: {e:?}");
            }
        });
    });
}

async fn join_home_space_inner(
    actors: WdsActors,
    command_tx: flume::Sender<NetworkCommand>,
) -> anyhow::Result<()> {
    let did = actors.local.did();

    let res = actors
        .local
        .create_record()
        .add_schema(*SCHEMA_HOME, |_| Ok(()))?
        .add_schema(*SCHEMA_SPACE, |doc| {
            let map = doc.get_map("space");
            map.insert("name", format!("{did}'s Home"))?;
            Ok(())
        })?
        .sync_to(actors.remote)
        .send()
        .await?;

    info!("Created home space: {}", res.id);
    command_tx.send_async(NetworkCommand::Join(res.id)).await?;

    Ok(())
}
