use loro_surgeon::Hydrate;

pub mod collider;
pub mod material;
pub mod mesh;
pub mod rigid_body;
pub mod xform;

pub trait Attribute: Hydrate + Send + Sync {
    #[must_use]
    fn merge(self, next: &Self) -> Self;
}
