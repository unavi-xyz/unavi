use std::time::Duration;

use bevy::prelude::*;
use iroh::EndpointId;

pub mod agent;
pub mod object;

#[derive(Component)]
pub struct PeerStream(pub EndpointId);

#[derive(Component)]
#[require(LastTick)]
pub struct Tickrate(Duration);

impl Default for Tickrate {
    fn default() -> Self {
        Self(Duration::from_millis(50))
    }
}

#[derive(Component, Default)]
pub struct LastTick(Duration);
