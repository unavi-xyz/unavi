use loro_surgeon::Hydrate;

pub mod xform;

pub trait Attribute: Hydrate + Send + Sync {
    fn merge(self, next: &Self) -> Self;
}
