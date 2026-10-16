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

/// Puts a document the script may write into the scene, or moves it.
pub async fn place(
    host: &ScriptHost,
    doc: u32,
    anchor: Anchor,
    offset: XformAttr,
) -> Result<(), ScriptError> {
    let id = host.writable_document(doc)?.id;
    let offset = checked(offset)?;
    host.world_call(move |world| place_in(world, id, anchor, offset))
        .await?
}

/// The smallest scale an offset may have on any axis.
///
/// A placed document's frame is the inverse of where it stands, so a flat or
/// vanishing scale would leave every conversion into that frame dividing by
/// nothing. A script hiding what it placed scales its own prims instead.
const MIN_SCALE: f32 = 1.0e-4;

fn checked(offset: XformAttr) -> Result<Transform, ScriptError> {
    match (
        finite::vec3(offset.translation),
        finite::quat(offset.rotation),
        finite::vec3(offset.scale),
    ) {
        (Some(_), Some(_), Some(scale)) if scale.abs().min_element() < MIN_SCALE => Err(
            ScriptError::invalid("an offset's scale must not be zero on any axis"),
        ),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn offset(scale: [f32; 3]) -> XformAttr {
        XformAttr {
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale,
            translation: [0.0; 3],
        }
    }

    #[test]
    fn an_offset_flat_on_any_axis_is_refused() {
        assert!(checked(offset([1.0, 1.0, 1.0])).is_ok());
        assert!(checked(offset([0.0, 0.0, 0.0])).is_err());
        assert!(checked(offset([1.0, -0.0, 1.0])).is_err());
        assert!(
            checked(offset([-1.0, 1.0, 1.0])).is_ok(),
            "a mirror is not flat"
        );
    }
}
