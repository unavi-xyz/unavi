use exports::unavi::shapes::shapes::Guest;

mod cuboid;

wit_bindgen::generate!({ generate_all });

struct World;

impl Guest for World {
    type Cuboid = cuboid::Cuboid;
}

export!(World);
