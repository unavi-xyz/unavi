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
        util::shallow_map_updated_keys,
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
    let new_local = {
        let Ok(mut transform) = transforms.get_mut(trigger.entity) else {
            warn!("Transform not found");
            return;
        };
        match trigger.value {
            XformEvent::Rotation(v) => transform.rotation = v,
            XformEvent::Scale(v) => transform.scale = v,
            XformEvent::Translation(v) => transform.translation = v,
        }
        *transform
    };

    let Ok((position, rotation)) = physics.get_mut(trigger.entity) else {
        return;
    };
    if position.is_none() && rotation.is_none() {
        return;
    }

    let mut chain = vec![new_local];
    let mut current = parents.get(trigger.entity).ok().map(ChildOf::parent);
    while let Some(e) = current {
        chain.push(transforms.get(e).copied().unwrap_or(Transform::IDENTITY));
        current = parents.get(e).ok().map(ChildOf::parent);
    }
    let mut global = Transform::IDENTITY;
    for local in chain.iter().rev() {
        global = global.mul_transform(*local);
    }
    if let Some(mut p) = position {
        p.0 = global.translation;
    }
    if let Some(mut r) = rotation {
        r.0 = global.rotation;
    }
}
