use loro::LoroValue;

mod xform;

trait Attribute {
    fn parse(value: LoroValue) -> Self;
}
