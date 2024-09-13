pub const PLAYER_HEIGHT: f32 = 1.7;
pub const PLAYER_WIDTH: f32 = 0.5;

#[cfg(feature = "physics")]
pub mod layers {
    use avian3d::prelude::LayerMask;

    pub const LAYER_LOCAL_PLAYER: LayerMask = LayerMask(1 << 0);
    pub const LAYER_OTHER_PLAYER: LayerMask = LayerMask(1 << 1);
    pub const LAYER_WORLD: LayerMask = LayerMask(1 << 2);
}
