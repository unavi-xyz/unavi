use loro_surgeon::Hydrate;

pub mod xform;

pub trait Attribute: Hydrate + Send + Sync {
    #[must_use]
    fn merge(self, next: &Self) -> Self;
}
