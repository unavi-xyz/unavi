use std::{
    sync::{
        Arc, LazyLock, Mutex,
        mpsc::{Receiver, Sender},
    },
    time::Duration,
};

use bevy::prelude::*;
use tokio::time::{MissedTickBehavior, interval};
use xdid::core::did_url::DidUrl;

use crate::space::{Space, stream::publish::PublishInterval};

pub struct SetTickrate {
    pub space_url: DidUrl,
    pub tickrate: Duration,
}

pub static TICKRATE_QUEUE: LazyLock<(Sender<SetTickrate>, Arc<Mutex<Receiver<SetTickrate>>>)> =
    LazyLock::new(|| {
        let (tx, rx) = std::sync::mpsc::channel();
        (tx, Arc::new(Mutex::new(rx)))
    });

pub fn set_space_tickrates(spaces: Query<(Entity, &Space)>, mut commands: Commands) {
    let Ok(rx) = TICKRATE_QUEUE.1.try_lock() else {
        return;
    };
    while let Ok(msg) = rx.try_recv() {
        for (entity, space) in spaces {
            if space.url != msg.space_url {
                continue;
            }

            let mut interval = interval(msg.tickrate);
            interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

            commands
                .entity(entity)
                .insert(PublishInterval(Arc::new(tokio::sync::Mutex::new(interval))));
            break;
        }
    }
}
