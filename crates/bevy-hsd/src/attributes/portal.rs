use bevy::prelude::*;
use hsd::attributes::portal::PortalAttr;

use crate::attributes::apply_simple;

/// A prim's portal destination and size.
#[derive(Component, Debug, Clone, Copy)]
pub struct PortalConfig(pub PortalAttr);

pub(crate) fn apply(
    commands: &mut Commands,
    prim: Entity,
    payload: Option<&[u8]>,
) -> Result<(), postcard::Error> {
    apply_simple::<PortalAttr, PortalConfig>(commands, prim, payload, |attr| {
        Some(PortalConfig(attr))
    })
}
