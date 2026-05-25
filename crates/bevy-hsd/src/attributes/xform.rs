use avian3d::prelude::{Position, Rotation};
use bevy::prelude::*;
use hsd::{
    HSD_CONTAINER_ID,
    attributes::{Attribute, hydrate_attr, xform::XformAttr},
};
use loro::{ContainerID, Index, TreeID, ValueOrContainer, event::Diff};

use crate::{
    attributes::{
        ApplyEvent, AttrDataEvent, AttributeParser, DocContext, ParseError,
        util::{compute_global_transform, shallow_map_updated_keys},
    },
    diff::HsdDiffEvent,
};

#[derive(Debug)]
pub enum XformEvent {
    Rotation(Quat),
    Scale(Vec3),
    Translation(Vec3),
}

pub struct XformParser;

impl AttributeParser for XformParser {
    fn key(&self) -> &'static str {
        XformAttr::KEY
    }

    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        value: Option<ValueOrContainer>,
    ) -> Result<(), ParseError> {
        if value.is_some() {
            commands.entity(prim).insert(Transform::default());
        } else {
            commands.entity(prim).remove::<Transform>();
        }
        Ok(())
    }

    fn parse(
        &self,
        ctx: &DocContext,
        prim: TreeID,
        path: &[(ContainerID, Index)],
        diff: Diff,
    ) -> Result<(), ParseError> {
        let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
        let meta = tree.get_meta(prim)?;

        let attr: XformAttr = hydrate_attr(&meta)?;

        let keys = shallow_map_updated_keys(path, diff)?;
        for key in keys {
            let event = match key.as_str() {
                "rotation" => XformEvent::Rotation(Quat::from_slice(&attr.rotation)),
                "scale" => XformEvent::Scale(Vec3::from_slice(&attr.scale)),
                "translation" => XformEvent::Translation(Vec3::from_slice(&attr.translation)),
                _ => continue,
            };
            ctx.tx
                .send(HsdDiffEvent::AttrData {
                    prim,
                    data: AttrDataEvent::Xform(event),
                })
                .map_err(|_| ParseError::SendDiff)?;
        }
        Ok(())
    }
}

pub fn apply_xform(
    trigger: On<ApplyEvent<XformEvent>>,
    mut transforms: Query<&mut Transform>,
    mut physics: Query<(Option<&mut Position>, Option<&mut Rotation>)>,
    parents: Query<&ChildOf>,
) {
    {
        let Ok(mut transform) = transforms.get_mut(trigger.entity) else {
            warn!("Transform not found");
            return;
        };
        match trigger.value {
            XformEvent::Rotation(v) => transform.rotation = v,
            XformEvent::Scale(v) => transform.scale = v,
            XformEvent::Translation(v) => transform.translation = v,
        }
    }

    let Ok((position, rotation)) = physics.get_mut(trigger.entity) else {
        return;
    };
    if position.is_none() && rotation.is_none() {
        return;
    }

    let global = compute_global_transform(trigger.entity, &transforms.as_readonly(), &parents);
    if let Some(mut p) = position {
        p.0 = global.translation;
    }
    if let Some(mut r) = rotation {
        r.0 = global.rotation;
    }
}
