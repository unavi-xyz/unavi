use std::sync::Arc;

use bevy::prelude::*;

use crate::networking::{WdsActor, thread::NetworkingThread};

pub fn join_home_space(actor: Res<WdsActor>, nt: Res<NetworkingThread>) {
    let _actor = Arc::clone(&actor);
    let _command_tx = nt.command_tx.clone();

    std::thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("join_home_space")
            .build()
            .expect("build tokio runtime");

        rt.block_on(async move {
            // if let Err(e) = join_home_space_inner(actor, command_tx).await {
            //     error!("Failed to join home space: {e:?}");
            // }
        });
    });
}

// async fn join_home_space_inner(
//     actor: Arc<Actor>,
//     command_tx: flume::Sender<NetworkCommand>,
// ) -> anyhow::Result<()> {
//     let did = actor.did();
//     let id = actor.create_record(None).await?;
//
//     actor
//         .update_record(id, |r| {
//             let map = r.get_map("space");
//             map.insert("name", format!("{did}'s Space"))?;
//             Ok(())
//         })
//         .await?;
//
//     info!("Joining home space: {id}");
//     command_tx.send_async(NetworkCommand::Join(id)).await?;
//
//     Ok(())
// }
