use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    prop_name,
    property::{
        Property,
        name::PropName,
    },
};

pub const GROUP: &str = "name";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NameAttr(pub String);

impl Property for NameAttr {
    const NAME: PropName = prop_name!(GROUP);
}
