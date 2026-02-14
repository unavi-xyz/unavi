use crate::exports::unavi::shapes::api::{Guest, GuestCuboidBuilder};

wit_bindgen::generate!({
    generate_all,
});

struct World;

impl Guest for World {
    type CuboidBuilder = CuboidBuilder;
}

struct CuboidBuilder {}

impl GuestCuboidBuilder for CuboidBuilder {
    fn new() -> Self {
        Self {}
    }
}

export!(World);
