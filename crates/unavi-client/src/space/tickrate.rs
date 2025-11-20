use std::{sync::LazyLock, time::Duration};

use bevy::prelude::*;
use xdid::core::did_url::DidUrl;

use crate::space::{
    Space,
    networking::streams::publish::{PublishInterval, TransformPublishState},
};

#[derive(Debug)]
pub struct SetTickrate {
    pub space_url: DidUrl,
    pub tickrate: Duration,
}

pub static TICKRATE_QUEUE: LazyLock<(flume::Sender<SetTickrate>, flume::Receiver<SetTickrate>)> =
    LazyLock::new(flume::unbounded);

pub fn set_space_tickrates(
    time: Res<Time>,
    spaces: Query<(Entity, &Space)>,
    mut commands: Commands,
) {
    while let Ok(msg) = TICKRATE_QUEUE.1.try_recv() {
        for (entity, space) in spaces.iter() {
            if space.url != msg.space_url {
                continue;
            }

            commands.entity(entity).insert((
                PublishInterval {
                    last_tick: time.elapsed(),
                    tickrate: msg.tickrate,
                },
                TransformPublishState::default(),
            ));
            break;
        }
    }
}
