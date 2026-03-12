use bevy::prelude::*;

pub mod collider;
pub mod material;
pub mod mesh;
pub mod node;
pub mod rigid_body;

#[derive(Component)]
pub struct Uncompiled;
