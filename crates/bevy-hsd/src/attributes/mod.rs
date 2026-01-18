use loro_surgeon::Hydrate;

mod xform;

trait Attribute: Hydrate {
    fn merge(self, next: Self) -> Self;
}
