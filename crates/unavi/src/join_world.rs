use bevy::prelude::*;
use xdid::methods::web::reqwest::Url;

#[derive(Event)]
pub struct JoinWorld(pub ConnectInfo);

#[derive(Debug)]
pub struct ConnectInfo {
    pub url: Url,
    pub world_id: String,
}

pub fn handle_join_world(event: Trigger<JoinWorld>) {
    info!("Joining world: {}@{}", event.0.world_id, event.0.url);
}
