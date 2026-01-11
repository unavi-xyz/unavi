use bevy::prelude::*;
use bevy_tnua::prelude::TnuaController;

use crate::{ControlScheme, Grounded, PlayerRig};

/// Syncs `TnuaController` airborne state to Grounded component for local players.
pub fn sync_grounded_state(
    mut rigs: Query<(&TnuaController<ControlScheme>, &mut Grounded), With<PlayerRig>>,
) {
    for (controller, mut grounded) in &mut rigs {
        if let Ok(airborne) = controller.is_airborne() {
            grounded.0 = !airborne;
        }
    }
}
