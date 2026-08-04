use avian3d::prelude::{
    Position,
    Rotation,
};
use bevy::prelude::*;
use hsd::attributes::{
    Attribute,
    xform::XformAttr,
};

use crate::attributes::{
    AttributeParser,
    ParseError,
    util::compute_global_transform,
};

#[derive(Component, Debug, Clone, Copy)]
pub struct XformData(pub XformAttr);

pub struct XformParser;

impl AttributeParser for XformParser {
    fn key(&self) -> &'static str {
        XformAttr::KEY
    }

    /// Removal resets the transform rather than removing it: `Prim` requires
    /// `Transform`, and a removed one breaks propagation to every child.
    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        payload: Option<&[u8]>,
    ) -> Result<(), ParseError> {
        match payload {
            Some(payload) => {
                commands
                    .entity(prim)
                    .insert(XformData(XformAttr::decode(payload)?));
            }
            None => {
                commands
                    .entity(prim)
                    .remove::<XformData>()
                    .insert(Transform::default());
            }
        }
        Ok(())
    }
}

pub fn apply_xform(
    changed: Query<(Entity, &XformData), Changed<XformData>>,
    mut transforms: Query<&mut Transform>,
    mut physics: Query<(Option<&mut Position>, Option<&mut Rotation>)>,
    parents: Query<&ChildOf>,
) {
    for (entity, data) in &changed {
        {
            let Ok(mut transform) = transforms.get_mut(entity) else {
                warn!("Transform not found");
                continue;
            };
            transform.translation = Vec3::from_slice(&data.0.translation);
            transform.rotation = Quat::from_slice(&data.0.rotation);
            transform.scale = Vec3::from_slice(&data.0.scale);
        }

        let Ok((position, rotation)) = physics.get_mut(entity) else {
            continue;
        };
        if position.is_none() && rotation.is_none() {
            continue;
        }

        let global = compute_global_transform(entity, &transforms.as_readonly(), &parents);
        if let Some(mut p) = position {
            p.0 = global.translation;
        }
        if let Some(mut r) = rotation {
            r.0 = global.rotation;
        }
    }
}
