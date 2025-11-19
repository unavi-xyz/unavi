use std::{
    sync::{
        Arc, LazyLock, Mutex,
        mpsc::{Receiver, Sender},
    },
    time::Duration,
};

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

pub static TICKRATE_QUEUE: LazyLock<(Sender<SetTickrate>, Arc<Mutex<Receiver<SetTickrate>>>)> =
    LazyLock::new(|| {
        let (tx, rx) = std::sync::mpsc::channel();
        (tx, Arc::new(Mutex::new(rx)))
    });

pub fn set_space_tickrates(
    time: Res<Time>,
    spaces: Query<(Entity, &Space)>,
    mut commands: Commands,
) {
    let Ok(rx) = TICKRATE_QUEUE.1.try_lock() else {
        return;
    };
    while let Ok(msg) = rx.try_recv() {
        for (entity, space) in spaces {
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
