use bindings::exports::unavi::shapes::api::Guest;
use shapes::{cuboid::Cuboid, cylinder::Cylinder, sphere::Sphere};

mod attributes;
#[allow(warnings)]
mod bindings;
mod shapes;

struct GuestImpl;

impl Guest for GuestImpl {
    type Cuboid = Cuboid;
    type Cylinder = Cylinder;
    type Sphere = Sphere;
}

bindings::export!(GuestImpl with_types_in bindings);
