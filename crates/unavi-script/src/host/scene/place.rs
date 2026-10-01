//! Putting documents into the scene. Placement is per peer and never stored.

use bevy::prelude::*;
use bevy_hsd::{
    anchor::{
        self,
        DocAnchor,
    },
    document::DocIndex,
    prim::PrimIndex,
};
use hsd::{
    attributes::xform::XformAttr,
    id::{
        DocId,
        PrimId,
    },
};
use unavi_physics::finite;

use crate::{
    error::ScriptError,
    host::ScriptHost,
};

/// What a placed document's root hangs from.
#[derive(Clone, Copy, Debug)]
pub enum Anchor {
    Space,
    Prim(DocId, PrimId),
    /// An entity the host resolved, such as a tracked camera.
    Entity(Entity),
}

/// Puts a document the script owns into the scene, or moves it.
pub async fn place(
    host: &ScriptHost,
    doc: u32,
    anchor: Anchor,
    offset: XformAttr,
) -> Result<(), ScriptError> {
    let id = host.owned_document(doc)?.id;
    let offset = checked(offset)?;
    host.world_call(move |world| place_in(world, id, anchor, offset))
        .await?
}

fn checked(offset: XformAttr) -> Result<Transform, ScriptError> {
    match (
        finite::vec3(offset.translation),
        finite::quat(offset.rotation),
        finite::vec3(offset.scale),
    ) {
        (Some(translation), Some(rotation), Some(scale)) => Ok(Transform {
            translation,
            rotation,
            scale,
        }),
        _ => Err(ScriptError::invalid("an offset must be finite")),
    }
}

fn place_in(
    world: &mut World,
    id: DocId,
    anchor: Anchor,
    offset: Transform,
) -> Result<(), ScriptError> {
    let index = world
        .get_resource::<DocIndex>()
        .ok_or(ScriptError::NotFound)?;
    let doc = index.get(id).ok_or(ScriptError::NotFound)?;
    let target = match anchor {
        Anchor::Space => None,
        Anchor::Prim(target_doc, prim) => {
            let target = index.get(target_doc).ok_or(ScriptError::NotFound)?;
            Some(
                world
                    .get::<PrimIndex>(target)
                    .and_then(|prims| prims.get(prim))
                    .ok_or(ScriptError::NotFound)?,
            )
        }
        Anchor::Entity(entity) => Some(entity),
    };
    anchor::place(&mut world.entity_mut(doc), DocAnchor { target, offset });
    Ok(())
}
