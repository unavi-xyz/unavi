use avian3d::prelude::*;

#[derive(PhysicsLayer, Clone, Copy, Debug)]
pub enum GameLayer {
    LocalPlayer,
    OtherPlayer,
    World,
}

pub const LOCAL_PLAYER_LAYER: LayerMask = LayerMask(1 << 0);
pub const OTHER_PLAYER_LAYER: LayerMask = LayerMask(1 << 1);
pub const WORLD_LAYER: LayerMask = LayerMask(1 << 2);
