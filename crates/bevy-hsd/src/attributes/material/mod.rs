pub mod pbr;
pub(crate) mod source;

use bevy::{
    pbr::MeshMaterial3d,
    prelude::*,
};
use hsd::{
    attributes::material::MaterialAttr,
    property::{
        Payload,
        Property,
        name::PropName,
    },
};

use crate::attributes::material::pbr::HsdMaterial;

#[derive(Component, Clone)]
pub(crate) struct MaterialData(pub MaterialAttr);

pub(crate) fn apply(
    commands: &mut Commands,
    prim: Entity,
    name: &PropName,
    payload: Option<&[u8]>,
) -> Result<(), postcard::Error> {
    if *name != MaterialAttr::NAME {
        return Ok(());
    }
    match payload {
        // Render components wait for `MaterialSource` to pick the backend.
        Some(payload) => {
            commands
                .entity(prim)
                .insert(MaterialData(MaterialAttr::decode(payload)?));
        }
        None => {
            commands
                .entity(prim)
                .remove::<HsdMaterial>()
                .remove::<MaterialData>()
                .remove::<MeshMaterial3d<StandardMaterial>>();
        }
    }
    Ok(())
}
