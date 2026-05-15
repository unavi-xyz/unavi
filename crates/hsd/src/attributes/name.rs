use lorosurgeon::{Hydrate, Reconcile};

use crate::attributes::Attribute;

#[derive(Hydrate, Reconcile, Debug)]
#[loro(default)]
pub struct NameAttr {
    pub name: String,
}

impl Attribute for NameAttr {
    const KEY: &str = "name";
}
