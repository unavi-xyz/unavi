use lorosurgeon::{Hydrate, Reconcile};

use crate::attributes::Attribute;

#[derive(Hydrate, Reconcile)]
pub struct Name {
    pub name: String,
}

impl Attribute for Name {
    const KEY: &str = "name";
}
