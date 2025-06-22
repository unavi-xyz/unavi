use bevy::prelude::*;

use crate::{PlayerBody, PlayerHead, PlayerHeight};

pub fn position_head(
    bodies: Query<(&PlayerHeight, &Children), (With<PlayerBody>, Changed<PlayerHeight>)>,
    mut heads: Query<&mut Transform, With<PlayerHead>>,
) {
    for (player_height, children) in bodies {
        for child in children {
            let Ok(mut head_tr) = heads.get_mut(*child) else {
                continue;
            };

            // The body is 8 heads tall for adults, or 6 for children (see Da Vinci, Michelangelo).
            let heads_tall = if player_height.0 >= 1.5 { 8.0 } else { 6.0 };
            let head_height = player_height.0 - (player_height.0 / heads_tall);

            head_tr.translation.x = 0.0;
            head_tr.translation.y = head_height;
            head_tr.translation.z = 0.0;
        }
    }
}
