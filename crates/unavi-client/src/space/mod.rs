use bevy::prelude::*;
use xdid::core::did_url::DidUrl;

pub mod connection;
pub mod join;
pub mod transform;

#[derive(Component)]
pub struct Space {
    pub url: DidUrl,
}
