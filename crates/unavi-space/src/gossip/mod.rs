use bevy::prelude::*;
use bevy_iroh::{IrohEndpoint, RouterBuilderFn, RouterBuilderFnTarget};
use bevy_wds::{LocalActor, SyncTargets};
use blake3::Hash;
use iroh::EndpointId;
use iroh_gossip::{Gossip, TopicId, api::JoinOptions};
use wds::actor::Actor;

use crate::Space;

mod bootstrap;

#[derive(Component)]
pub struct IrohGossip(Gossip);

pub fn spawn_gossip(
    trigger: On<Add, IrohEndpoint>,
    endpoints: Query<&IrohEndpoint>,
    mut commands: Commands,
) {
    let endpoint = endpoints.get(trigger.entity).expect("endpoint");
    let gossip = Gossip::builder().spawn(endpoint.0.clone());

    commands
        .entity(trigger.entity)
        .insert(IrohGossip(gossip.clone()));

    commands.spawn((
        RouterBuilderFnTarget(trigger.entity),
        RouterBuilderFn(Some(Box::new(|router| {
            router.accept(iroh_gossip::ALPN, gossip)
        }))),
    ));
}

pub struct GossipCtx {
    endpoint_id: EndpointId,
    gossip: Gossip,
    actor: Actor,
    sync_targets: Vec<Actor>,
}

pub fn on_space_add(
    trigger: On<Add, Space>,
    spaces: Query<&Space>,
    endpoints: Query<(&IrohEndpoint, &IrohGossip)>,
    actors: Query<(&LocalActor, &SyncTargets)>,
) {
    let space = spaces.get(trigger.entity).map(|s| s.0).expect("space");

    let Some((endpoint, gossip)) = endpoints.into_iter().next() else {
        return;
    };

    let Some((actor, sync_targets)) = actors.into_iter().next() else {
        return;
    };

    let ctx = GossipCtx {
        endpoint_id: endpoint.0.id(),
        gossip: gossip.0.clone(),
        actor: actor.0.clone(),
        sync_targets: sync_targets.0.clone(),
    };

    unavi_wasm_compat::spawn_thread(async move {
        if let Err(err) = handle_space_topic(ctx, space).await {
            error!(?err, "error handling space topic");
        }
    });
}

async fn handle_space_topic(ctx: GossipCtx, space: Hash) -> anyhow::Result<()> {
    let peers = bootstrap::find_bootstrap_peers(&ctx, space).await?;

    let topic_id = TopicId::from_bytes(*space.as_bytes());
    let topic = ctx
        .gossip
        .subscribe_with_opts(
            topic_id,
            JoinOptions {
                bootstrap: peers,
                subscription_capacity: 256,
            },
        )
        .await?;
    let (tx, mut rx) = topic.split();

    Ok(())
}

pub fn on_space_remove(_trigger: On<Remove, Space>) {
    // TODO leave gossip topic
}
