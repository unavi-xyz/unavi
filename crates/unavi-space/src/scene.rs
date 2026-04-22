use bevy::prelude::*;
use bevy_wds::{LocalActor, SyncTargets};

use crate::Space;

pub fn spawn_space_scene(
    trigger: On<Add, Space>,
    spaces: Query<&Space>,
    actor: Query<(&LocalActor, &SyncTargets)>,
) {
    let Ok((_actor, _sync_targets)) = actor.single() else {
        warn!("space scene failed: no actor");
        return;
    };

    let _space = spaces.get(trigger.entity).map(|v| v.0).expect("space");

    // unavi_wasm_compat::spawn_thread(async move {
    //     let mut delay_secs = 4;
    //
    //     while let Err(err) = fetch_space_record(&actor, space).await {
    //         error!(?err, "error fetching space record");
    //         n0_future::time::sleep(Duration::from_secs(delay_secs)).await;
    //         delay_secs = delay_secs.wrapping_mul(2);
    //     }
    // });
}

// const SPACE_TTL: Duration = Duration::from_hours(24 * 7);
//
// async fn fetch_space_record(actor: &Actor, space: Hash) -> anyhow::Result<()> {
//     let builder = actor.read(space).ttl(SPACE_TTL);
//
//     let doc = builder.send().await?;
//
//     Ok(())
// }
