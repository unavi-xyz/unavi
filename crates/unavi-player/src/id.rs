use bevy::prelude::*;
use bevy_vrm::BoneName;

#[derive(Component, PartialEq, Eq)]
pub struct PlayerId(pub u16);

pub(crate) fn id_player_bones(
    mut commands: Commands,
    new_bones: Query<Entity, Added<BoneName>>,
    parents: Query<&Parent>,
    players: Query<&PlayerId>,
) {
    for ent in new_bones.iter() {
        if let Some(id) = find_player_id(ent, &parents, &players) {
            commands.entity(ent).insert(PlayerId(id));
        }
    }
}

fn find_player_id(
    ent: Entity,
    parents: &Query<&Parent>,
    players: &Query<&PlayerId>,
) -> Option<u16> {
    if let Ok(parent) = parents.get(ent) {
        if let Ok(id) = players.get(**parent) {
            Some(id.0)
        } else {
            find_player_id(**parent, parents, players)
        }
    } else {
        None
    }
}
