use bevy::prelude::*;
use iroh::EndpointId;

use crate::networking::event::PlayerInboundState;

#[derive(Component, Deref)]
pub struct OtherPlayer(pub EndpointId);

pub fn receive_player_transforms(mut players: Query<(&PlayerInboundState, &mut Transform)>) {
    for (state, mut root) in &mut players {
        let Some(iframe) = state.latest_iframe.try_lock() else {
            continue;
        };
        let Some(pframe) = state.latest_pframe.try_lock() else {
            continue;
        };
        let Some(iframe) = iframe.as_ref() else {
            continue;
        };
        let Some(pframe) = pframe.as_ref() else {
            continue;
        };

        root.translation.clone_from(&iframe.pose.root.pos.into());

        for _pose in &iframe.pose.bones {
            // TODO apply bones
        }

        if pframe.iframe_id == iframe.id {
            // Apply p-frame.
            root.translation = pframe.pose.root.pos.apply_to(root.translation);
            root.rotation.clone_from(&pframe.pose.root.rot.into());

            for _pose in &pframe.pose.bones {
                // TODO apply bones
            }
        } else {
            // Only apply i-frame.
            root.rotation.clone_from(&iframe.pose.root.rot.into());
        }
    }
}
