use bindings::exports::unavi::shapes::api::Guest;
use shapes::{
    axes::Axes, circle::Circle, cuboid::Cuboid, cylinder::Cylinder, ellipse::Ellipse,
    rectangle::Rectangle, sphere::Sphere,
};

#[allow(warnings)]
mod bindings;
mod shapes;
mod wired_math_impls;

struct GuestImpl;

impl Guest for GuestImpl {
    type Axes = Axes;
    type Circle = Circle;
    type Cuboid = Cuboid;
    type Cylinder = Cylinder;
    type Ellipse = Ellipse;
    type Rectangle = Rectangle;
    type Sphere = Sphere;
}

bindings::export!(GuestImpl with_types_in bindings);
