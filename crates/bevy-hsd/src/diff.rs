use std::sync::{
    Arc, Mutex,
    mpsc::{Receiver, Sender},
};

use bevy::prelude::*;
use loro::{TreeDiffItem, TreeID, ValueOrContainer};

use crate::attributes::AttrDataEvent;

pub type DiffSender = Arc<Sender<HsdDiffEvent>>;

pub enum HsdDiffEvent {
    Prim(TreeDiffItem),
    Attr {
        prim: TreeID,
        attr: String,
        value: Option<ValueOrContainer>,
    },
    AttrData {
        prim: TreeID,
        attr: &'static str,
        data: AttrDataEvent,
    },
}

#[derive(Component)]
pub struct DiffQueue(pub Arc<Mutex<Receiver<HsdDiffEvent>>>);

pub fn drain_diff_queues(queues: Query<&DiffQueue>, mut commands: Commands) {
    for q in queues {
        let Ok(q) = q.0.try_lock() else {
            continue;
        };

        while let Ok(event) = q.try_recv() {
            match event {
                HsdDiffEvent::Prim(d) => {}
                HsdDiffEvent::Attr { .. } => {}
                HsdDiffEvent::AttrData { .. } => {}
            }
        }
    }
}
