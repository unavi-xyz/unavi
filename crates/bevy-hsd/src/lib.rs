use bevy::prelude::*;

mod attributes;

pub struct HsdPulgin;

impl Plugin for HsdPulgin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Component)]
struct Layer;

#[derive(Component)]
struct Opinion;
