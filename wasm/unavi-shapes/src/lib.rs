use bindings::exports::unavi::shapes::api::Guest;
use shapes::{cuboid::Cuboid, cylinder::Cylinder, rectangle::Rectangle, sphere::Sphere};

mod attributes;
#[allow(warnings)]
mod bindings;
mod shapes;
mod wired_math_impls;

struct GuestImpl;

impl Guest for GuestImpl {
    type Cuboid = Cuboid;
    type Cylinder = Cylinder;
    type Rectangle = Rectangle;
    type Sphere = Sphere;
}

bindings::export!(GuestImpl with_types_in bindings);
