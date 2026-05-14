use std::sync::Arc;

use bevy::prelude::*;
use loro::LoroDoc;

mod attributes;
mod diff;
mod subscribe;

pub struct HsdPlugin;

impl Plugin for HsdPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(subscribe::subscribe_to_docs);
    }
}

#[derive(Component)]
pub struct Hsd(pub Arc<LoroDoc>);
