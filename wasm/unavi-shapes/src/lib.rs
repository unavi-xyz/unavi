use bindings::exports::unavi::shapes::api::Guest;
use shapes::{cuboid::Cuboid, sphere::Sphere};

mod attributes;
#[allow(warnings)]
mod bindings;
mod shapes;

struct GuestImpl;

impl Guest for GuestImpl {
    type Cuboid = Cuboid;
    type Sphere = Sphere;
}

bindings::export!(GuestImpl with_types_in bindings);
