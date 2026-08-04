use serde::{
    Deserialize,
    Serialize,
};

use crate::attributes::Attribute;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NameAttr(pub String);

impl Attribute for NameAttr {
    const KEY: &'static str = "name";
}
